use actix_files::NamedFile;
use actix_web::{error, get, post, web, Responder};
use serde::{Deserialize, Serialize};

use crate::state::AppState;

#[get("/")]
pub async fn handle_index(state: web::Data<AppState>) -> impl Responder {
    return NamedFile::open_async(state.base_dir.as_path().join("index.html")).await;
}
#[derive(Deserialize)]
pub struct QueryRequest {
    pub search: String,
}

#[derive(Serialize)]
pub struct QueryResponse {
    pub response: String,
}
#[post("/query")]
pub async fn handle_query(
    state: web::Data<AppState>,
    req: web::Json<QueryRequest>,
) -> actix_web::Result<web::Json<QueryResponse>> {
    let query_str = &req.search;
    if query_str.is_empty() {
        return Err(error::ErrorBadRequest(
            "Empty strings are not accepted".to_string(),
        ));
    }
    match state.workers.run_query(query_str).await {
        Ok(resp) => Ok(actix_web::web::Json(QueryResponse { response: resp })),
        Err(e) => Err(error::ErrorBadRequest(e.to_string())),
    }
}
