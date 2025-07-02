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
        "Gain +0.2 passive income/sec.".into(),
        1,
        10_000,
        Some(200),
        None,
        None,
    )
    .expect("Failed to add upgrade");

    add_upgrade(
        ctx,
        "passive_income_lv2".into(),
        "Passive Income II".into(),
        "Gain +1.0 passive income/sec.".into(),
        2,
        50_000,
        Some(1_000),
        None,
        None,
    )
    .expect("Failed to add upgrade");

    add_upgrade(
        ctx,
        "passive_income_lv3".into(),
        "Passive Income III".into(),
        "Gain +5.0 passive income/sec.".into(),
        3,
        300_000,
        Some(5_000),
        None,
        None,
    )
    .expect("Failed to add upgrade");

    add_upgrade(
        ctx,
        "click_power_lv1".into(),
        "Click Power I".into(),
        "Increase click power by 2.".into(),
        1,
        20_000,
        None,
        Some(2),
        None,
    )
    .expect("Failed to add upgrade");

    add_upgrade(
        ctx,
        "click_power_lv2".into(),
        "Click Power II".into(),
        "Increase click power by 5.".into(),
        2,
        100_000,
        None,
        Some(5),
        None,
    )
    .expect("Failed to add upgrade");

    add_upgrade(
        ctx,
        "click_power_lv3".into(),
        "Click Power III".into(),
        "Increase click power by 10.".into(),
        3,
        500_000,
        None,
        Some(10),
        None,
    )
    .expect("Failed to add upgrade");

    add_upgrade(
        ctx,
        "faster_clicks".into(),
        "Faster Clicks".into(),
        "Reduce click timer by 100 ms.".into(),
        1,
        30_000,
        None,
        None,
        Some(100_000),
    )
    .expect("Failed to add upgrade");

    add_upgrade(
        ctx,
        "turbo_clicks".into(),
        "Turbo Clicks".into(),
        "Reduce click timer by 500 ms.".into(),
        1,
        200_000,
        None,
        None,
        Some(500_000),
    )
    .expect("Failed to add upgrade");
}

pub fn init_stocks(ctx: &ReducerContext) {
    create_stock(
        ctx,
        "QuantumCompute".into(),
        "Next-gen quantum processors at scale.".into(),
        100,
        1_000_000_000_000_000,
        50
    )
    .expect("Failed to create QuantumCompute");

    create_stock(
        ctx,
        "EtherFiber".into(),
        "Distributed broadband over blockchain.".into(),
        50,
        500_000_000_000_000,
        50
    )
    .expect("Failed to create EtherFiber");

    create_stock(
        ctx,
        "MarsVacations".into(),
        "Luxury travel packages to Mars colonies.".into(),
        20,
        2_000_000_000_000_000,
        50
    )
    .expect("Failed to create MarsVacations");

    create_stock(
        ctx,
        "CaffeineInc".into(),
        "Instant power-nap coffee crystals.".into(),
        1,
        5_000_000_000_000_000,
        50
    )
    .expect("Failed to create CaffeineInc");

    create_stock(
        ctx,
        "RoboFarm".into(),
        "Autonomous farming robots and AI-driven yields.".into(),
        10,
        750_000_000_000_000,
        50
    )
    .expect("Failed to create RoboFarm");

    create_stock(
        ctx,
        "GenAILabs".into(),
        "Cutting-edge generative AI research co-op.".into(),
        250,
        100_000_000_000_000,
        50
    )
    .expect("Failed to create GenAILabs");
}
