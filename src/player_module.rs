use spacetimedb::sats::u256;
use spacetimedb::{reducer, table, Identity, ReducerContext, ScheduleAt, SpacetimeType, Table, TimeDuration, Timestamp};

#[derive(SpacetimeType)]
pub struct StockType{
    pub stock_id: u16,
    pub amount: u64,
}

#[table(name = player, public)]
pub struct Player {
    #[primary_key]
    pub identity: Identity,
    pub username: Option<String>,
    pub money: u256,
    pub passive_income: u128,
    pub click_power: u128,
    pub click_timer: i64,
    pub online: bool,
    pub upgrades: Vec<u16>,
    pub stocks: Vec<StockType>,
    pub last_click: Timestamp,
}
#[table(name = update_player_schedule, scheduled(update_players))]
pub struct UpdatePlayersSchedule {
    #[primary_key]
    #[auto_inc]
    pub id: u64,
    pub scheduled_at: ScheduleAt,
}

#[reducer]
pub fn update_players(ctx: &ReducerContext, _args: UpdatePlayersSchedule) -> Result<(), String> {
    for mut player in ctx.db.player().iter() {
        if player.passive_income > 0 {
            player.money += u256::from(player.passive_income);
            ctx.db.player().identity().update(player);
        }
    }

    let one_second = TimeDuration::from_micros(1_000_000);
    let future_timestamp: Timestamp = ctx.timestamp + one_second;

    ctx.db.update_player_schedule().insert(UpdatePlayersSchedule {
        id: 0,
        scheduled_at: future_timestamp.into(),
    });

    Ok(())
}

#[reducer]
pub fn set_name(ctx: &ReducerContext, username: String) -> Result<(), String> {
    let cleaned = username.trim();
    if cleaned.is_empty() {
        return Err("Username cannot be empty.".to_string());
    }

    if let Some(player) = ctx.db.player().identity().find(ctx.sender) {
        ctx.db.player().identity().update(Player {
            username: Some(cleaned.to_string()),
            ..player
        });
        Ok(())
    } else {
        Err("Cannot set name for unknown user".to_string())
    }
}

#[reducer]
pub fn increase_money(ctx: &ReducerContext) -> Result<(), String> {
    if let Some(mut player) = ctx.db.player().identity().find(ctx.sender) {
        let duration = TimeDuration::from_micros(player.click_timer);
        let now = ctx.timestamp;

        match player.last_click.checked_add(duration) {
            Some(allowed_time) if now < allowed_time => {
                Err("Clicking too fast. Wait for the timer.".to_string())
            }
            Some(_) => {
                player.money += u256::from(player.click_power);
                player.last_click = now;
                ctx.db.player().identity().update(player);
                Ok(())
            }
            None => Err("Failed to calculate cooldown time.".to_string()),
        }
    } else {
        Err("Cannot reward bonus for unknown player".to_string())
    }
}