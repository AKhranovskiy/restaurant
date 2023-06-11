mod app;
mod entities;
mod storage;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_logger()?;

    axum::Server::bind(&"0.0.0.0:9000".parse().unwrap())
        .serve(app::app().into_make_service())
        .await
        .unwrap();

    Ok(())
}

fn init_logger() -> anyhow::Result<()> {
    simplelog::TermLogger::init(
        log::LevelFilter::Debug,
        simplelog::Config::default(),
        simplelog::TerminalMode::Mixed,
        simplelog::ColorChoice::Auto,
    )
    .map_err(Into::into)
}
