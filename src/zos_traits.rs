use anyhow::Result;
use ipnet::IpNet;
use rbus::{object, server::Sender};
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, Bytes};
use std::fmt::Display;
use std::net::{Ipv4Addr, Ipv6Addr};

type FarmID = u32;

#[object(name = "manager", version = "0.0.1")]
pub trait IdentityManager {
    #[rename("FarmID")]
    fn farm_id(&self) -> Result<FarmID>;
    #[rename("Farm")]
    fn farm(&self) -> Result<String>;
}

#[object(name = "registrar", version = "0.0.1")]
pub trait Registrar {
    #[rename("NodeID")]
    fn node_id(&self) -> Result<u32>;
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PRVersion {
    #[serde(rename = "VersionStr")]
    pub version_str: String,
    #[serde(rename = "VersionNum")]
    pub version_num: u64,
    #[serde(rename = "IsNum")]
    pub is_num: bool,
}

impl Display for PRVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_num {
            write!(f, "{}", self.version_num)
        } else {
            write!(f, "{}", self.version_str)
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Version {
    #[serde(rename = "Major")]
    pub major: u64,
    #[serde(rename = "Minor")]
    pub minor: u64,
    #[serde(rename = "Patch")]
    pub patch: u64,
    #[serde(rename = "Pre")]
    pub pre: Option<Vec<PRVersion>>,
    #[serde(rename = "Build")]
    pub build: Option<Vec<String>>, //No Precendence
}

impl Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)?;

        if let Some(pre) = &self.pre {
            write!(f, "_{}", pre[0])?;
            for pre_version in pre[1..].iter() {
                write!(f, ".{}", pre_version)?
            }
        }

        if let Some(build) = &self.build {
            write!(f, "+{}", build[0])?;
            for build_item in build[1..].iter() {
                write!(f, ".{}", build_item)?;
            }
        }

        Ok(())
    }
}

#[object(name = "monitor", version = "0.0.1")]
#[async_trait::async_trait]
pub trait VersionMonitor {
    #[rename("Version")]
    #[stream]
    async fn version(&self, rec: Sender<Version>);
}

pub type Unit = u64;
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Capacity {
    #[serde(rename = "CRU")]
    pub cru: u64,
    #[serde(rename = "SRU")]
    pub sru: Unit,
    #[serde(rename = "HRU")]
    pub hru: Unit,
    #[serde(rename = "MRU")]
    pub mru: Unit,
    #[serde(rename = "IPV4U")]
    pub ipv4u: u64,
}

#[object(name = "statistics", version = "0.0.1")]
#[async_trait::async_trait]
pub trait Statistics {
    #[rename("ReservedStream")]
    #[stream]
    async fn reserved(&self, rec: Sender<Capacity>);
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct VirtualMemory {
    #[serde(rename = "Total")]
    pub total: u64,
    #[serde(rename = "Available")]
    pub available: u64,
    #[serde(rename = "Used")]
    pub used: u64,
    #[serde(rename = "UsedPercent")]
    pub used_percent: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimesStat {
    #[serde(rename = "Percent")]
    pub percent: f64,
}

#[object(name = "system", version = "0.0.1")]
#[async_trait::async_trait]
pub trait SystemMonitor {
    #[rename("CPU")]
    #[stream]
    async fn cpu(&self, rec: Sender<TimesStat>);
    #[rename("Memory")]
    #[stream]
    async fn memory(&self, rec: Sender<VirtualMemory>);
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IPNet {
    #[serde(rename = "IP")]
    #[serde_as(as = "Bytes")]
    pub ip: Vec<u8>,

    #[serde_as(as = "Bytes")]
    #[serde(rename = "Mask")]
    pub mask: Vec<u8>,
}

impl IPNet {
    pub fn to_string(&self) -> String {
        let mut ip_str = String::from("");
        if self.ip.len() == 4 {
            let ip4 = Ipv4Addr::new(self.ip[0], self.ip[1], self.ip[2], self.ip[3]);
            ip_str = format!("{} {}", &ip_str, ip4.to_string());
        } else if self.ip.len() == 16 {
            let ip_arr: [u8; 16] = [
                self.ip[0],
                self.ip[1],
                self.ip[2],
                self.ip[3],
                self.ip[4],
                self.ip[5],
                self.ip[6],
                self.ip[7],
                self.ip[8],
                self.ip[9],
                self.ip[10],
                self.ip[11],
                self.ip[12],
                self.ip[13],
                self.ip[14],
                self.ip[15],
            ];

            let ip6 = Ipv6Addr::from(ip_arr);
            ip_str = format!("{} {}", &ip_str, ip6.to_string())
        }
        return ip_str.trim().to_string();
    }
}
impl From<IpNet> for IPNet {
    fn from(ipnet: IpNet) -> Self {
        IPNet {
            ip: ipnet.addr().to_string().as_bytes().to_vec(),
            mask: ipnet.netmask().to_string().as_bytes().to_vec(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptionPublicConfig {
    #[serde(rename = "IPv4")]
    pub ipv4: IPNet,
    #[serde(rename = "IPv6")]
    pub ipv6: IPNet,
    #[serde(rename = "HasPublicConfig")]
    pub has_public_config: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExitDevice {
    // IsSingle is set to true if br-pub
    // is connected to zos bridge
    #[serde(rename = "IsSingle")]
    pub is_single: bool,
    // IsDual is set to true if br-pub is
    // connected to a physical nic
    #[serde(rename = "IsDual")]
    pub is_dual: bool,
    // AsDualInterface is set to the physical
    // interface name if IsDual is true
    #[serde(rename = "AsDualInterface")]
    pub as_dual_interface: String,
}
impl ExitDevice {
    pub fn to_string(&self) -> String {
        if self.is_single {
            String::from("Single")
        } else if self.is_dual {
            format!("Dual {}", self.as_dual_interface)
        } else {
            String::from("Unknown")
        }
    }
}

pub type NetlinkAddresses = Vec<IPNet>;
#[object(name = "network", version = "0.0.1")]
#[async_trait::async_trait]
pub trait Networker {
    #[rename("ZOSAddresses")]
    #[stream]
    async fn zos_addresses(&self, rec: Sender<NetlinkAddresses>);

    #[rename("YggAddresses")]
    #[stream]
    async fn ygg_addresses(&self, rec: Sender<NetlinkAddresses>);

    #[rename("DMZAddresses")]
    #[stream]
    async fn dmz_addresses(&self, rec: Sender<NetlinkAddresses>);

    #[rename("PublicAddresses")]
    #[stream]
    async fn public_addresses(&self, rec: Sender<OptionPublicConfig>);

    #[rename("GetPublicExitDevice")]
    fn get_public_exit_device(&self) -> Result<ExitDevice>;
}
