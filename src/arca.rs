use std::{sync::Arc, time::{Duration, SystemTime, UNIX_EPOCH}};

use integracion_arca::{types::FEDummyResult, wscpe, wsfev1, wsfexv1, wslpg, wsmtxca};
use reqwest::Client;
use serde::Serialize;

use tokio::{join, sync::RwLock, time};

use crate::{front::render_front, AppState};

const SECS_INTERVAL: Duration = Duration::from_secs(120);
const LARGO_HISTORIAL: u64 = 24*60*60 / SECS_INTERVAL.as_secs();
//const LARGO_HISTORIAL: u64 = 1440;

///Inicia un job que consulta el estado de los servicios de Arca cada tanto tiempo y actualiza el status
pub fn check_status_job(
    status: Arc<RwLock<AppState>>
) {
		let req_cli = Client::new();
    tokio::spawn(async move {
				let mut interval = time::interval(SECS_INTERVAL); 

				loop {
						interval.tick().await;
						let proxima = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() + SECS_INTERVAL.as_secs();
						  					
						let new_status = get_status(&req_cli).await;
						//dbg!(&new_status);
						
						match status.write().await {
							mut locked => {

								locked.status.get_from_dummy(new_status);
								

								locked.front = render_front(&locked.status, proxima, SECS_INTERVAL);
								
							}
						}
						
						println!("Updated status");
				}
		});
}
	
async fn get_status(req_cli:&Client)->Dummies {
	let (
		wsfev1_prod , wsfev1_homo , wsfexv1_prod, wsfexv1_homo, 
		wsmtxca_prod, wsmtxca_homo, wscpe_prod  , wscpe_homo,
		wslpg_prod  , wslpg_homo,
	)= join!( 
		wsfev1::service_status( req_cli, true , None),
		wsfev1::service_status( req_cli, false, None),
		wsfexv1::service_status(req_cli, true , None),
		wsfexv1::service_status(req_cli, false, None),
		wsmtxca::service_status(req_cli, true , None),
		wsmtxca::service_status(req_cli, false, None),
		wscpe::service_status(  req_cli, true , None),
		wscpe::service_status(  req_cli, false, None),
		wslpg::service_status(  req_cli, true , None),
		wslpg::service_status(  req_cli, false, None),
	);

	Dummies { wsfev1_prod, wsfev1_homo, wsfexv1_prod, wsfexv1_homo, wsmtxca_prod, wsmtxca_homo, wscpe_prod, wscpe_homo, wslpg_prod, wslpg_homo }
}



#[derive(Debug, Clone, Serialize)]
pub struct ServiceStatus {
	name       : String,
	status     : u16,
	status_msg : String,
	app_server : bool,
	db_server  : bool,
	auth_server: bool,
	milis_respuesta : u128,
	historial  : Vec<TriEstado>
}

impl ServiceStatus {
	fn new(name:String) -> Self {
		ServiceStatus {
			name,
			status: 500,
			status_msg: "Desconocido".to_owned(),
			app_server: false,
			db_server: false,
			auth_server: false,
			milis_respuesta: 0,
			historial: vec![],
		}
	}

	fn tri_estado(&self) -> TriEstado{
		if self.status != reqwest::StatusCode::OK {
			TriEstado::Error
		} else if self.milis_respuesta > 1000  {
			TriEstado::Alerta
		} else {
			TriEstado::Ok
		}
	}

	fn get_from_dummy(&mut self, dum:FEDummyResult) {
		self.app_server      = dum.app_server;
		self.auth_server     = dum.auth_server;
		self.db_server       = dum.db_server;
		self.milis_respuesta = dum.milis_respuesta;
		self.status_msg      = dum.status.canonical_reason().unwrap_or("Desconocido").to_owned();
		self.status          = dum.status.as_u16();

		self.historial.push(self.tri_estado());
		if self.historial.len() > LARGO_HISTORIAL.try_into().unwrap() {
			self.historial.remove(0);
		}
	}
}

#[derive(Debug, Clone, Serialize)]
pub enum TriEstado {
	Error,
	Alerta,
	Ok
}

