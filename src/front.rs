use std::{io::Write, sync::Arc, time::Duration};

use axum::{body::Body, extract::State, http::{HeaderValue, Response}, response::IntoResponse};
use flate2::{write::GzEncoder, Compression};
use lazy_static::lazy_static;

use reqwest::header;
use tera::Tera;
use tokio::sync::RwLock;

use crate::{arca::Servicios, AppState};

lazy_static!{
	pub static ref TEMPLATES: Tera = Tera::new("templates/**/*").unwrap();
}

/// Renderiza el template con tera y retorna una version comprimida
pub fn render_front(status:&Servicios, proxima:u64, intervalo:Duration) -> Vec<u8> {
	let minutos = intervalo.as_secs()/60;

	let v = status.to_vec();

	let mut context = tera::Context::new();
	context.insert("services", &v);
	context.insert("next_update", &(proxima+2)/*meto 2 segundos de changui por si alguna consulta se atrasa*/);
	context.insert("intervalo", &minutos);

	let rendered = TEMPLATES.render("index.tera", &context).unwrap();
	let b = rendered.as_bytes();

	let mut encoder = GzEncoder::new(Vec::new(), Compression::best());
	let _ = encoder.write_all(b);
		
  let compressed = encoder.finish().unwrap();

	compressed
}

pub async fn get_front(State(status): State<Arc<RwLock<AppState>>>) -> impl IntoResponse{
	let bytes_ = status.read().await.front.clone();
	let body = Body::from(bytes_);

	let mut response = Response::new(body);
    response.headers_mut().insert(
        header::CONTENT_ENCODING,
        HeaderValue::from_static("gzip"),
    );
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("text/html; charset=utf-8"),
    );
    response
}

pub async fn get_css() -> impl IntoResponse {
	let css = TEMPLATES.render("styles.css", &tera::Context::new()).unwrap();
	let mut response = Response::new(css);
	
	response.headers_mut().insert(
			header::CONTENT_TYPE,
			HeaderValue::from_static("text/css; charset=utf-8"),
	);
	response.headers_mut().insert(
			header::CACHE_CONTROL,
			HeaderValue::from_static("public, max-age=31536000, immutable"),
	);
	response
}