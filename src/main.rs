use rbus;
use std::time::Duration;

mod zos;
use crate::{
    api::{
        IdentityManagerStub, NetworkerStub, RegistrarStub, StatisticsStub, SystemMonitorStub,
        VersionMonitorStub,
    },
    app::Stubs,
};
mod api;
mod app;
use core::result::Result;
use std::error::Error;
mod ui;
mod zui;
use crate::zui::run;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    const IDENTITY_MOD: &str = "identityd";
    let client = rbus::Client::new("redis://0.0.0.0:6379").await.unwrap();
    let identity_manager = IdentityManagerStub::new(IDENTITY_MOD, client.clone());

    let version_monitor = VersionMonitorStub::new(IDENTITY_MOD, client.clone());

    const REGISTRAR_MOD: &str = "registrar";
    let registrar = RegistrarStub::new(REGISTRAR_MOD, client.clone());

    const PROVISION_MOD: &str = "provision";
    let statistics = StatisticsStub::new(PROVISION_MOD, client.clone());

    const NODE_MOD: &str = "node";
    let sys_monitor = SystemMonitorStub::new(NODE_MOD, client.clone());

    const NETWORK_MOD: &str = "network";
    let network = NetworkerStub::new(NETWORK_MOD, client.clone());

    let stubs = Stubs {
        identity_manager,
        registrar,
        version_monitor,
        statistics,
        sys_monitor,
        network,
    };
    let tick_rate = Duration::from_millis(250);
    run(stubs, tick_rate).await?;
    Ok(())
}
