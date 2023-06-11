use axum::{
    extract::Path, http::StatusCode, response::IntoResponse, routing::get, Json, Router, Server,
};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;

fn init_logger() -> anyhow::Result<()> {
    simplelog::TermLogger::init(
        log::LevelFilter::Debug,
        simplelog::Config::default(),
        simplelog::TerminalMode::Mixed,
        simplelog::ColorChoice::Auto,
    )
    .map_err(Into::into)
}

fn app() -> Router {
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

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_logger()?;

    Server::bind(&"0.0.0.0:9000".parse().unwrap())
        .serve(app().into_make_service())
        .await
        .unwrap();

    Ok(())
}

type TableId = u32;
type MealId = u32;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
struct Table {
    id: TableId,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    meals: Vec<Meal>,
}

#[derive(Debug, Serialize, Deserialize, Eq)]
struct Meal {
    id: MealId,
    name: String,
    created_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    ready_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    completed_at: Option<DateTime<Utc>>,
}

impl PartialEq for Meal {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

async fn get_meal_on_table(
    Path((table_id, meal_id)): Path<(TableId, MealId)>,
) -> impl IntoResponse {
    log::info!("get_meal_on_table({table_id}, {meal_id}");

    let meal = Meal {
        id: meal_id,
        name: "Meal".to_string(),
        created_at: Utc::now() - Duration::minutes(1),
        ready_at: None,
        completed_at: None,
    };

    (
        StatusCode::OK,
        Json(json! ({"table": table_id, "meal": meal, "count": 1_u32})),
    )
}

async fn get_all_meals_on_table(Path(table): Path<TableId>) -> impl IntoResponse {
    log::info!("get_all_meals_on_table({table})");
    (
        StatusCode::OK,
        Json(Table {
            id: table,
            meals: vec![],
        }),
    )
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

    use crate::Table;

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
        assert_eq!(
            table,
            Table {
                id: 1,
                meals: vec![]
            }
        );
    }
}
