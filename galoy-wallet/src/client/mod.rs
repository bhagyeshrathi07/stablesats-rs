mod error;
mod queries;

use futures::{
    stream::{self},
    Stream,
};
use graphql_client::{reqwest::post_graphql, GraphQLQuery, Response};
use reqwest::{
    header::{HeaderMap, HeaderValue, AUTHORIZATION},
    Client as ReqwestClient,
};

use error::*;
pub use queries::*;

use auth_code::*;
use btc_price::*;
use transactions_list::*;
use user_login::*;

#[derive(Debug)]
pub struct StablesatsWalletsBalances {
    pub btc_wallet_balance: Option<queries::SignedAmount>,
    pub usd_wallet_balance: Option<queries::SignedAmount>,
}

#[derive(Debug)]
pub struct StablesatsWalletTransactions {
    pub btc_transactions:
        Option<transactions_list::TransactionsListMeDefaultAccountWalletsTransactions>,
    pub usd_transactions:
        Option<transactions_list::TransactionsListMeDefaultAccountWalletsTransactions>,
}

#[derive(Debug, Clone)]
pub struct GaloyClient {
    client: ReqwestClient,
    config: GaloyClientConfig,
}

#[derive(Debug, Clone)]
pub struct GaloyClientConfig {
    pub api: String,
    pub code: String,
    pub phone_number: String,
    pub jwt: String,
}

impl GaloyClient {
    pub fn new(config: GaloyClientConfig) -> Self {
        Self {
            client: ReqwestClient::new(),
            config,
        }
    }

    pub async fn btc_price(&self) -> Result<BtcPriceBtcPrice, GaloyWalletError> {
        let variables = btc_price::Variables;
        let response =
            post_graphql::<BtcPrice, _>(&self.client, &self.config.api, variables).await?;
        let response_data = response.data;

        if let Some(response_data) = response_data {
            let result = response_data.btc_price;
            if let Some(result) = result {
                return Ok(result);
            }
        }
        Err(GaloyWalletError::UnknownResponse(
            "Failed to parse response data".to_string(),
        ))
    }

    pub async fn authentication_code(
        &self,
    ) -> Result<AuthCodeUserRequestAuthCode, GaloyWalletError> {
        let phone_number = auth_code::Variables {
            input: auth_code::UserRequestAuthCodeInput {
                phone: self.config.phone_number.clone(),
            },
        };
        let response =
            post_graphql::<AuthCode, _>(&self.client, &self.config.api, phone_number).await?;
        let response_data = response.data;

        if let Some(response_data) = response_data {
            let result = response_data.user_request_auth_code;

            return Ok(result);
        }
        Err(GaloyWalletError::UnknownResponse(
            "Failed to parse response data".to_string(),
        ))
    }

    pub async fn login(&mut self) -> Result<UserLoginUserLogin, GaloyWalletError> {
        let variables = user_login::Variables {
            input: user_login::UserLoginInput {
                code: self.config.code.clone(),
                phone: self.config.phone_number.clone(),
            },
        };

        let response =
            post_graphql::<UserLogin, _>(&self.client, &self.config.api, variables).await?;

        let response_data = response.data;

        if let Some(response_data) = response_data {
            let result = response_data.user_login;

            // Update config JWT
            self.config.jwt = match result.clone().auth_token {
                Some(jwt) => jwt,
                None => "".to_string(),
            };

            return Ok(result);
        }
        Err(GaloyWalletError::UnknownResponse(
            "Failed to parse response data".to_string(),
        ))
    }

