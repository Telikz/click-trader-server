use spacetimedb::sats::u256;
use spacetimedb::{reducer, table, Identity, ReducerContext, ScheduleAt, Table, TimeDuration, Timestamp};

#[table(name = player, public)]
pub struct Player {
    #[primary_key]
    identity: Identity,
    username: Option<String>,
    money: u256,
    passive_income: u128,
    click_power: u64,
    click_timer: i64,
    online: bool,
    upgrades: Vec<u16>,
    last_click: Timestamp,
}

#[table(name = upgrades, public)]
pub struct Upgrades {
    #[unique]
    #[auto_inc]
    #[primary_key]
    id: u16,
    level: u8,
    cost: u128,
    title: String,
    identifier: String,
    description: String,
    passive_income_bonus: Option<u64>,
    click_power_bonus: Option<u64>,
    click_timer_bonus: Option<i64>,
}

#[table(name = update_player_schedule, scheduled(update_players))]
pub struct UpdatePlayersSchedule {
    #[primary_key]
    #[auto_inc]
    id: u64,
    scheduled_at: ScheduleAt,
}

#[reducer]
pub fn update_players(ctx: &ReducerContext, _args: UpdatePlayersSchedule) -> Result<(), String> {
    for mut player in ctx.db.player().iter() {
        if player.online && player.passive_income > 0 {
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
pub fn identity_disconnected(_ctx: &ReducerContext) {}

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
pub fn buy_upgrade(ctx: &ReducerContext, upgrade_id: u16) -> Result<(), String> {
    let Some(mut player) = ctx.db.player().identity().find(ctx.sender) else {
        return Err("Player not found".to_string());
    };

    if player.upgrades.contains(&upgrade_id) {
        return Err("Upgrade already owned".to_string());
    }

    let Some(upgrade) = ctx.db.upgrades().id().find(upgrade_id) else {
        return Err("Upgrade not found".to_string());
    };

    if player.money < u256::from(upgrade.cost) {
        return Err("Not enough money".to_string());
    }

    // Deduct cost
    player.money -= u256::from(upgrade.cost);

    // Apply upgrade effects
    if let Some(bonus) = upgrade.passive_income_bonus {
        player.passive_income += bonus as u128;
    }

    if let Some(bonus) = upgrade.click_power_bonus {
        player.click_power += bonus;
    }

    if let Some(bonus) = upgrade.click_timer_bonus {
        player.click_timer = player.click_timer.saturating_sub(bonus);
    }

    // Add upgrade to owned list
    player.upgrades.push(upgrade_id);

    // Save
    ctx.db.player().identity().update(player);
    Ok(())
}

#[reducer]
pub fn add_upgrade(
    ctx: &ReducerContext,
    identifier: String,
    title: String,
    description: String,
    level: u8,
    cost: u128,
    passive_income_bonus: Option<u64>,
    click_power_bonus: Option<u64>,
    click_timer_bonus: Option<i64>,
) -> Result<(), String> {
    ctx.db.upgrades().insert(Upgrades {
        id: 0,
        title,
        level,
        cost,
        passive_income_bonus,
        click_power_bonus,
        click_timer_bonus,
        identifier,
        description,
    });
    Ok(())
}
