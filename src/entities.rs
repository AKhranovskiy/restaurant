use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::meals_catalog::MEALS;

pub(crate) type TableId = u32;
pub(crate) type MealId = u32;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct Table {
    pub(crate) table_id: TableId,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    meals: Vec<Meal>,
}

#[allow(dead_code)]
impl Table {
    pub(crate) fn new(id: TableId) -> Self {
        Self {
            table_id: id,
            meals: vec![],
        }
    }

    pub(crate) fn add_meal(self, meal: Meal) -> Self {
        let mut s = self;
        s.meals.push(meal);
        s
    }

    pub(crate) fn add_meals(self, mut meals: Vec<Meal>) -> Self {
        let mut s = self;
        s.meals.append(&mut meals);
        s
    }

    pub(crate) fn get_meal(&self, meal_id: MealId) -> Vec<Meal> {
        self.meals
            .iter()
            .filter(|m| m.meal_id == meal_id)
            .cloned()
            .collect()
    }
}

#[derive(Debug, Serialize, Deserialize, Eq, Clone, sqlx::FromRow)]
pub(crate) struct Meal {
    pub(crate) meal_id: MealId,
    pub(crate) added_at: DateTime<Utc>,
    pub(crate) ready_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub(crate) completed_at: Option<DateTime<Utc>>,
}

#[allow(dead_code)]
impl Meal {
    pub(crate) fn new(id: MealId) -> Self {
        let meal = MEALS.get_meal(id).unwrap();
        let now = Utc::now();
        Self {
            meal_id: id,
            added_at: now,
            ready_at: now + meal.cooking_time,
            completed_at: None,
        }
    }

    pub(crate) fn name(&self) -> &'static str {
        MEALS.get_meal(self.meal_id).unwrap().name
    }
}

impl PartialEq for Meal {
    fn eq(&self, other: &Self) -> bool {
        self.meal_id == other.meal_id
    }
}

#[cfg(test)]
mod tests {
    use crate::meals_catalog::MEALS;

    use super::Meal;

    #[test]
    fn test_new_meal() {
        let sut = Meal::new(1);
        let meal = MEALS.get_meal(1).unwrap();

        assert_eq!(1, sut.meal_id);
        assert_eq!(meal.name, sut.name());
        assert_eq!(meal.cooking_time, sut.ready_at - sut.added_at);
    }
}
