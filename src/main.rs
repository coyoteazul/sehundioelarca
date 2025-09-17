use std::{sync::{Arc}, time::Duration};

use axum::{extract::State, Json, Router};
use tokio::sync::RwLock;

use crate::arca::get_status::{check_status_job, Services};

mod arca;

#[tokio::main]
async fn main() {
    let status = Arc::new(RwLock::new(Services::new()));
		
		check_status_job(status.clone(), Duration::from_secs(300/*5 minutos*/));

		let app = Router::new()
        .route("/status", axum::routing::get(get_status_handler))
        .with_state(status.clone()); 


		let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn get_status_handler(State(status): State<Arc<RwLock<Services>>>) -> Json<Services> {
    let status_guard = status.read().await;
    Json(status_guard.clone())
}