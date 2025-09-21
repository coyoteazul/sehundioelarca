use std::time::Duration;

use lazy_static::lazy_static;
use tera::Tera;

use crate::arca::ServiceStatus;

lazy_static!{
	pub static ref TEMPLATES: Tera = Tera::new("templates/**/*").unwrap();
}

pub fn render_front(status:&Vec<ServiceStatus>, proxima:u64, intervalo:Duration) -> String {
	let minutos = intervalo.as_secs()/60;

	let mut context = tera::Context::new();
	context.insert("services", status);
	context.insert("next_update", &(proxima+10)/*meto 10 segundos de changui */);
	context.insert("intervalo", &minutos);

	TEMPLATES.render("index.tera", &context).unwrap()
}