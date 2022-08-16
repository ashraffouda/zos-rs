use serde::{Deserialize, Serialize};
use serde_bytes::ByteBuf;
use std::{
    fmt::Display,
    net::{IpAddr, Ipv4Addr, Ipv6Addr},
};

/// IP is a Golang compatible IP type
/// According to the Go docs (and net pkg implementation) a 16 byte array does not mean
/// it's an Ipv6. A 16 bytes array can still hold Ipv4 address [IETF RFC 4291 section 2.5.5.1](https://tools.ietf.org/html/rfc4291#section-2.5.5.1)
///
/// In the matter of fact, all Ipv4 methods in Go net pkg will always create a 16 bytes
/// array to hold the Ipv4. Hence the code here need to interpret the format of the IP
/// not the array length.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct IP(ByteBuf);

impl From<IP> for IpAddr {
    fn from(ip: IP) -> Self {
        let inner = ip.0;
        if inner.len() == 4 {
            return IpAddr::V4(Ipv4Addr::new(inner[0], inner[1], inner[2], inner[3]));
        }
        // there must be a better way to do this
        let mut bytes: [u8; 16] = [0; 16];
        for (i, v) in inner.into_iter().take(16).enumerate() {
            bytes[i] = v;
        }
        let ipv6 = Ipv6Addr::from(bytes);
        if let Some(ipv4) = ipv6.to_ipv4() {
            IpAddr::V4(ipv4)
        } else {
            IpAddr::V6(ipv6)
        }
    }
}

impl From<&IP> for IpAddr {
    fn from(ip: &IP) -> Self {
        let inner = &ip.0;
        if inner.len() == 4 {
            return IpAddr::V4(Ipv4Addr::new(inner[0], inner[1], inner[2], inner[3]));
        }
        let mut bytes: [u8; 16] = [0; 16];
        for (i, v) in inner.iter().take(16).enumerate() {
            bytes[i] = *v;
        }
        let ipv6 = Ipv6Addr::from(bytes);
        if let Some(ipv4) = ipv6.to_ipv4() {
            IpAddr::V4(ipv4)
        } else {
            IpAddr::V6(ipv6)
        }
    }
}

impl Display for IP {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let addr: IpAddr = self.into();
        write!(f, "{}", addr)
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct IPMask(ByteBuf);

impl IPMask {
    pub fn bits(&self) -> u8 {
        let mut size: u8 = 0;
        for v in self.0.iter() {
            let mut x = *v;
            while x > 0 {
                x = x << 1;
                size += 1;
            }
        }
        size
    }
}

impl From<u8> for IPMask {
    fn from(size: u8) -> Self {
        // this is probably not the best way
        // to implement
        if size == 0 {
            return Self::default();
        }
        let mut v: Vec<u8> = vec![0];
        let mut index: usize = 0;
        for i in 0..size {
            v[index] = v[index] >> 1 | 0x80; // this is basically 0b1000 0000
            if v[index] == 0xff && i < size - 1 {
                // we only push new value if there is still more iterations
                v.push(0);
                index += 1;
            }
        }

        Self(ByteBuf::from(v))
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IPNet {
    #[serde(rename = "IP")]
    pub ip: IP,

    #[serde(rename = "Mask")]
    pub mask: IPMask,
}

impl Display for IPNet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/{}", self.ip, self.mask.bits())
    }
}

#[cfg(test)]
mod test {
    use super::{IPMask, IPNet, IP};
    use std::net::IpAddr;

    #[test]
    fn test_mask_bits() {
        let mask: IPMask = 16.into();
        assert!(mask.0[0] == 0xff);
        assert!(mask.0[1] == 0xff);

        assert!(mask.bits() == 16);

        let mask: IPMask = 18.into();
        assert!(mask.bits() == 18);
        assert!(mask.0[0] == 0xff);
        assert!(mask.0[1] == 0xff);
        assert!(mask.0[2] == 0b11000000);

        let mask: IPMask = 4.into();
        assert!(mask.bits() == 4);
        assert!(mask.0[0] == 0b11110000);

        let mask: IPMask = 6.into();
        assert!(mask.bits() == 6);
        assert!(mask.0[0] == 0b11111100);

        let mask: IPMask = 128.into();
        assert!(mask.bits() == 128);
        assert!(mask.0.len() == 16);
        assert!(mask.0.iter().all(|v| *v == 0xff));
    }

    #[test]
    fn test_go_compatibility() {
        // 192.168.1.20 (in a 16 bytes array)
        let data = "c41000000000000000000000ffffc0a80114";
        let data = hex::decode(data).unwrap();
        let ip: IP = rmp_serde::from_slice(&data).unwrap();
        let ip: IpAddr = ip.into();
        assert!(ip.to_string() == "192.168.1.20");

        // 2a10:b600:0:be77:f1d6:fc0:40ad:8b29
        let data = "c4102a10b6000000be77f1d60fc040ad8b29";
        let data = hex::decode(data).unwrap();
        let ip: IP = rmp_serde::from_slice(&data).unwrap();
        let ip: IpAddr = ip.into();
        assert!(ip.to_string() == "2a10:b600:0:be77:f1d6:fc0:40ad:8b29");

        // 192.168.1.0/24 (in ip net the ipv4 is actually in a 4 bytes array)
        let data = "82a24950c404c0a80100a44d61736bc404ffffff00";
        let data = hex::decode(data).unwrap();
        let net: IPNet = rmp_serde::from_slice(&data).unwrap();
        println!("{}", net);
        assert!(net.to_string() == "192.168.1.0/24");

        // 2a10:b600:0:be77::/64
        let data = "82a24950c4102a10b6000000be770000000000000000a44d61736bc410ffffffffffffffff0000000000000000";
        let data = hex::decode(data).unwrap();
        let net: IPNet = rmp_serde::from_slice(&data).unwrap();
        assert!(net.to_string() == "2a10:b600:0:be77::/64");
    }
}
