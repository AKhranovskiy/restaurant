use storage::create_storage;

pub mod api;
mod app;
mod meals_catalog;
mod storage;

pub async fn run_service() -> anyhow::Result<()> {
    axum::Server::bind(&"0.0.0.0:9000".parse().unwrap())
        .serve(app::app(create_storage().await?).into_make_service())
        .await
        .unwrap();

    Ok(())
}

pub fn init_logger() -> anyhow::Result<()> {
    simplelog::TermLogger::init(
        log::LevelFilter::Info,
        simplelog::ConfigBuilder::new()
            .add_filter_allow_str("restaurant")
            .add_filter_allow_str("clients")
            .build(),
        simplelog::TerminalMode::Mixed,
        simplelog::ColorChoice::Auto,
    )
    .map_err(Into::into)
}
