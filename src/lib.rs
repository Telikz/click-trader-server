mod constants;
mod player_module;
mod stock_module;
mod transaction_module;
mod upgrades_module;
mod initializer;

use crate::constants::{
    PLAYER_STARTING_CLICK_POWER, PLAYER_STARTING_CLICK_TIMER_MICROS, PLAYER_STARTING_MONEY,
    PLAYER_STARTING_PASSIVE_INCOME, PLAYER_STARTING_STOCK_BUY_FEE, PLAYER_STARTING_STOCK_SELL_FEE,
    PLAYER_UPDATE_INTERVAL_MICROS, STOCK_UPDATE_INTERVAL_MICROS,
};
use crate::initializer::initializer;
use crate::player_module::{player, update_player_schedule, Player, UpdatePlayersSchedule};
use crate::stock_module::{stock_market_schedule, StockMarketSchedule};
use spacetimedb::{reducer, ReducerContext, Table};
use std::time::Duration;

#[reducer(init)]
pub fn init(ctx: &ReducerContext) {
    initializer(ctx);
    
    ctx.db
        .update_player_schedule()
        .insert(UpdatePlayersSchedule {
            id: 0,
            scheduled_at: (ctx.timestamp + Duration::from_micros(PLAYER_UPDATE_INTERVAL_MICROS)).into(),
        });
    ctx.db.stock_market_schedule().insert(StockMarketSchedule {
        id: 0,
        scheduled_at: (ctx.timestamp + Duration::from_micros(STOCK_UPDATE_INTERVAL_MICROS)).into(),
    });
}

#[reducer(client_connected)]
pub fn identity_connected(ctx: &ReducerContext) {
    let identity = ctx.sender;

    if ctx.db.player().identity().find(identity).is_none() {
        ctx.db.player().insert(Player {
            identity,
            username: None,
            money: PLAYER_STARTING_MONEY,
            passive_income: PLAYER_STARTING_PASSIVE_INCOME,
            click_power: PLAYER_STARTING_CLICK_POWER,
            click_timer: PLAYER_STARTING_CLICK_TIMER_MICROS,
            stock_buy_fee: PLAYER_STARTING_STOCK_BUY_FEE,
            stock_sell_fee: PLAYER_STARTING_STOCK_SELL_FEE,
            last_click: ctx.timestamp,
            upgrades: Vec::new(),
            stocks: Vec::new(),
            online: true,
        });
    } else if let Some(mut player) = ctx.db.player().identity().find(identity) {
        player.online = true;
        ctx.db.player().identity().update(player);
    }
}

#[reducer(client_disconnected)]
pub fn identity_disconnected(ctx: &ReducerContext) {
    let identity = ctx.sender;

    if ctx.db.player().identity().find(identity).is_none() {
    } else if let Some(mut player) = ctx.db.player().identity().find(identity) {
        player.online = false;
        ctx.db.player().identity().update(player);
    }
}
