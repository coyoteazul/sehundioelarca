use std::{sync::Arc, time::{Duration, SystemTime, UNIX_EPOCH}};

use integracion_arca::{types::FEDummyResult, wsfev1};
use reqwest::Client;
use serde::Serialize;

use tokio::{join, sync::RwLock, time};

use crate::{front::render_front, AppState};


///Inicia un job que consulta el estado de los servicios de Arca cada tanto tiempo y actualiza el status
pub fn check_status_job(
    status: Arc<RwLock<AppState>>,
		tiempo: Duration
) {
		let req_cli = Client::new();
    tokio::spawn(async move {
				let mut interval = time::interval(tiempo); 

				loop {
						interval.tick().await;
						let proxima = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() + tiempo.as_secs();
  					
						let new_status = get_status(&req_cli).await;
						dbg!(&new_status);
						
						match status.write().await {
							mut locked => {
								locked.front = render_front(&new_status, proxima, tiempo);
								locked.status = new_status;
							}
						}
						
						println!("Updated status");
				}
		});
}

async fn get_status(req_cli:&Client)->Vec<ServiceStatus> {
	let (wsfev1_prod,wsfev1_homo )= join!( 
		wsfev1::service_status(req_cli, true),
		wsfev1::service_status(req_cli, false)
	);

	vec![
		ServiceStatus::from_dummy("Factura Nacional (wsfev1) Produccion"  , wsfev1_prod),
		ServiceStatus::from_dummy("Factura Nacional (wsfev1) Homologacion", wsfev1_homo),
	]
}



#[derive(Debug, Clone, Serialize)]
pub struct ServiceStatus {
	name       : String,
	status     : u16,
	app_server : bool,
	db_server  : bool,
	auth_server: bool,
	milis_respuesta : u128,
}

impl ServiceStatus {
	pub fn from_dummy(name: &str, dummy:FEDummyResult) -> Self{
		let FEDummyResult{ status, app_server, db_server, auth_server , milis_respuesta} = dummy;
		ServiceStatus {
			name: name.to_owned(),
			status: status.as_u16(),
			milis_respuesta,
			app_server,
			db_server,
			auth_server,
		}
	}
}
