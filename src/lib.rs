use spacetimedb::sats::u256;
use spacetimedb::{reducer, table, Identity, ReducerContext, Table};

#[table(name = player, public)]
pub struct Player {
    #[primary_key]
    identity: Identity,
    username: Option<String>,
    money: u256,
    click_power: u64,
    online: bool,
}

#[reducer(init)]
pub fn init(_ctx: &ReducerContext) {
}

#[reducer(client_connected)]
pub fn identity_connected(ctx: &ReducerContext) {
    let identity = ctx.sender;

    if ctx.db.player().identity().find(identity).is_none() {
        ctx.db.player().insert(Player {
            identity,
            username: None,
            money: u256::new(0),
            click_power: 1,
            online: true,
        });
    } else {
        if let Some(mut player) = ctx.db.player().identity().find(identity) {
            player.online = true;
            ctx.db.player().identity().update(player);
        }
    }
}

#[reducer(client_disconnected)]
pub fn identity_disconnected(_ctx: &ReducerContext) {
}

#[reducer]
pub fn increase_money(ctx: &ReducerContext) -> Result<(), String> {
    if let Some(mut player) = ctx.db.player().identity().find(ctx.sender) {
        player.money += u256::from(player.click_power);
        ctx.db.player().identity().update(player);
        Ok(())
    } else {
        Err("Cannot reward bonus for unknown player".to_string())
    }
}


#[reducer]
pub fn set_name(ctx: &ReducerContext, username: String) -> Result<(), String> {
    if let Some(player) = ctx.db.player().identity().find(ctx.sender) {
        ctx.db.player().identity().update(Player { username: Some(username), ..player });
        Ok(())
    } else {
        Err("Cannot set name for unknown user".to_string())
    }
}