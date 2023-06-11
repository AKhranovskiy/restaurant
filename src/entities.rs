use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

pub(crate) type TableId = u32;
pub(crate) type MealId = u32;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct Table {
    pub(crate) id: TableId,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub(crate) meals: Vec<Meal>,
}

#[derive(Debug, Serialize, Deserialize, Eq)]
pub(crate) struct Meal {
    pub(crate) id: MealId,
    pub(crate) name: String,
    pub(crate) created_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub(crate) ready_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub(crate) completed_at: Option<DateTime<Utc>>,
}

impl PartialEq for Meal {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
