use std::time::Duration;

use crate::entities::MealId;

const MINUTE: Duration = Duration::from_secs(60);

struct MealInfo {
    id: MealId,
    name: &'static str,
    cooking_time: Duration,
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

pub(crate) const MEALS: &[MealInfo] = &[
    (1, "Green Tea", MINUTE).into(),
    (2, "Americano Coffee", 2 * MINUTE).into(),
    (3, "Omellete", 5 * MINUTE).into(),
    (4, "Fried Egg", 3 * MINUTE).into(),
    (5, "Club Sandwich", 6 * MINUTE).into(),
    (6, "Fried Rice", 4 * MINUTE).into(),
];
