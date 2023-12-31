use std::sync::Arc;

use axum::async_trait;
use chrono::Utc;

use crate::api::{Order, OrderId, TableId};

#[async_trait]
pub(crate) trait Storage {
    async fn add_order(&self, order: Order) -> anyhow::Result<Order>;
    async fn get_order(&self, order_id: OrderId) -> anyhow::Result<Option<Order>>;
    async fn delete_order(&self, order_id: OrderId) -> anyhow::Result<bool>;
    async fn get_orders_for_table(&self, table_id: TableId) -> anyhow::Result<Vec<Order>>;
}

pub(crate) async fn create_storage() -> anyhow::Result<Arc<dyn Storage + Send + Sync>> {
    let storage = InMemorySQLiteStorage::create().await?;
    Ok(Arc::new(storage))
}

#[derive(Clone)]
struct InMemorySQLiteStorage {
    pool: sqlx::SqlitePool,
}

impl InMemorySQLiteStorage {
    async fn init(pool: sqlx::SqlitePool) -> anyhow::Result<Self> {
        let mut conn = pool.acquire().await?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS orders (\
                id INTEGER PRIMARY KEY, \
                table_id INTEGER NOT NULL, \
                meal_id INTEGER NOT NULL, \
                added_at NUMERIC NOT NULL, \
                ready_at NUMERIC NOT NULL, \
                deleted_at NUMERIC \
            ); \
            CREATE INDEX IF NOT EXISTS order_id_idx ON orders(id, deleted_at); \
            CREATE INDEX IF NOT EXISTS table_id_idx ON orders(table_id, deleted_at);
            ",
        )
        .execute(&mut conn)
        .await?;

        Ok(Self { pool })
    }

    async fn create() -> anyhow::Result<Self> {
        let pool = sqlx::sqlite::SqlitePool::connect(":memory:").await?;
        Self::init(pool).await
    }
}

#[async_trait]
impl Storage for InMemorySQLiteStorage {
    async fn add_order(&self, order: Order) -> anyhow::Result<Order> {
        log::debug!("Storage::add_order(order:?)");

        let mut conn = self.pool.acquire().await?;

        sqlx::query_as::<_, Order>(
            "INSERT INTO orders (table_id, meal_id, added_at, ready_at) VALUES (?, ?, ?, ?) RETURNING *",
        )
        .bind(order.table_id)
        .bind(order.meal_id)
        .bind(order.added_at)
        .bind(order.ready_at)
        .fetch_one(&mut conn)
        .await
        .map_err(Into::into)
    }

    async fn get_order(&self, order_id: OrderId) -> anyhow::Result<Option<Order>> {
        log::debug!("Storage::get_order({order_id})");

        let mut conn = self.pool.acquire().await?;

        sqlx::query_as::<_, Order>("SELECT * FROM orders where id = ? and deleted_at IS NULL")
            .bind(order_id)
            .fetch_optional(&mut conn)
            .await
            .map_err(Into::into)
    }

    async fn delete_order(&self, order_id: OrderId) -> anyhow::Result<bool> {
        log::debug!("Storage::delete_order({order_id})");

        let mut conn = self.pool.acquire().await?;

        sqlx::query("UPDATE orders SET deleted_at = ? WHERE id = ?")
            .bind(Utc::now())
            .bind(order_id)
            .execute(&mut conn)
            .await
            .map_err(Into::into)
            .map(|result| result.rows_affected() == 1)
    }

    async fn get_orders_for_table(&self, table_id: TableId) -> anyhow::Result<Vec<Order>> {
        log::debug!("Storage::get_orders_for_table({table_id})");

        let mut conn = self.pool.acquire().await?;

        sqlx::query_as::<_, Order>("SELECT * FROM orders WHERE table_id = ? AND deleted_at IS NULL")
            .bind(table_id)
            .fetch_all(&mut conn)
            .await
            .map_err(Into::into)
    }
}

#[cfg(test)]
mod tests {
    use crate::meals_catalog::MEALS;

    use super::*;

    #[sqlx::test]
    async fn test_add_order(pool: sqlx::SqlitePool) -> sqlx::Result<()> {
        let storage = InMemorySQLiteStorage::init(pool).await.unwrap();

        let meal = MEALS.get(3).unwrap();

        let order_id = storage.add_order(Order::new(2, meal)).await.unwrap().id;
        let order_id_2 = storage.add_order(Order::new(2, meal)).await.unwrap().id;
        let order_id_3 = storage.add_order(Order::new(1, meal)).await.unwrap().id;

        assert_ne!(order_id, order_id_2);
        assert_ne!(order_id_2, order_id_3);

        Ok(())
    }

    #[sqlx::test]
    async fn test_get_order(pool: sqlx::SqlitePool) -> sqlx::Result<()> {
        let storage = InMemorySQLiteStorage::init(pool).await.unwrap();

        assert!(storage.get_order(1).await.unwrap().is_none());

        let meal = MEALS.get(3).unwrap();
        let order_id = storage.add_order(Order::new(2, meal)).await.unwrap().id;
        let order = storage.get_order(order_id).await.unwrap().unwrap();

        assert_eq!(order, Order::new(2, meal));

        Ok(())
    }

    #[sqlx::test]
    async fn test_delete_order(pool: sqlx::SqlitePool) -> sqlx::Result<()> {
        let storage = InMemorySQLiteStorage::init(pool).await.unwrap();

        // Delete non-existing order.
        storage.delete_order(1).await.unwrap();

        let meal = MEALS.get(3).unwrap();
        let order_id = storage.add_order(Order::new(2, meal)).await.unwrap().id;
        storage.delete_order(order_id).await.unwrap();

        assert!(storage.get_order(order_id).await.unwrap().is_none());

        Ok(())
    }

    #[sqlx::test]
    async fn test_get_orders_for_table(pool: sqlx::SqlitePool) -> sqlx::Result<()> {
        let storage = InMemorySQLiteStorage::init(pool).await.unwrap();

        assert!(storage.get_orders_for_table(1).await.unwrap().is_empty());

        storage
            .add_order(Order::new(1, MEALS.get(3).unwrap()))
            .await
            .unwrap();
        storage
            .add_order(Order::new(1, MEALS.get(3).unwrap()))
            .await
            .unwrap();
        storage
            .add_order(Order::new(1, MEALS.get(4).unwrap()))
            .await
            .unwrap();
        storage
            .add_order(Order::new(2, MEALS.get(3).unwrap()))
            .await
            .unwrap();

        let orders = storage.get_orders_for_table(1).await.unwrap();
        assert_eq!(3, orders.len());
        assert!(orders.iter().all(|order| order.table_id == 1));

        Ok(())
    }
}
