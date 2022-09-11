use rust_decimal::Decimal;
use sqlx::{Executor, PgPool, Postgres, Transaction};
use uuid::Uuid;

use shared::pubsub::CorrelationId;

use crate::error::HedgingError;

#[derive(Clone)]
pub struct SynthUsdLiability {
    pool: PgPool,
}

impl SynthUsdLiability {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn insert_if_new<'a>(
        &self,
        correlation_id: CorrelationId,
        amount: Decimal,
    ) -> Result<Option<Transaction<'a, Postgres>>, HedgingError> {
        let mut tx = self.pool.begin().await?;
        tx.execute("SET TRANSACTION ISOLATION LEVEL SERIALIZABLE;")
            .await?;

        let result = sqlx::query_file!(
            "src/synth_usd_liability/sql/insert-if-new.sql",
            amount,
            Uuid::from(correlation_id)
        )
        .fetch_all(&mut tx)
        .await?;

        if result.is_empty() {
            Ok(None)
        } else {
            Ok(Some(tx))
        }
    }

    pub async fn get_latest_liability(&self) -> Result<Decimal, HedgingError> {
        let result =
            sqlx::query!("SELECT amount FROM synth_usd_liability ORDER BY idx DESC LIMIT 1")
                .fetch_one(&self.pool)
                .await?;

        Ok(result.amount)
    }
}