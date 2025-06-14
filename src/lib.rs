mod player_module;
mod upgrades_module;
use crate::player_module::{player, update_player_schedule, Player, UpdatePlayersSchedule};
use spacetimedb::sats::u256;
use spacetimedb::{reducer, ReducerContext, Table, TimeDuration, Timestamp};

#[reducer(init)]
pub fn init(ctx: &ReducerContext) {
    let one_second = TimeDuration::from_micros(1_000_000);
    let future_timestamp: Timestamp = ctx.timestamp + one_second;

    ctx.db.update_player_schedule().insert(UpdatePlayersSchedule {
        id: 0,
        scheduled_at: future_timestamp.into(),
    });
}

#[reducer(client_connected)]
pub fn identity_connected(ctx: &ReducerContext) {
    let identity = ctx.sender;

    if ctx.db.player().identity().find(identity).is_none() {
        ctx.db.player().insert(Player {
            identity,
            username: None,
            money: u256::new(0),
            passive_income: 0,
            click_power: 1,
            click_timer: 1_000_000, // microseconds (1s)
            last_click: ctx.timestamp,
            online: true,
            upgrades: Vec::new(),
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
