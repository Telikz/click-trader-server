use crate::stock_module::{create_stock, init_market_config};
use crate::upgrades_module::add_upgrade;
use spacetimedb::ReducerContext;

pub fn initializer(ctx: &ReducerContext) {
    init_market_config(ctx, 20, 10, 1).expect("Market failed to init configuration");
    init_upgrades(ctx);
    init_stocks(ctx);
}

pub fn init_upgrades(ctx: &ReducerContext) {
    add_upgrade(
        ctx,
        "passive_income_lv1".into(),
        "Passive Income I".into(),
        "Gain +0.1 passive income/sec.".into(),
        1,
        1_000,
        Some(100),
        None,
        None,
    )
    .expect("Failed to add upgrade");

    add_upgrade(
        ctx,
        "passive_income_lv2".into(),
        "Passive Income II".into(),
        "Gain +0.5 passive income/sec.".into(),
        2,
        5_000,
        Some(500),
        None,
        None,
    )
    .expect("Failed to add upgrade");

    add_upgrade(
        ctx,
        "click_power_lv1".into(),
        "Click Power I".into(),
        "Increase click power by 1.".into(),
        1,
        2_000,
        None,
        Some(1),
        None,
    )
    .expect("Failed to add upgrade");

    add_upgrade(
        ctx,
        "faster_clicks".into(),
        "Faster Clicks".into(),
        "Reduce click timer by 200ms.".into(),
        1,
        3_000,
        None,
        None,
        Some(200_000), // microseconds
    )
    .expect("Failed to add upgrade");
}

pub fn init_stocks(ctx: &ReducerContext) {
    create_stock(
        ctx,
        "Banana Corp".into(),
        "Leading exporter of potassium-based optimism.".into(),
        1,
        1_000_000,
    )
    .expect("Failed to create Banana Corp");

    create_stock(
        ctx,
        "ByteMiner".into(),
        "Blockchain infrastructure for frogs.".into(),
        5, // 5.000 price
        500_000,
    )
    .expect("Failed to create ByteMiner");

    create_stock(
        ctx,
        "SkyRockets".into(),
        "Aerospace dreams, Earthly debt.".into(),
        10,
        1_000_000_000,
    )
    .expect("Failed to create SkyRockets");

    create_stock(
        ctx,
        "SleepyCoffee".into(),
        "The only coffee that makes you nap faster.".into(),
        2,
        100_000,
    )
    .expect("Failed to create SleepyCoffee");
}
