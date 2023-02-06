use sqlx_ledger::{balance::AccountBalance, AccountId as LedgerAccountId, Currency, SqlxLedger};
use tracing::instrument;

use crate::{constants::*, LedgerError};

pub struct Balances<'a> {
    pub(super) inner: &'a SqlxLedger,
    pub(super) usd: Currency,
    pub(super) btc: Currency,
}

impl<'a> Balances<'a> {
    pub async fn stablesats_liability(&self) -> Result<Option<AccountBalance>, LedgerError> {
        self.get_ledger_account_balance(STABLESATS_LIABILITY_ID, self.usd)
            .await
    }

    pub async fn stablesats_btc_wallet(&self) -> Result<Option<AccountBalance>, LedgerError> {
        self.get_ledger_account_balance(STABLESATS_BTC_WALLET_ID, self.btc)
            .await
    }

    #[instrument(name = "ledger.get_ledger_account_balance", skip(self))]
    pub async fn get_ledger_account_balance(
        &self,
        account_id: impl Into<LedgerAccountId> + std::fmt::Debug,
        currency: Currency,
    ) -> Result<Option<AccountBalance>, LedgerError> {
        Ok(self
            .inner
            .balances()
            .find(STABLESATS_JOURNAL_ID.into(), account_id.into(), currency)
            .await?)
    }
}