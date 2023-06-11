use restaurant::init_logger;
use restaurant::run_service;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_logger()?;
    run_service().await
}
