use crate::constants::PRICE_SCALE_FACTOR;
use crate::constants::{DECIMAL_SCALE_FACTOR, STOCK_UPDATE_INTERVAL_MICROS};
use crate::transaction_module::update_transactions;
use spacetimedb::{reducer, table, ReducerContext, ScheduleAt, Table};
use std::time::Duration;

#[table(name = stock, public)]
pub struct Stock {
    #[unique]
    #[auto_inc]
    #[primary_key]
    pub id: u16,
    pub name: String,
    pub description: String,
    pub price_per_share: u128,
    pub total_shares: u64,
    pub available_shares: u64,
    pub last_price: u128,
    pub recent_buys: u64,
    pub recent_sells: u64,
}

#[table(name = stock_market_schedule, scheduled(update_stock_prices))]
pub struct StockMarketSchedule {
    #[primary_key]
    #[auto_inc]
    pub id: u64,
    pub scheduled_at: ScheduleAt,
}

#[table(name = market_config, public)]
pub struct MarketConfig {
    pub sensitivity: u64,
    pub slippage_factor: u64,
    pub min_price: u128,
}

pub fn init_market_config(
    ctx: &ReducerContext,
    sensitivity_scaled: u64,
    slippage_factor_scaled: u64,
    min_price_scaled: u128,
) -> Result<(), String> {
    ctx.db.market_config().insert(MarketConfig {
        sensitivity: sensitivity_scaled,
        slippage_factor: slippage_factor_scaled,
        min_price: min_price_scaled,
    });

    Ok(())
}


#[reducer]
pub fn create_stock(
    ctx: &ReducerContext,
    name: String,
    description: String,
    initial_price: u128,
    total_shares: u64,
) -> Result<(), String> {
    if total_shares == 0 {
        return Err("Total shares cannot be zero.".to_string());
    }

    let price_per_share = initial_price
        .checked_mul(PRICE_SCALE_FACTOR)
        .ok_or("Initial price is too large, results in overflow.".to_string())?;

    ctx.db.stock().insert(Stock {
        id: 0,
        name,
        description,
        price_per_share,
        total_shares,
        available_shares: total_shares,
        last_price: price_per_share,
        recent_buys: 0,
        recent_sells: 0,
    });
    Ok(())
}

#[reducer]
pub fn update_stock_prices(ctx: &ReducerContext, _args: StockMarketSchedule) -> Result<(), String> {
    let config = ctx
        .db
        .market_config()
        .iter()
        .next()
        .ok_or("Market configuration not initialized.")?;

    if let Err(e) = update_transactions(ctx) {
        log::error!("Could not process transactions during market update: {}", e);
    }

    for mut stock in ctx.db.stock().iter() {
        let buys = stock.recent_buys as i128;
        let sells = stock.recent_sells as i128;

        if buys == 0 && sells == 0 {
            continue;
        }

        let net_demand = buys - sells;
        let demand_units = net_demand / 100;

        let price = stock.price_per_share as i128;
        let sensitivity = config.sensitivity as i128;
        let scale = DECIMAL_SCALE_FACTOR as i128;

        let delta = (price * demand_units * sensitivity) / scale;

        let slippage = config.slippage_factor as i128;
        let adjusted_delta = (delta * (scale - slippage)) / scale;

        let new_price = (price + adjusted_delta).max(config.min_price as i128) as u128;

        stock.last_price = stock.price_per_share;
        stock.price_per_share = new_price;
        stock.recent_buys = 0;
        stock.recent_sells = 0;

        ctx.db.stock().id().update(stock);
    }

    ctx.db.stock_market_schedule().insert(StockMarketSchedule {
        id: 0,
        scheduled_at: (ctx.timestamp + Duration::from_micros(STOCK_UPDATE_INTERVAL_MICROS)).into(),
    });

    Ok(())
}