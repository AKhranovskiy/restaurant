mod order;

pub use crate::meals_catalog::MealInfo;
pub use order::{MealId, Order, OrderId, TableId};

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct PutOrderResponse {
    pub order: Order,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetOrderResponse {
    pub order: Order,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetOrdersResponse {
    pub orders: Vec<Order>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MealsResponse {
    pub meals: Vec<MealInfo>,
}
