#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

mod app;
mod error;
mod galoy_transactions;
pub mod job;
pub mod user_trade_balances;
pub mod user_trade_unit;
pub mod user_trades;

use galoy_client::GaloyClientConfig;
use shared::pubsub::*;

pub use app::*;
pub use error::*;

pub async fn run(
    pool: sqlx::PgPool,
    config: UserTradesConfig,
    pubsub_cfg: PubSubConfig,
    galoy_client_cfg: GaloyClientConfig,
) -> Result<(), UserTradesError> {
    UserTradesApp::run(pool, config, pubsub_cfg, galoy_client_cfg).await?;
    Ok(())
}
