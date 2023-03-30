use std::{path::PathBuf, sync::Arc};

use crate::worker::WorkerManager;

#[derive(Clone)]
pub struct AppState {
    pub workers: Arc<WorkerManager>,
    pub base_dir: PathBuf,
}