    pub async fn transactions_list(
        &mut self,
        last_transaction_cursor: String,
        wallet_currency: transactions_list::WalletCurrency,
    ) -> Result<
        std::pin::Pin<
            Box<impl Stream<Item = TransactionsListMeDefaultAccountWalletsTransactionsEdges>>,
        >,
        GaloyWalletError,
    > {
        let header_value = format!("Bearer {}", self.config.jwt);
        let mut header = HeaderMap::new();
        header.insert(AUTHORIZATION, HeaderValue::from_str(header_value.as_str())?);

        let variables = transactions_list::Variables {
            last: None,
            first: None,
            before: None,
            after: Some(last_transaction_cursor),
        };

        let request_body = TransactionsList::build_query(variables);
        let response = self
            .client
            .post(&self.config.api)
            .headers(header)
            .json(&request_body)
            .send()
            .await?;

        let response_body: Response<transactions_list::ResponseData> = response.json().await?;
        let response_data = response_body.data;

        let result = match response_data {
            Some(data) => data,
            None => {
                return Err(GaloyWalletError::UnknownResponse(
                    "Empty`data` response data".to_string(),
                ))
            }
        };

        let user = result.me;

        let default_account = match user {
            Some(data) => data.default_account,
            None => {
                return Err(GaloyWalletError::UnknownResponse(
                    "Empty `me` response data".to_string(),
                ))
            }
        };

        let wallet = default_account
            .wallets
            .into_iter()
            .find(|wallet| wallet.wallet_currency == wallet_currency);

        if let Some(wallet) = wallet {
            let transaction_edges = match wallet.transactions {
                Some(tx) => tx.edges,
                None => {
                    return Err(GaloyWalletError::UnknownResponse(
                        "Empty `transactions` response data".to_string(),
                    ))
                }
            };

            let transactions = match transaction_edges {
                Some(txs) => txs,
                None => {
                    return Err(GaloyWalletError::UnknownResponse(
                        "Empty `edges` response data".to_string(),
                    ))
                }
            };

            return Ok(Box::pin(stream::iter(transactions)));
        }
        Err(GaloyWalletError::UnknownResponse(
            "Failed to parse response data".to_string(),
        ))
    }

    pub async fn wallets_balances(&self) -> Result<StablesatsWalletsBalances, GaloyWalletError> {
        let header_value = format!("Bearer {}", self.config.jwt);
        let mut header = HeaderMap::new();
        header.insert(AUTHORIZATION, HeaderValue::from_str(header_value.as_str())?);

        let variables = wallets::Variables;
        let request_body = Wallets::build_query(variables);
        let response = self
            .client
            .post(&self.config.api)
            .headers(header)
            .json(&request_body)
            .send()
            .await?;

        let response_body: Response<wallets::ResponseData> = response.json().await?;
        if response_body.errors.is_some() {
            if let Some(error) = response_body.errors {
                return Err(GaloyWalletError::GrapqQlApi(error[0].clone().message));
            }
        }

        let response_data = response_body.data;
        if response_data.is_none() {
            return Err(GaloyWalletError::UnknownResponse(
                "Empty `data` in response data".to_string(),
            ));
        }

        let me = match response_data {
            Some(data) => data.me,
            None => {
                return Err(GaloyWalletError::UnknownResponse(
                    "Empty `data` in response data".to_string(),
                ))
            }
        };

        let default_account = match me {
            Some(me) => me.default_account,
            None => {
                return Err(GaloyWalletError::UnknownResponse(
                    "Empty `me` in response data".to_string(),
                ))
            }
        };

        let wallets = default_account.wallets;

        let btc_wallet = wallets
            .clone()
            .into_iter()
            .find(|wallet| wallet.wallet_currency == wallets::WalletCurrency::BTC);

        let usd_wallet = wallets
            .into_iter()
            .find(|wallet| wallet.wallet_currency == wallets::WalletCurrency::USD);

        let btc_wallet_balance: Option<SignedAmount> = match btc_wallet {
            Some(wallet) => Some(wallet.balance),
            None => None,
        };

        let usd_wallet_balance: Option<SignedAmount> = match usd_wallet {
            Some(wallet) => Some(wallet.balance),
            None => None,
        };

        Ok(StablesatsWalletsBalances {
            usd_wallet_balance,
            btc_wallet_balance,
        })
    }
}
