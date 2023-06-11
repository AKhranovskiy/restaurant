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
    entities::{MealId, Table, TableId},
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

async fn get_meal_on_table(
    State(_storage): State<StorageState>,
    Path((table_id, meal_id)): Path<(TableId, MealId)>,
) -> impl IntoResponse {
    log::info!("get_meal_on_table({table_id}, {meal_id}");

    // let meal = MEALS
    //     .get_meal(meal_id)
    //     .ok_or(anyhow::anyhow!("Invalid meal"))?;

    (
        StatusCode::OK,
        Json(json! ({"table": table_id, "meal_d": meal_id, "count": 1_u32})),
    )
}

async fn get_all_meals_on_table(
    State(_storage): State<StorageState>,
    Path(table_id): Path<TableId>,
) -> impl IntoResponse {
    log::info!("get_all_meals_on_table({table_id})");
    (StatusCode::OK, Json(Table::new(table_id)))
}

async fn add_meal_to_table(
    State(_storage): State<StorageState>,
    Path((table, meal)): Path<(TableId, MealId)>,
) -> impl IntoResponse {
    log::info!("Server add_meal_to_table({table}, {meal}");

    (
        StatusCode::OK,
        Json(json! ({"table": table, "meal": meal, "total": 1_u32})),
    )
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
    use axum::{body::Body, http::Request};
    use tower::ServiceExt;

    use crate::{entities::Table, storage::create_storage};

    use super::app;

    #[tokio::test]
    async fn test_get_all_meals_on_table() {
        let app = app(create_storage().await.unwrap());

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
        let table: Table = serde_json::from_slice(&body).unwrap();
        assert_eq!(table, Table::new(1));
    }
}
