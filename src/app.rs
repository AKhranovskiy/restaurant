use axum::{extract::Path, http::StatusCode, response::IntoResponse, routing::get, Json, Router};
use serde_json::json;

use crate::entities::{Meal, MealId, Table, TableId};

pub(crate) fn app() -> Router {
    Router::new()
        // TODO client UI
        .route(
            "/table/:table/meal/:meal",
            get(get_meal_on_table)
                .put(add_meal_to_table)
                .delete(delete_meal_from_table),
        )
        .route("/table/:table", get(get_all_meals_on_table))
}

async fn get_meal_on_table(
    Path((table_id, meal_id)): Path<(TableId, MealId)>,
) -> impl IntoResponse {
    log::info!("get_meal_on_table({table_id}, {meal_id}");

    let meal = Meal::new(meal_id);

    (
        StatusCode::OK,
        Json(json! ({"table": table_id, "meal": meal, "count": 1_u32})),
    )
}

async fn get_all_meals_on_table(Path(table_id): Path<TableId>) -> impl IntoResponse {
    log::info!("get_all_meals_on_table({table_id})");
    (StatusCode::OK, Json(Table::new(table_id)))
}

async fn add_meal_to_table(Path((table, meal)): Path<(TableId, MealId)>) -> impl IntoResponse {
    log::info!("Server add_meal_to_table({table}, {meal}");

    (
        StatusCode::OK,
        Json(json! ({"table": table, "meal": meal, "total": 1_u32})),
    )
}

async fn delete_meal_from_table(Path((table, meal)): Path<(TableId, MealId)>) -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(json! ({"table": table, "meal": meal, "total": 0_u32})),
    )
}

#[cfg(test)]
mod tests {
    use axum::{body::Body, http::Request};
    use tower::ServiceExt;

    use crate::entities::Table;

    use super::app;

    #[tokio::test]
    async fn test_get_all_meals_on_table() {
        let app = app();

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