#[derive(Debug, Clone, Serialize)]
pub struct Servicios {
	wsfev1_prod : ServiceStatus,
	wsfev1_homo : ServiceStatus,
	wsfexv1_prod: ServiceStatus,
	wsfexv1_homo: ServiceStatus,
	wsmtxca_prod: ServiceStatus,
	wsmtxca_homo: ServiceStatus,
	wscpe_prod  : ServiceStatus,
	wscpe_homo  : ServiceStatus,
	wslpg_prod  : ServiceStatus,
	wslpg_homo  : ServiceStatus,
}

impl Servicios {
	pub fn new() -> Self {
		Servicios{
			wsfev1_prod : ServiceStatus::new("Factura Nacional (wsfev1)<br>Producción".to_owned()      ),
			wsfev1_homo : ServiceStatus::new("Factura Nacional (wsfev1)<br>Homologación".to_owned()    ),
			wsfexv1_prod: ServiceStatus::new("Factura Exportación (wsfexv1)<br>Producción".to_owned()  ),
			wsfexv1_homo: ServiceStatus::new("Factura Exportación (wsfexv1)<br>Homologación".to_owned()),
			wsmtxca_prod: ServiceStatus::new("Factura Nominación (wsmtxca)<br>Producción".to_owned()   ),
			wsmtxca_homo: ServiceStatus::new("Factura Nominación (wsmtxca)<br>Homologación".to_owned() ),
			wscpe_prod  : ServiceStatus::new("Carta de Porte (wscpe)<br>Producción".to_owned()         ),
			wscpe_homo  : ServiceStatus::new("Carta de Porte (wscpe)<br>Homologación".to_owned()       ),
			wslpg_prod  : ServiceStatus::new("Liq. Prim. Granos (wslpg)<br>Producción".to_owned()      ),
			wslpg_homo  : ServiceStatus::new("Liq. Prim. Granos (wslpg)<br>Homologación".to_owned()    ),
		}
	}

	fn get_from_dummy(&mut self, dum:Dummies) {
			self.wsfev1_prod .get_from_dummy(dum.wsfev1_prod );
			self.wsfev1_homo .get_from_dummy(dum.wsfev1_homo );
			self.wsfexv1_prod.get_from_dummy(dum.wsfexv1_prod);
			self.wsfexv1_homo.get_from_dummy(dum.wsfexv1_homo);
			self.wsmtxca_prod.get_from_dummy(dum.wsmtxca_prod);
			self.wsmtxca_homo.get_from_dummy(dum.wsmtxca_homo);
			self.wscpe_prod  .get_from_dummy(dum.wscpe_prod  );
			self.wscpe_homo  .get_from_dummy(dum.wscpe_homo  );
			self.wslpg_prod  .get_from_dummy(dum.wslpg_prod  );
			self.wslpg_homo  .get_from_dummy(dum.wslpg_homo  );
	}

	pub fn to_vec(&self) -> Vec<&ServiceStatus> {
		vec![
			&self.wsfev1_prod ,
			&self.wsfev1_homo ,
			&self.wsfexv1_prod,
			&self.wsfexv1_homo,
			&self.wsmtxca_prod,
			&self.wsmtxca_homo,
			&self.wscpe_prod  ,
			&self.wscpe_homo  ,
			&self.wslpg_prod  ,
			&self.wslpg_homo  ,
		]
	}
}

//impl ServiceStatus {
//	pub fn from_dummy(name: &str, dummy:FEDummyResult) -> Self{
//		let FEDummyResult{ status, app_server, db_server, auth_server , milis_respuesta} = dummy;
//		ServiceStatus {
//			name: name.to_owned(),
//			status: status.as_u16(),
//			milis_respuesta,
//			app_server,
//			db_server,
//			auth_server,
//		}
//	}
//}

#[derive(Debug)]
struct Dummies {
	wsfev1_prod : FEDummyResult,
	wsfev1_homo : FEDummyResult,
	wsfexv1_prod: FEDummyResult,
	wsfexv1_homo: FEDummyResult,
	wsmtxca_prod: FEDummyResult,
	wsmtxca_homo: FEDummyResult,
	wscpe_prod  : FEDummyResult,
	wscpe_homo  : FEDummyResult,
	wslpg_prod  : FEDummyResult,
	wslpg_homo  : FEDummyResult,
}