use std::net::IpAddr;

pub mod dyn_dns;
pub mod ip_grabber;
pub mod persistence;
pub mod runner;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum IpVersion {
    V4,
    V6,
}

impl From<&IpAddr> for IpVersion {
    fn from(ip: &IpAddr) -> Self {
        match ip {
            IpAddr::V4(_) => IpVersion::V4,
            IpAddr::V6(_) => IpVersion::V6,
        }
    }
}

pub trait SimpleName {
    fn simple_name(&self) -> &str;
}

impl SimpleName for IpVersion {
    fn simple_name(&self) -> &str {
        match self {
            IpVersion::V4 => "ipv4",
            IpVersion::V6 => "ipv6",
        }
    }
}

impl TryFrom<&str> for IpVersion {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "ipv4" => Ok(IpVersion::V4),
            "ipv6" => Ok(IpVersion::V6),
            _ => Err(format!("Invalid value: {value}")),
        }
    }
}

#[cfg(test)]
mod tests {}
