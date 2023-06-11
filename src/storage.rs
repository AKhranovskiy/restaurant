use std::sync::Arc;

use axum::async_trait;

use crate::entities::{Meal, MealId, Table, TableId};

#[async_trait]
pub(crate) trait Storage: Clone {
    async fn add_meal(&self, table_id: TableId, meal_id: MealId) -> anyhow::Result<Table>;
    async fn get_meal(&self, table_id: TableId, meal_id: MealId) -> anyhow::Result<Vec<Meal>>;
    async fn get_table(&self, table_id: TableId) -> anyhow::Result<Table>;
    async fn delete_meal(&self, table_id: TableId, meal_id: MealId) -> anyhow::Result<Table>;
}

#[allow(dead_code)]
pub(crate) async fn create_storage() -> anyhow::Result<Box<impl Storage>> {
    InMemorySQLiteStorage::create().await.map(Box::new)
}

#[allow(dead_code)]
#[derive(Clone)]
struct InMemorySQLiteStorage {
    pool: Arc<sqlx::SqlitePool>,
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
                completed_at NUMERIC \
            ); \
            CREATE INDEX IF NOT EXISTS table_id_idx ON orders(table_id); \
            CREATE INDEX IF NOT EXISTS table_id_meal_id_idx ON orders(table_id, meal_id);",
        )
        .execute(&mut conn)
        .await?;

        Ok(Self {
            pool: Arc::new(pool),
        })
    }

    async fn create() -> anyhow::Result<Self> {
        let pool = sqlx::sqlite::SqlitePool::connect(":memory:").await?;
        Self::init(pool).await
    }
}

#[async_trait]
impl Storage for InMemorySQLiteStorage {
    async fn add_meal(&self, table_id: TableId, meal_id: MealId) -> anyhow::Result<Table> {
        log::debug!("Storage::add_meal({table_id}, {meal_id})");

        let meal = Meal::new(meal_id);

        let mut conn = self.pool.acquire().await?;

        sqlx::query(
            "INSERT INTO orders (table_id, meal_id, added_at, ready_at) VALUES (?, ?, ?, ?)",
        )
        .bind(table_id)
        .bind(meal_id)
        .bind(meal.added_at)
        .bind(meal.ready_at)
        .execute(&mut conn)
        .await?;

        self.get_table(table_id).await
    }

    async fn get_meal(&self, table_id: TableId, meal_id: MealId) -> anyhow::Result<Vec<Meal>> {
        log::debug!("Storage::get_meal({table_id}, {meal_id})");
        todo!()
    }

    async fn delete_meal(&self, table_id: TableId, meal_id: MealId) -> anyhow::Result<Table> {
        log::debug!("Storage::delete_meal({table_id}, {meal_id})");
        todo!()
    }

    async fn get_table(&self, table_id: TableId) -> anyhow::Result<Table> {
        log::debug!("Storage::get_table({table_id})");

        let mut conn = self.pool.acquire().await?;

        let meals = sqlx::query_as::<_, Meal>(
            "SELECT * FROM orders WHERE table_id = ? AND completed_at IS NULL",
        )
        .bind(table_id)
        .fetch_all(&mut conn)
        .await?;

        Ok(Table::new(table_id).add_meals(meals))
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;

    impl From<sqlx::SqlitePool> for InMemorySQLiteStorage {
        fn from(pool: sqlx::SqlitePool) -> Self {
            Self {
                pool: Arc::new(pool),
            }
        }
    }

    #[sqlx::test]
    async fn test_add_same_meals(pool: sqlx::SqlitePool) -> sqlx::Result<()> {
        let storage = InMemorySQLiteStorage::init(pool).await.unwrap();

        let table = storage.add_meal(2, 3).await.unwrap();
        assert_eq!(table, Table::new(2).add_meal(Meal::new(3)));

        let table_2 = storage.add_meal(2, 3).await.unwrap();
        assert_eq!(table_2, table.add_meal(Meal::new(3)));

        Ok(())
    }

    #[sqlx::test]
    async fn test_add_different_meals(pool: sqlx::SqlitePool) -> sqlx::Result<()> {
        let storage = InMemorySQLiteStorage::init(pool).await.unwrap();

        let table = storage.add_meal(2, 3).await.unwrap();
        assert_eq!(table, Table::new(2).add_meal(Meal::new(3)));

        let table_2 = storage.add_meal(2, 5).await.unwrap();
        assert_eq!(table_2, table.add_meal(Meal::new(5)));

        Ok(())
    }
}
