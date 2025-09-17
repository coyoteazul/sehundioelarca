use std::{sync::Arc, time::Duration};

use integracion_arca::{types::FEDummyResult, wsfev1::service_status};
use reqwest::Client;
use serde::Serialize;
use tokio::{join, sync::RwLock, time};


///Inicia un job que consulta el estado de los servicios de Arca cada tanto tiempo y actualiza el status
pub fn check_status_job(
    status: Arc<RwLock<Services>>,
		tiempo: Duration
) {
		let req_cli = Client::new();
    tokio::spawn(async move {
        let mut interval = time::interval(tiempo); 

        loop {
            interval.tick().await;

						let new_status = get_status(&req_cli).await;
						dbg!(&new_status);
						*status.write().await = new_status;
            println!("Updated status");
						
        }
    });
}

async fn get_status(req_cli:&Client)->Services{
	let (wsfev1_prod,wsfev1_homo )= join!( 
		service_status(req_cli, true),
		service_status(req_cli, false)
	);

	Services { 
		wsfev1_prod: wsfev1_prod.into(), 
		wsfev1_homo: wsfev1_homo.into()
 	}
}

#[derive(Debug, Clone, Serialize)]
pub struct Services {
	wsfev1_prod: ServiceStatus,
	wsfev1_homo: ServiceStatus
}

impl Services {
	pub fn new() -> Services{
		Services{ wsfev1_prod: ServiceStatus::new(), wsfev1_homo: ServiceStatus::new() }
	}
}

#[derive(Debug, Clone, Serialize)]
pub struct ServiceStatus {
	status     : u16,
	app_server : bool,
	db_server  : bool,
	auth_server: bool,
}

impl ServiceStatus {
	fn new() -> ServiceStatus {
		ServiceStatus{ 
			status: reqwest::StatusCode::SERVICE_UNAVAILABLE.as_u16(), 
			app_server: false, 
			db_server: false, 
			auth_server: false 
		}
	}
}

		
impl From<FEDummyResult> for ServiceStatus {
	fn from(FEDummyResult { status, app_server, db_server, auth_server }: FEDummyResult) -> Self {
		ServiceStatus {
			status: status.as_u16(),
			app_server,
			db_server,
			auth_server,
		}
	}
}