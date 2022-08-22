use ipnet::IpNet;
use rbus::{self, client::Receiver};
use std::net::Ipv6Addr;
use std::time::Duration;
use std::{env, net::Ipv4Addr};
use zos_traits::NetlinkAddresses;

mod zos;
use crate::{
    app::Stubs,
    zos_traits::{
        IdentityManagerStub, NetworkerStub, RegistrarStub, StatisticsStub, SystemMonitorStub,
        VersionMonitorStub,
    },
};
mod app;
mod zos_traits;
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
    // let ignore_case = env::var("IGNORE_CASE").unwrap();
    let tick_rate = Duration::from_millis(250);
    run(stubs, tick_rate, true).await?;

    // let mut recev: Receiver<NetlinkAddresses> = loop {
    //     match network.zos_addresses().await {
    //         Ok(recev) => {
    //             break recev;
    //         }
    //         Err(err) => {
    //             println!("Error executing version method: {}", err);
    //             continue;
    //         }
    //     };
    // };
    // loop {
    //     let cpu = match recev.recv().await {
    //         Some(res) => match res {
    //             Ok(cpu) => cpu,
    //             Err(err) => {
    //                 println!("Error getting ZOS IP usage: {}", err);
    //                 continue;
    //             }
    //         },
    //         None => {
    //             println!("None");
    //             continue;
    //         }
    //     };
    //     let mut ip_str = String::from("");
    //     for entry in cpu {
    //         ip_str = format!("{} {}", ip_str, entry.to_string());
    //     }
    //     println!("ZOS IP: {}", &ip_str)
    // }

    Ok(())
}
