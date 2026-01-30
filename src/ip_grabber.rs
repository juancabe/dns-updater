use std::{
    net::{IpAddr, Ipv4Addr, Ipv6Addr},
    num::ParseIntError,
    time::Duration,
};
use tokio::{
    fs::File,
    io::{AsyncBufReadExt, BufReader},
    sync::mpsc::Sender,
};

use crate::IpVersion; // Use Tokio's async Sender

#[derive(Debug)]
pub enum ParseError {
    LenMismatch,
    InvalidStr(ParseIntError),
}

#[derive(Debug)]
pub enum Error {
    OpenFileError(tokio::io::Error),
    ReadLineError(tokio::io::Error),
    ParseError(ParseError),
    NoneMatched,
    HttpError(reqwest::Error),
    AddrParseError(std::net::AddrParseError),
}

pub struct IpGrabber {
    iface: String,
    ip_version: IpVersion,
    poll_secs: u64,
    last_ip: Option<IpAddr>,
}

impl IpGrabber {
    pub fn new(iface: String, ip_version: IpVersion, poll_secs: u64) -> Result<Self, Error> {
        Ok(Self {
            iface,
            ip_version,
            poll_secs,
            last_ip: None,
        })
    }

    async fn get_updated(&self) -> Result<IpAddr, Error> {
        match self.ip_version {
            IpVersion::V4 => self.get_public_ipv4().await.map(IpAddr::V4),
            IpVersion::V6 => self.get_stable_global_ipv6().await.map(IpAddr::V6),
        }
    }

    /// Monitors the interface for a stable Global IPv6 address.
    /// Only sends the IP if it is found and is DIFFERENT from the last one sent.
    pub async fn run(&mut self, sender: Sender<IpAddr>) {
        let mut interval = tokio::time::interval(Duration::from_secs(self.poll_secs));
        let mut err_interval = tokio::time::interval(Duration::from_secs(self.poll_secs / 10));
        loop {
            match self.get_updated().await {
                Ok(current_ip) => {
                    // Check if the IP has changed since the last successful check
                    if let Some(last_ip) = self.last_ip
                        && current_ip == last_ip
                    {
                        interval.tick().await;
                        continue;
                    }

                    self.last_ip = Some(current_ip);

                    log::info!("New Stable ip detected: {}", current_ip);

                    // Send the new IP. If the receiver dropped, stop the loop.
                    if sender.send(current_ip).await.is_err() {
                        log::warn!("Receiver dropped. Stopping monitor.");
                        break;
                    }
                }
                Err(e) => {
                    log::debug!("Couldn't find an IP now, will try again, error: {e:?}");
                    err_interval.tick().await;
                }
            }
        }
    }

    pub async fn get_public_ipv4(&self) -> Result<Ipv4Addr, Error> {
        let response = reqwest::get("https://api.ipify.org")
            .await
            .map_err(Error::HttpError)?;

        let content = response.text().await.map_err(Error::HttpError)?;
        content.trim().parse().map_err(Error::AddrParseError)
    }

    pub async fn get_stable_global_ipv6(&self) -> Result<Ipv6Addr, Error> {
        const FILE_PATH: &str = "/proc/net/if_inet6";
        let file = File::open(FILE_PATH).await.map_err(Error::OpenFileError)?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();

        while let Some(line) = lines.next_line().await.map_err(Error::ReadLineError)? {
            let parts: Vec<&str> = line.split_whitespace().collect();

            if parts.len() < 6 {
                continue;
            }

            let name = parts[5];
            if name != self.iface {
                continue;
            }

            let scope = u8::from_str_radix(parts[3], 16).unwrap_or(0xFF);
            let flags = u8::from_str_radix(parts[4], 16).unwrap_or(0xFF);

            if scope != 0x00 {
                continue;
            }

            let is_temporary = (flags & 0x01) == 0x01;
            let is_deprecated = (flags & 0x20) == 0x20;

            if !is_temporary && !is_deprecated {
                return Self::parse_ipv6(parts[0]).map_err(Error::ParseError);
            }
        }

        Err(Error::NoneMatched)
    }

    fn parse_ipv6(hex: &str) -> Result<Ipv6Addr, ParseError> {
        if hex.len() != 32 {
            return Err(ParseError::LenMismatch);
        }
        let mut segments = [0u16; 8];
        for i in 0..8 {
            segments[i] = u16::from_str_radix(&hex[i * 4..(i + 1) * 4], 16)
                .map_err(ParseError::InvalidStr)?;
        }
        Ok(Ipv6Addr::from(segments))
    }
}
