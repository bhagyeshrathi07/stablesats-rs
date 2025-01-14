use anyhow::Context;
use serde::{Deserialize, Serialize};
use std::path::Path;

use galoy_client::GaloyClientConfig;
use hedging::{ExchangesConfig, HedgingAppConfig};
use price_server::{
    ExchangePriceCacheConfig, FeeCalculatorConfig, PriceServerConfig, PriceServerHealthCheckConfig,
};
use shared::pubsub::PubSubConfig;
use user_trades::UserTradesConfig;

use super::{db::DbConfig, tracing::TracingConfig};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub db: DbConfig,
    #[serde(default)]
    pub pubsub: PubSubConfig,
    #[serde(default)]
    pub tracing: TracingConfig,
    #[serde(default)]
    pub price_server: PriceServerWrapper,
    #[serde(default)]
    pub bitfinex_price_feed: BitfinexPriceFeedConfigWrapper,
    #[serde(default)]
    pub user_trades: UserTradesConfigWrapper,
    #[serde(default)]
    pub galoy: GaloyClientConfig,
    #[serde(default)]
    pub hedging: HedgingConfigWrapper,
    #[serde(default)]
    pub exchanges: ExchangesConfig,
}

pub struct EnvOverride {
    pub redis_password: Option<String>,
    pub pg_con: String,
    pub okex_secret_key: String,
    pub okex_passphrase: String,
    pub galoy_phone_code: String,
    pub bitfinex_secret_key: String,
}

impl Config {
    pub fn from_path(
        path: impl AsRef<Path>,
        EnvOverride {
            redis_password,
            galoy_phone_code,
            okex_passphrase,
            okex_secret_key,
            pg_con: stablesats_pg_con,
            bitfinex_secret_key: _,
        }: EnvOverride,
    ) -> anyhow::Result<Self> {
        let config_file = std::fs::read_to_string(path).context("Couldn't read config file")?;
        let mut config: Config =
            serde_yaml::from_str(&config_file).context("Couldn't parse config file")?;
        if let Some(redis_password) = redis_password {
            config.pubsub.password = Some(redis_password);
        }

        config.galoy.auth_code = galoy_phone_code;

        if let Some(okex) = config.exchanges.okex.as_mut() {
            okex.config.client.secret_key = okex_secret_key;
            okex.config.client.passphrase = okex_passphrase;
        };

        config.db.pg_con = stablesats_pg_con;

        Ok(config)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceServerWrapper {
    #[serde(default = "bool_true")]
    pub enabled: bool,
    #[serde(default)]
    pub health: PriceServerHealthCheckConfig,
    #[serde(default)]
    pub server: PriceServerConfig,
    #[serde(default)]
    pub fees: FeeCalculatorConfig,
    #[serde(default)]
    pub price_cache: ExchangePriceCacheConfig,
}
impl Default for PriceServerWrapper {
    fn default() -> Self {
        Self {
            enabled: true,
            server: PriceServerConfig::default(),
            health: PriceServerHealthCheckConfig::default(),
            fees: FeeCalculatorConfig::default(),
            price_cache: ExchangePriceCacheConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BitfinexPriceFeedConfigWrapper {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub config: bitfinex_price::PriceFeedConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserTradesConfigWrapper {
    #[serde(default = "bool_true")]
    pub enabled: bool,
    #[serde(default)]
    pub config: UserTradesConfig,
}
impl Default for UserTradesConfigWrapper {
    fn default() -> Self {
        Self {
            enabled: true,
            config: UserTradesConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HedgingConfigWrapper {
    #[serde(default = "bool_true")]
    pub enabled: bool,
    #[serde(default)]
    pub config: HedgingAppConfig,
}
impl Default for HedgingConfigWrapper {
    fn default() -> Self {
        Self {
            enabled: true,
            config: HedgingAppConfig::default(),
        }
    }
}

fn bool_true() -> bool {
    true
}
