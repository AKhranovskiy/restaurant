use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use serde_json::json;

use crate::{
    entities::{MealId, Order, TableId},
    meals_catalog::MEALS,
    storage::Storage,
};

type StorageState = Arc<dyn Storage + Send + Sync>;

pub(crate) fn app(state: StorageState) -> Router {
    Router::new()
        // TODO client UI
        .route(
            "/table/:table/meal/:meal",
            get(get_meal_on_table)
                .put(add_meal_to_table)
                .delete(delete_meal_from_table),
        )
        .route("/table/:table", get(get_all_meals_on_table))
        .with_state(state)
}

async fn add_meal_to_table(
    State(storage): State<StorageState>,
    Path((table_id, meal_id)): Path<(TableId, MealId)>,
) -> impl IntoResponse {
    log::info!("Server add_meal_to_table({table_id}, {meal_id}");

    if let Some(meal) = MEALS.get_meal(meal_id) {
        let order = Order::new(table_id, meal);
        match storage.add_order(order).await {
            Ok(order_id) => (StatusCode::OK, Json(json!({ "order": order_id }))),
            Err(error) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Storage failure", "details": error.to_string()})),
            ),
        }
    } else {
        (
            StatusCode::BAD_REQUEST,
            Json(json! ({"error": "Invalid meal"})),
        )
    }
}

async fn get_meal_on_table(
    State(storage): State<StorageState>,
    Path((table_id, meal_id)): Path<(TableId, MealId)>,
) -> impl IntoResponse {
    log::info!("get_meal_on_table({table_id}, {meal_id}");

    if MEALS.get_meal(meal_id).is_some() {
        match storage.get_meal_orders_for_table(table_id, meal_id).await {
            Ok(orders) => (StatusCode::OK, Json(json!({ "orders": orders }))),
            Err(error) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Storage failure", "details": error.to_string()})),
            ),
        }
    } else {
        (
            StatusCode::BAD_REQUEST,
            Json(json! ({"error": "Invalid meal"})),
        )
    }
}

async fn get_all_meals_on_table(
    State(storage): State<StorageState>,
    Path(table_id): Path<TableId>,
) -> impl IntoResponse {
    match storage.get_orders_for_table(table_id).await {
        Ok(orders) => (StatusCode::OK, Json(json!({ "orders": orders }))),
        Err(error) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": "Storage failure", "details": error.to_string()})),
        ),
    }
}

async fn delete_meal_from_table(
    State(_storage): State<StorageState>,
    Path((table, meal)): Path<(TableId, MealId)>,
) -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(json! ({"table": table, "meal": meal, "total": 0_u32})),
    )
}

#[cfg(test)]
mod tests {
    use axum::{body::Body, http::Request, Router};
    use serde_json::Value;
    use tower::{Service, ServiceExt};

    use crate::{
        entities::{MealId, TableId},
        storage::create_storage,
    };

    use super::app;

    #[tokio::test]
    async fn test_add_meal_to_table() {
        let app = app(create_storage().await.unwrap());

        let response = app
            .oneshot(
                Request::builder()
                    .method("PUT")
                    .uri("/table/1/meal/3")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert!(response.status().is_success());

        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let value: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(1, value.get("order").unwrap().as_u64().unwrap());
    }

    #[tokio::test]
    async fn test_add_invalid_meal_to_table() {
        let app = app(create_storage().await.unwrap());

        let response = app
            .oneshot(
                Request::builder()
                    .method("PUT")
                    .uri("/table/1/meal/1234")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert!(response.status().is_client_error());
    }

    #[tokio::test]
    async fn test_get_meal_orders() {
        let mut app = app(create_storage().await.unwrap());

        add_meal(&mut app, 1, 1).await;
        add_meal(&mut app, 1, 1).await;
        add_meal(&mut app, 1, 2).await;
        add_meal(&mut app, 1, 2).await;
        add_meal(&mut app, 2, 1).await;
        add_meal(&mut app, 2, 2).await;

        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/table/1/meal/2")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert!(response.status().is_success());

        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let body = serde_json::from_slice::<Value>(&body).unwrap();
        let orders = body.get("orders").unwrap().as_array().unwrap();
        assert_eq!(2, orders.len());
        assert!(orders.iter().all(|order| {
            order.get("table_id").unwrap().as_u64().unwrap() == 1
                && order.get("meal_id").unwrap().as_u64().unwrap() == 2
        }));
    }

    #[tokio::test]
    async fn test_get_all_meals_on_table() {
        let mut app = app(create_storage().await.unwrap());

        add_meal(&mut app, 1, 1).await;
        add_meal(&mut app, 1, 1).await;
        add_meal(&mut app, 1, 2).await;
        add_meal(&mut app, 1, 2).await;
        add_meal(&mut app, 1, 3).await;
        add_meal(&mut app, 2, 1).await;
        add_meal(&mut app, 2, 2).await;
        add_meal(&mut app, 2, 3).await;

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/table/1")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert!(response.status().is_success());

        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let body = serde_json::from_slice::<Value>(&body).unwrap();
        let orders = body.get("orders").unwrap().as_array().unwrap();

        assert_eq!(5, orders.len());
        assert!(orders
            .iter()
            .all(|order| { order.get("table_id").unwrap().as_u64().unwrap() == 1 }));

        assert_eq!(
            [1_u64, 1, 2, 2, 3],
            orders
                .iter()
                .map(|order| order.get("meal_id").unwrap().as_u64().unwrap())
                .collect::<Vec<u64>>()
                .as_slice()
        );
    }

    async fn add_meal(app: &mut Router, table_id: TableId, meal_id: MealId) {
        let request = Request::builder()
            .method("PUT")
            .uri(format!("/table/{table_id}/meal/{meal_id}"))
            .body(Body::empty())
            .unwrap();
        ServiceExt::<Request<Body>>::ready(app)
            .await
            .unwrap()
            .call(request)
            .await
            .unwrap();
    }
}
