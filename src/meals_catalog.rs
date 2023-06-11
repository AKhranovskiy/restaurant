use chrono::Duration;
use lazy_static::lazy_static;

use crate::entities::MealId;

#[derive(Clone, Debug)]
pub(crate) struct MealInfo {
    pub(crate) id: MealId,
    pub(crate) name: &'static str,
    pub(crate) cooking_time: Duration,
}

impl From<(MealId, &'static str, Duration)> for MealInfo {
    fn from((id, name, cooking_time): (MealId, &'static str, Duration)) -> Self {
        Self {
            id,
            name,
            cooking_time,
        }
    }
}

#[derive(Default)]
pub(crate) struct MealCatalog {
    meals: Vec<MealInfo>,
}

#[allow(dead_code)]
impl MealCatalog {
    fn add_meal(mut self, meal: MealInfo) -> Self {
        self.meals.push(meal);
        self
    }

    pub(crate) fn get_all_meals(&self) -> &[MealInfo] {
        self.meals.as_slice()
    }

    pub(crate) fn get_meal(&self, meal_id: MealId) -> Option<&MealInfo> {
        self.meals.iter().find(|m| m.id == meal_id)
    }
}

lazy_static! {
    pub(crate) static ref MEALS: MealCatalog = MealCatalog::default()
        .add_meal((0, "Green Tea", Duration::minutes(1)).into())
        .add_meal((1, "Americano Coffee", Duration::minutes(2)).into())
        .add_meal((2, "Omellete", Duration::minutes(3)).into())
        .add_meal((3, "Fried Egg", Duration::minutes(4)).into())
        .add_meal((4, "Club Sandwich", Duration::minutes(5)).into())
        .add_meal((5, "Fried Rice", Duration::minutes(6)).into());
}
