use rust_decimal_macros::dec;
use std::fs;

use price_server::{app::*, ExchangePriceCacheConfig};
use shared::{payload::*, pubsub::*, time::*};

#[derive(serde::Deserialize)]
struct Fixture {
    payloads: Vec<PriceMessagePayload>,
}

fn load_fixture() -> anyhow::Result<Fixture> {
    let contents =
        fs::read_to_string("./tests/fixtures/price_app.json").expect("Couldn't load fixtures");
    Ok(serde_json::from_str(&contents)?)
}

#[tokio::test]
async fn price_app() -> anyhow::Result<()> {
    let (tick_send, tick_recv) =
        memory::channel(chrono::Duration::from_std(std::time::Duration::from_secs(2)).unwrap());
    let publisher = tick_send.clone();
    let mut subscriber = tick_recv.resubscribe();

    let (_, recv) = futures::channel::mpsc::unbounded();

    let ex_cfgs = ExchangeWeights {
        okex: Some(dec!(1.0)),
        bitfinex: None,
    };

    let app = PriceApp::run(
        recv,
        PriceServerHealthCheckConfig::default(),
        FeeCalculatorConfig {
            base_fee_rate: dec!(0.001),
            immediate_fee_rate: dec!(0.01),
            delayed_fee_rate: dec!(0.1),
        },
        tick_recv,
        ExchangePriceCacheConfig::default(),
        ex_cfgs,
    )
    .await?;

    let err = app
        .get_cents_from_sats_for_immediate_buy(Sats::from_major(100_000_000))
        .await;
    if let Err(PriceAppError::ExchangePriceCacheError(ExchangePriceCacheError::NoPriceAvailable)) =
        err
    {
        assert!(true)
    } else {
        assert!(false)
    }

    let mut payloads = load_fixture()?.payloads.into_iter();
    let mut payload = payloads.next().unwrap();
    tick_send
        .publish(PriceStreamPayload::OkexBtcSwapPricePayload(payload.clone()))
        .await?;
    subscriber.next().await;
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    let err = app
        .get_cents_from_sats_for_immediate_buy(Sats::from_major(100_000_000))
        .await;
    if let Err(PriceAppError::ExchangePriceCacheError(ExchangePriceCacheError::StalePrice(_))) = err
    {
        assert!(true)
    } else {
        assert!(false)
    }

    payload.timestamp = TimeStamp::now();
    publisher
        .publish(PriceStreamPayload::OkexBtcSwapPricePayload(payload))
        .await?;
    subscriber.next().await;
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    let cents = app
        .get_cents_from_sats_for_immediate_buy(Sats::from_major(100_000_000))
        .await?;
    assert_eq!(cents, UsdCents::from_major(98900));
    let cents = app
        .get_cents_from_sats_for_immediate_buy(Sats::from_major(1))
        .await?;
    assert_eq!(cents, UsdCents::from_major(0));

    let cents = app
        .get_cents_from_sats_for_immediate_sell(Sats::from_major(100_000_000))
        .await?;
    assert_eq!(cents, UsdCents::from_major(1011000));
    let cents = app
        .get_cents_from_sats_for_immediate_sell(Sats::from_major(1))
        .await?;
    assert_eq!(cents, UsdCents::from_major(1));

    let cents = app
        .get_cents_from_sats_for_future_buy(Sats::from_major(100_000_000))
        .await?;
    assert_eq!(cents, UsdCents::from_major(89900));
    let cents = app
        .get_cents_from_sats_for_future_buy(Sats::from_major(1))
        .await?;
    assert_eq!(cents, UsdCents::from_major(0));

    let future_buy = app
        .get_cents_from_sats_for_future_sell(Sats::from_major(100_000_000))
        .await?;
    assert_eq!(future_buy, UsdCents::from_major(1101000));
    let future_buy = app
        .get_cents_from_sats_for_future_sell(Sats::from_major(1))
        .await?;
    assert_eq!(future_buy, UsdCents::from_major(1));

    let sats = app
        .get_sats_from_cents_for_immediate_buy(UsdCents::from_major(1000000))
        .await?;
    assert_eq!(sats, Sats::from_major(1011000000));

    let sats = app
        .get_sats_from_cents_for_immediate_sell(UsdCents::from_major(1000000))
        .await?;
    assert_eq!(sats, Sats::from_major(98900000));
    let sats = app
        .get_sats_from_cents_for_immediate_sell(UsdCents::from_major(1))
        .await?;
    assert_eq!(sats, Sats::from_major(98));

    let sats = app
        .get_sats_from_cents_for_future_buy(UsdCents::from_major(1000000))
        .await?;
    assert_eq!(sats, Sats::from_major(1101000000));
    let sats = app
        .get_sats_from_cents_for_future_buy(UsdCents::from_major(1))
        .await?;
    assert_eq!(sats, Sats::from_major(1101));

    let sats = app
        .get_sats_from_cents_for_future_sell(UsdCents::from_major(1000000))
        .await?;
    assert_eq!(sats, Sats::from_major(89900000));
    let sats = app
        .get_sats_from_cents_for_future_sell(UsdCents::from_major(1))
        .await?;
    assert_eq!(sats, Sats::from_major(89));

    let ratio = app.get_cents_per_sat_exchange_mid_rate().await?;
    assert_eq!(ratio, 0.0055);

    Ok(())
}
