use std::{net::Ipv6Addr, num::ParseIntError, time::Duration};
use tokio::{
    fs::File,
    io::{AsyncBufReadExt, BufReader},
    sync::mpsc::Sender,
}; // Use Tokio's async Sender

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
}

pub struct IpGrabber {
    pub iface: String,
    pub ip: Option<Ipv6Addr>,
}

impl IpGrabber {
    pub fn new(iface: String, start_ip: Option<Ipv6Addr>) -> Result<Self, Error> {
        Ok(Self {
            iface,
            ip: start_ip,
        })
    }
    /// Monitors the interface for a stable Global IPv6 address.
    /// Only sends the IP if it is found and is DIFFERENT from the last one sent.
    pub async fn run(&self, sender: Sender<Ipv6Addr>, poll_secs: u64) {
        let mut last_ip = self.ip;
        let mut interval = tokio::time::interval(Duration::from_secs(poll_secs));

        loop {
            // Wait for the next tick (polls every 5 seconds)
            interval.tick().await;

            match self.get_stable_global_ipv6().await {
                Ok(current_ip) => {
                    // Check if the IP has changed since the last successful check
                    if last_ip != Some(current_ip) {
                        log::info!("New Stable IPv6 detected: {}", current_ip);

                        // Send the new IP. If the receiver dropped, stop the loop.
                        if sender.send(current_ip).await.is_err() {
                            log::warn!("Receiver dropped. Stopping monitor.");
                            break;
                        }

                        // Update cache
                        last_ip = Some(current_ip);
                    }
                }
                Err(e) => {
                    // Optional: Handle case where IP is lost (e.g., interface goes down)
                    if last_ip.is_some() {
                        log::error!("Stable IPv6 lost. Due to: {e:?}");
                        last_ip = None;
                    }
                }
            }
        }
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
