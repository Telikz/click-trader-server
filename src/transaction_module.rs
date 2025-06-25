use crate::constants::PRICE_SCALE_FACTOR;
use crate::player_module::{player, StockType};
use crate::stock_module::stock;
use spacetimedb::{reducer, table, Identity, ReducerContext, SpacetimeType, Table, Timestamp};

#[derive(SpacetimeType, PartialEq, Clone)]
pub enum TransactionStatus {
    Pending,
    Confirmed,
    Rejected,
}

#[derive(SpacetimeType, PartialEq, Clone)]
pub enum TransactionType {
    Buy,
    Sell,
}

#[table(name = transaction, public)]
pub struct Transaction {
    #[unique]
    #[auto_inc]
    #[primary_key]
    pub id: u16,
    pub sender: Identity,
    pub stock_id: u16,
    pub amount: u64,
    pub tx_type: TransactionType,
    pub status: TransactionStatus,
    pub timestamp: Timestamp,
}

#[reducer]
pub fn create_transaction(
    ctx: &ReducerContext,
    stock_id: u16,
    amount: u64,
    tx_type: TransactionType,
) -> Result<(), String> {
    let Some(_player) = ctx.db.player().identity().find(ctx.sender) else {
        return Err("Player not found.".to_string());
    };

    let Some(_stock) = ctx.db.stock().id().find(stock_id) else {
        return Err("Stock not found.".to_string());
    };

    ctx.db.transaction().insert(Transaction {
        id: 0,
        sender: ctx.sender,
        stock_id,
        amount,
        tx_type,
        status: TransactionStatus::Pending,
        timestamp: ctx.timestamp,
    });

    Ok(())
}

#[reducer]
pub fn update_transactions(ctx: &ReducerContext) -> Result<(), String> {
    for tx in ctx
        .db
        .transaction()
        .iter()
        .filter(|t| t.status == TransactionStatus::Pending)
    {
        let mut tx = tx;

        let Some(mut player) = ctx.db.player().identity().find(tx.sender) else {
            tx.status = TransactionStatus::Rejected;
            ctx.db.transaction().id().update(tx);
            continue;
        };

        let Some(mut stock) = ctx.db.stock().id().find(tx.stock_id) else {
            tx.status = TransactionStatus::Rejected;
            ctx.db.transaction().id().update(tx);
            continue;
        };

        let Some(total_price) = stock.price_per_share.checked_mul(tx.amount.into()) else {
            tx.status = TransactionStatus::Rejected;
            ctx.db.transaction().id().update(tx);
            continue;
        };

        match tx.tx_type {
            TransactionType::Buy => {
                let fee = (total_price * player.stock_buy_fee as u128) / PRICE_SCALE_FACTOR;
                let total_cost = total_price + fee;

                if player.money < total_cost {
                    tx.status = TransactionStatus::Rejected;
                    ctx.db.transaction().id().update(tx);
                    continue;
                }
                
                if stock.available_shares < tx.amount {
                    tx.status = TransactionStatus::Rejected;
                    ctx.db.transaction().id().update(tx);
                    continue;
                }

                player.money -= total_cost;
                stock.available_shares -= tx.amount;
                stock.recent_buys += tx.amount;

                match player.stocks.iter_mut().find(|s| s.stock_id == tx.stock_id) {
                    Some(existing) => existing.amount += tx.amount,
                    None => player.stocks.push(StockType {
                        stock_id: tx.stock_id,
                        amount: tx.amount,
                    }),
                }
            }

            TransactionType::Sell => {
                let Some(existing) = player.stocks.iter_mut().find(|s| s.stock_id == tx.stock_id)
                else {
                    tx.status = TransactionStatus::Rejected;
                    ctx.db.transaction().id().update(tx);
                    continue;
                };

                if existing.amount < tx.amount {
                    tx.status = TransactionStatus::Rejected;
                    ctx.db.transaction().id().update(tx);
                    continue;
                }

                let fee = (total_price * player.stock_sell_fee as u128) / PRICE_SCALE_FACTOR;
                let proceeds = total_price.saturating_sub(fee);

                existing.amount -= tx.amount;
                player.money += proceeds;
                stock.available_shares += tx.amount;
                stock.recent_sells += tx.amount;

                if existing.amount == 0 {
                    player.stocks.retain(|s| s.stock_id != tx.stock_id);
                }
            }
        }

        ctx.db.player().identity().update(player);
        ctx.db.stock().id().update(stock);
        tx.status = TransactionStatus::Confirmed;
        ctx.db.transaction().id().update(tx);
    }

    Ok(())
}
