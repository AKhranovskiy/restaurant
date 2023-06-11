use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, put},
    Json, Router,
};
use serde_json::json;

use crate::{
    entities::{MealId, Order, OrderId, TableId},
    meals_catalog::MEALS,
    storage::Storage,
};

type StorageState = Arc<dyn Storage + Send + Sync>;

pub(crate) fn app(state: StorageState) -> Router {
    Router::new()
        .route("/table/:table/meal/:meal", put(put_order))
        .route("/order/:order", get(get_order).delete(delete_order))
        .route("/table/:table/orders", get(get_orders_for_table))
        .with_state(state)
}

async fn put_order(
    State(storage): State<StorageState>,
    Path((table_id, meal_id)): Path<(TableId, MealId)>,
) -> impl IntoResponse {
    log::info!("Server put_order({table_id}, {meal_id}");

    if let Some(meal) = MEALS.get_meal(meal_id) {
        match storage.add_order(Order::new(table_id, meal)).await {
            Ok(order) => (StatusCode::OK, Json(json!({ "order": order }))),
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

async fn get_order(
    State(storage): State<StorageState>,
    Path(order_id): Path<OrderId>,
) -> impl IntoResponse {
    log::info!("get_order({order_id}");

    match storage.get_order(order_id).await {
        Ok(Some(order)) => (StatusCode::OK, Json(json!({ "order": order }))),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(json!({"error": "Order not found"})),
        ),
        Err(error) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": "Storage failure", "details": error.to_string()})),
        ),
    }
}

async fn get_orders_for_table(
    State(storage): State<StorageState>,
    Path(table_id): Path<TableId>,
) -> impl IntoResponse {
    match storage.get_orders_for_table(table_id).await {
        Ok(orders) => (StatusCode::OK, Json(json!({ "orders": orders }))),
        Err(error) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": format!("Storage failure: {error:#}") })),
        ),
    }
}

async fn delete_order(
    State(storage): State<StorageState>,
    Path(order_id): Path<OrderId>,
) -> impl IntoResponse {
    match storage.delete_order(order_id).await {
        Ok(true) => (StatusCode::OK, Json(json!({}))),
        Ok(false) => (
            StatusCode::NOT_FOUND,
            Json(json!({"error": "Order not found"})),
        ),
        Err(error) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": "Storage failure", "details": error.to_string()})),
        ),
    }
}

#[cfg(test)]
mod tests {
    use axum::{body::Body, http::Request, Router};
    use serde::Deserialize;
    use tower::{Service, ServiceExt};

    use crate::{
        entities::{MealId, Order, TableId},
        storage::create_storage,
    };

    use super::app;

    #[tokio::test]
    async fn test_put_order() {
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

        #[derive(Deserialize)]
        struct PutOrderResponse {
            order: Order,
        }

        let order: &Order = &serde_json::from_slice::<PutOrderResponse>(&body)
            .unwrap()
            .order;
        assert_eq!(1, order.id);
        assert_eq!(1, order.table_id);
        assert_eq!(3, order.meal_id);
    }

    #[tokio::test]
    async fn test_put_invalid_order() {
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
    async fn test_get_order() {
        let mut app = app(create_storage().await.unwrap());

        put_order(&mut app, 1, 1).await;
        put_order(&mut app, 2, 2).await;

        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/order/2")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert!(response.status().is_success());

        #[derive(Deserialize)]
        struct GetOrderResponse {
            order: Order,
        }
        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let order = &serde_json::from_slice::<GetOrderResponse>(&body)
            .unwrap()
            .order;
        assert_eq!(2, order.id);
    }

    #[tokio::test]
    async fn test_delete_order() {
        let mut app = app(create_storage().await.unwrap());

        put_order(&mut app, 1, 1).await;
        put_order(&mut app, 2, 2).await;

        let response = app
            .oneshot(
                Request::builder()
                    .method("DELETE")
                    .uri("/order/2")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert!(response.status().is_success());
    }

    #[tokio::test]
    async fn test_delete_nonexisting_order() {
        let mut app = app(create_storage().await.unwrap());

        put_order(&mut app, 1, 1).await;
        put_order(&mut app, 2, 2).await;

        let response = app
            .oneshot(
                Request::builder()
                    .method("DELETE")
                    .uri("/order/3")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert!(response.status().is_client_error());
    }

    #[tokio::test]
    async fn test_get_invalid_order() {
        let mut app = app(create_storage().await.unwrap());

        put_order(&mut app, 1, 1).await;

        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/order/2")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert!(response.status().is_client_error());
    }

    #[tokio::test]
    async fn test_get_orders_for_table() {
        let mut app = app(create_storage().await.unwrap());

        put_order(&mut app, 1, 1).await;
        put_order(&mut app, 1, 1).await;
        put_order(&mut app, 1, 2).await;
        put_order(&mut app, 1, 2).await;
        put_order(&mut app, 1, 3).await;
        put_order(&mut app, 2, 1).await;
        put_order(&mut app, 2, 2).await;
        put_order(&mut app, 2, 3).await;

        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/table/1/orders")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert!(response.status().is_success());

        #[derive(Deserialize)]
        struct GetOrdersResponse {
            orders: Vec<Order>,
        }

        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let orders = serde_json::from_slice::<GetOrdersResponse>(&body)
            .unwrap()
            .orders;

        assert_eq!(5, orders.len());
        assert!(orders.iter().all(|order| { order.table_id == 1 }));

        assert_eq!(
            [1, 1, 2, 2, 3],
            orders
                .iter()
                .map(|order| order.meal_id)
                .collect::<Vec<_>>()
                .as_slice()
        );
    }

    async fn put_order(app: &mut Router, table_id: TableId, meal_id: MealId) {
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
