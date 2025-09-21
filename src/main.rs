use std::sync::{Arc};

use axum::{extract::State, Json, Router};
use tokio::sync::RwLock;
use tower_http::cors::CorsLayer;

use crate::{arca::{check_status_job, Servicios}, front::{get_css, get_front}};

mod arca;
mod front;

#[tokio::main]
async fn main() {
    
		let status = Arc::new(RwLock::new(AppState {
			status: Servicios::new(),
			front : vec![],
		}));
		
		check_status_job(status.clone());

		let cors_permissive = CorsLayer::new()
				.allow_origin(tower_http::cors::Any) // Allow any origin (wildcard *)
				.allow_methods(tower_http::cors::Any) // Allow any method
				.allow_headers(tower_http::cors::Any	); // Allow any header


		let app = Router::new()
        .route("/status"    , axum::routing::get(get_status_handler))
				.route("/"          , axum::routing::get(get_front))
				.route("/style2.css", axum::routing::get(get_css))
				.layer(cors_permissive)
        .with_state(status.clone()); 


		let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn get_status_handler(State(status): State<Arc<RwLock<AppState>>>) -> Json<Servicios> {
    let status_guard = status.read().await;
    Json(status_guard.status.clone())
}





pub struct AppState {
	pub status: Servicios,
	pub front : Vec<u8>,
}