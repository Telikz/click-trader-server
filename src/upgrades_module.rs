use crate::player_module::player;
use spacetimedb::sats::u256;
use spacetimedb::{reducer, table, ReducerContext, Table};

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
