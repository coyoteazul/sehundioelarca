use std::{sync::{Arc}, time::Duration};

use axum::{extract::State, response::Html, Json, Router};
use tokio::sync::RwLock;
use tower_http::cors::CorsLayer;

use crate::arca::{check_status_job, ServiceStatus};



mod arca;
mod front;

#[tokio::main]
async fn main() {
    
		let status = Arc::new(RwLock::new(AppState {
			status: vec![],
			front : String::new(),
		}));
		
		check_status_job(status.clone(), Duration::from_secs(60/*1 minutos*/));

		let cors_permissive = CorsLayer::new()
				.allow_origin(tower_http::cors::Any) // Allow any origin (wildcard *)
				.allow_methods(tower_http::cors::Any) // Allow any method
				.allow_headers(tower_http::cors::Any	); // Allow any header


		let app = Router::new()
        .route("/status", axum::routing::get(get_status_handler))
				.route("/"      , axum::routing::get(get_front))
				.layer(cors_permissive)
        .with_state(status.clone()); 


		let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn get_status_handler(State(status): State<Arc<RwLock<AppState>>>) -> Json<Vec<ServiceStatus>> {
    let status_guard = status.read().await;
    Json(status_guard.status.clone())
}

async fn get_front(State(status): State<Arc<RwLock<AppState>>>) -> Html<String>{
	Html(status.read().await.front.clone())
}

pub struct AppState {
	pub status: Vec<ServiceStatus>,
	pub front : String,
}