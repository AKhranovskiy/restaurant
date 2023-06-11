use std::{collections::VecDeque, sync::Arc};

use rand::{seq::SliceRandom, Rng};
use tokio::sync::{Barrier, Mutex};

use restaurant::{
    api::{GetOrdersResponse, MealInfo, TableId},
    init_logger,
};

const TABLES: usize = 200;
const WAITERS: usize = 50;
const ITERATIONS: usize = 10_000;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_logger()?;

    log::info!("Start business. {TABLES} tables, {WAITERS} waiters, {ITERATIONS} iterations");

    let client = reqwest::Client::new();

    log::info!("Getting meals catalog");
    let catalog = MealsCatalog {
        meals: client
            .get("http://localhost:9000/meals")
            .send()
            .await?
            .json()
            .await?,
    };

    let tables = (0..TABLES)
        .map(|id| Table::new(id as _))
        .collect::<VecDeque<_>>();
    let tables = Arc::new(Mutex::new(tables));

    let barrier = Arc::new(Barrier::new(WAITERS));

    let tasks = (0..WAITERS)
        .map(|id| {
            let tables = tables.clone();
            let waiter = Waiter::new(id as _, catalog.clone(), client.clone());
            let c = barrier.clone();

            tokio::spawn(async move {
                let _ = c.wait().await;
                log::info!("Waiter {} starts", waiter.id);
                for _ in 0..ITERATIONS {
                    let table = tables.lock().await.pop_front().unwrap().advance().await;
                    waiter.serve(&table).await?;
                    tables.lock().await.push_back(table);
                }
                anyhow::Ok(())
            })
        })
        .collect::<Vec<_>>();

    for task in tasks {
        _ = task.await?;
    }

    Ok(())
}

#[derive(Debug)]
struct Table {
    id: TableId,
    state: TableState,
}

#[derive(Debug)]
enum TableState {
    Empty,
    Ordering,
    Eating,
    Complete,
}

impl Table {
    fn new(id: u32) -> Self {
        Self {
            id,
            state: TableState::Empty,
        }
    }

    async fn advance(self) -> Table {
        let state = match self.state {
            TableState::Empty => match rand::thread_rng().gen_bool(0.3) {
                true => TableState::Ordering,
                false => TableState::Empty,
            },
            TableState::Ordering => match rand::thread_rng().gen_bool(0.5) {
                true => TableState::Ordering,
                false => TableState::Eating,
            },
            TableState::Eating => match rand::thread_rng().gen_bool(0.3) {
                true => TableState::Ordering,
                false => match rand::thread_rng().gen_bool(0.6) {
                    true => TableState::Eating,
                    false => TableState::Complete,
                },
            },
            TableState::Complete => TableState::Empty,
        };
        Table { state, ..self }
    }
}

struct Waiter {
    id: u32,
    catalog: MealsCatalog,
    client: reqwest::Client,
}
impl Waiter {
    fn new(id: u32, catalog: MealsCatalog, client: reqwest::Client) -> Self {
        Self {
            id,
            catalog,
            client,
        }
    }

    async fn serve(&self, table: &Table) -> anyhow::Result<()> {
        match table.state {
            TableState::Empty | TableState::Eating => {}
            TableState::Ordering => {
                let meal = self.catalog.random();
                log::info!(
                    "Waiter {} is taking order {}:{} from table {}",
                    self.id,
                    meal.name,
                    meal.id,
                    table.id,
                );

                self.client
                    .put(format!(
                        "http://localhost:9000/table/{}/meal/{}",
                        table.id, meal.id
                    ))
                    .send()
                    .await?;
            }
            TableState::Complete => {
                let orders: GetOrdersResponse = self
                    .client
                    .get(format!("http://localhost:9000/table/{}/orders", table.id))
                    .send()
                    .await?
                    .json()
                    .await?;

                log::info!(
                    "Waiter {} cleans {} orders for table {}",
                    self.id,
                    orders.orders.len(),
                    table.id
                );

                for order in orders.orders {
                    self.client
                        .delete(format!("http://localhost:9000/order/{}", order.id))
                        .send()
                        .await?;
                }
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
struct MealsCatalog {
    meals: Vec<MealInfo>,
}

impl MealsCatalog {
    fn random(&self) -> &MealInfo {
        self.meals.choose(&mut rand::thread_rng()).unwrap()
    }
}
