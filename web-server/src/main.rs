use std::path::Path;
use std::sync::Arc;

use actix_web::web::Data;
use actix_web::App;
use actix_web::HttpServer;
use anyhow::anyhow;
use anyhow::Result;

use flexi_logger::Logger;
use log::info;
use route::handle_index;
use state::AppState;
use util::my_log_format;
use worker::WorkerManager;

use crate::route::handle_query;

pub mod route;
pub mod state;
pub mod util;
#[allow(clippy::await_holding_lock)]
pub mod worker;
async fn write_default(base_path: &Path, file_name: &str, bytes: &[u8]) -> Result<()> {
    let path = base_path.join(file_name);
    if !path.exists() {
        info!("Writing: {}", file_name);
        tokio::fs::write(path, bytes).await?;
    }
    Ok(())
}

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<()> {
    let log_level = option_env!("EKB_LOG_LEVEL").unwrap_or("info");
    println!(
        "Log level: `{}`. Use EKB_LOG_LEVEL to customize it.",
        log_level
    );
    Logger::try_with_str(log_level)
        .unwrap()
        .format(my_log_format)
        .log_to_stdout()
        .start()
        .map_err(|e| anyhow!("Failed to start logger!\n{}", e))?;

    let base_dir = std::env::current_dir()?;
    write_default(&base_dir, "index.html", include_bytes!("index.html")).await?;
    let worker_count = option_env!("EKB_WORKER_COUNT")
        .unwrap_or("2")
        .parse::<usize>()?;
    info!(
        "Worker count: `{}` Use env var `EKB_WORKER_COUNT` to customize this",
        worker_count
    );
    let python_executable = option_env!("EKB_PYTHON_EXECUTABLE").unwrap_or("python");
    info!(
        "Python executable: `{}` Use env var `EKB_PYTHON_EXECUTABLE` to customize this",
        python_executable
    );

    let worker_py = option_env!("EKB_WORKER_SCRIPT").unwrap_or("worker.py");
    info!(
        "Worker script: `{}` Use env var `EKB_WORKER_SCRIPT` to customize this",
        worker_py
    );

    let mut worker_manager = WorkerManager::new(worker_count, python_executable, worker_py)?;
    info!("Starting workers..");
    worker_manager.wait_for_all_start().await?;
    info!("Workers started");

    let app_data = Data::new(AppState {
        workers: Arc::new(worker_manager),
        base_dir,
    });
    HttpServer::new(move || {
        App::new()
            .app_data(app_data.clone())
            .wrap(actix_web::middleware::Logger::new(
                r#"%a,%{r}a "%r" %s %b %T"#,
            ))
            .service(handle_index)
            .service(handle_query)
    })
    .bind(("0.0.0.0".to_string(), 4100))?
    .run()
    .await?;
    Ok(())
}
