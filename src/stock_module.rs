use crate::constants::PRICE_SCALE_FACTOR;
use crate::player_module::{player, StockType};
use spacetimedb::rand::Rng;
use spacetimedb::sats::u256;
use spacetimedb::{reducer, table, ReducerContext, ScheduleAt, Table, TimeDuration};


/// How much momentum is retained each tick. 90 means 90% persistence.
const MOMENTUM_PERSISTENCE_FACTOR: i32 = 9;
const MOMENTUM_PERSISTENCE_DIVISOR: i32 = 10;

/// Divisor to scale the impact of raw buy/sell demand. A higher value makes
/// the market less sensitive to single large trades, preventing manipulation.
const DEMAND_IMPACT_DIVISOR: i32 = 100;

/// Percentage-point bonus/penalty for active events.
const HYPE_EVENT_BONUS: i32 = 5;
const SCANDAL_EVENT_PENALTY: i32 = -5;

/// Chance for an event to start or end each tick.
const EVENT_START_CHANCE: f64 = 0.05;
const EVENT_END_CHANCE: f64 = 0.25;

/// The interval for running the market update logic.
const UPDATE_INTERVAL_MICROS: u64 = 5_000_000; // 5 seconds

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
    pub momentum: i32,
    pub volatility: u8,
    pub recent_buys: u64,
    pub recent_sells: u64,
    pub trending_event: Option<String>,
}

#[reducer]
pub fn create_stock(
    ctx: &ReducerContext,
    name: String,
    description: String,
    initial_price: u128,
    total_shares: u64,
    volatility: u8,
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
        momentum: 0,
        volatility,
        recent_buys: 0,
        recent_sells: 0,
        trending_event: None,
    });
    Ok(())
}

#[reducer]
pub fn buy_stock(ctx: &ReducerContext, stock_id: u16, amount: u64) -> Result<(), String> {
    let Some(mut player) = ctx.db.player().identity().find(ctx.sender) else {
        return Err("Player not found.".to_string());
    };

    let Some(mut stock) = ctx.db.stock().id().find(stock_id) else {
        return Err("Stock not found.".to_string());
    };

    if stock.available_shares < amount {
        return Err("Not enough shares available to buy.".to_string());
    }

    let price = stock
        .price_per_share
        .checked_mul(u128::from(amount))
        .ok_or("Overflow in total price calculation.".to_string())?;

    if player.money < u256::from(price) {
        return Err("Player does not have enough money.".to_string());
    }

    player.money -= u256::from(price);
    stock.available_shares -= amount;
    stock.recent_buys += amount;

    match player.stocks.iter_mut().find(|s| s.stock_id == stock_id) {
        Some(existing) => existing.amount += amount,
        None => player.stocks.push(StockType { stock_id, amount }),
    }

    ctx.db.player().identity().update(player);
    ctx.db.stock().id().update(stock);

    Ok(())
}

#[reducer]
pub fn sell_stock(ctx: &ReducerContext, stock_id: u16, amount: u64) -> Result<(), String> {
    let Some(mut player) = ctx.db.player().identity().find(ctx.sender) else {
        return Err("Player not found.".to_string());
    };

    let Some(mut stock) = ctx.db.stock().id().find(stock_id) else {
        return Err("Stock not found.".to_string());
    };

    let Some(existing) = player.stocks.iter_mut().find(|s| s.stock_id == stock_id) else {
        return Err("Player does not own this stock.".to_string());
    };

    if existing.amount < amount {
        return Err("Not enough shares to sell.".to_string());
    }

    let gain = stock
        .price_per_share
        .checked_mul(amount as u128)
        .ok_or("Overflow in gain calculation.")?;

    player.money += u256::from(gain);
    stock.available_shares += amount;
    stock.recent_sells += amount;
    existing.amount -= amount;

    if existing.amount == 0 {
        player.stocks.retain(|s| s.stock_id != stock_id);
    }

    ctx.db.player().identity().update(player);
    ctx.db.stock().id().update(stock);

    Ok(())
}


#[table(name = stock_market_schedule, scheduled(update_stock_prices))]
pub struct StockMarketSchedule {
    #[primary_key]
    #[auto_inc]
    pub id: u64,
    pub scheduled_at: ScheduleAt,
}

#[reducer]
pub fn update_stock_prices(ctx: &ReducerContext, _args: StockMarketSchedule) -> Result<(), String> {
    let mut rng = ctx.rng();

    for mut stock in ctx.db.stock().iter() {
        let vol_range = stock.volatility as i32;
        let base_change = rng.gen_range(-vol_range..=vol_range);

       
        let net_demand = stock.recent_buys as i32 - stock.recent_sells as i32;
        let demand_boost = net_demand / DEMAND_IMPACT_DIVISOR;

        let event_bonus = match stock.trending_event.as_deref() {
            Some("🔥 Hype") => HYPE_EVENT_BONUS,
            Some("📉 Scandal") => SCANDAL_EVENT_PENALTY,
            _ => 0,
        };

        let total_change_percent = base_change + (stock.momentum / 10) + demand_boost + event_bonus;
        let price = stock.price_per_share as i128;

        let delta = (price * total_change_percent as i128) / 100;
        let new_price = (price + delta).max(1);

        stock.last_price = stock.price_per_share;
        stock.price_per_share = new_price as u128;

        let new_momentum = (stock.momentum * MOMENTUM_PERSISTENCE_FACTOR) / MOMENTUM_PERSISTENCE_DIVISOR
            + total_change_percent;
        stock.momentum = new_momentum.clamp(-100, 100);

        stock.recent_buys = 0;
        stock.recent_sells = 0;

        if stock.trending_event.is_none() {
            if rng.gen_bool(EVENT_START_CHANCE) {
                stock.trending_event = Some(if rng.gen_bool(0.5) {
                    "🔥 Hype".to_string()
                } else {
                    "📉 Scandal".to_string()
                });
            }
        } else if rng.gen_bool(EVENT_END_CHANCE) {
            stock.trending_event = None;
        }

        ctx.db.stock().id().update(stock);
    }

    let next_update_time = ctx.timestamp + TimeDuration::from_micros(UPDATE_INTERVAL_MICROS as i64);
    ctx.db.stock_market_schedule().insert(StockMarketSchedule {
        id: 0,
        scheduled_at: next_update_time.into(),
    });

    Ok(())
}