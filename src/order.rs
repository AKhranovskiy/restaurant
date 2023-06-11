use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::meals_catalog::MealInfo;

pub(crate) type TableId = u32;
pub(crate) type OrderId = u32;
pub(crate) type MealId = u32;

#[derive(Debug, Serialize, Deserialize, Eq, Clone, sqlx::FromRow)]
pub(crate) struct Order {
    pub(crate) id: OrderId,
    pub(crate) table_id: TableId,
    pub(crate) meal_id: MealId,
    pub(crate) added_at: DateTime<Utc>,
    pub(crate) ready_at: DateTime<Utc>,
}

impl Order {
    pub(crate) fn new(table_id: TableId, meal: &MealInfo) -> Self {
        let now = Utc::now();
        Self {
            id: OrderId::MAX,
            table_id,
            meal_id: meal.id,
            added_at: now,
            ready_at: now + meal.cooking_time,
        }
    }
}

impl PartialEq for Order {
    fn eq(&self, other: &Self) -> bool {
        self.table_id == other.table_id && self.meal_id == other.meal_id
    }
}

#[cfg(test)]
mod tests {
    use crate::meals_catalog::MEALS;

    use super::*;

    #[test]
    fn test_new_order() {
        let meal = MEALS.get(2).unwrap();
        let order = Order::new(1, meal);

        assert_eq!(1, order.table_id);
        assert_eq!(2, order.meal_id);
        assert_eq!(meal.cooking_time, order.ready_at - order.added_at);
    }
}
