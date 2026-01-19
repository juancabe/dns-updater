use std::path::PathBuf;

use tokio::sync::mpsc::channel;

use crate::{ip_grabber::IpGrabber, persistence::Persistence};

pub mod ip_grabber;
pub mod persistence;

pub struct Runner {
    grabber: IpGrabber,
    poll_secs: u64,
    pers: Persistence,
}

impl Runner {
    pub fn new(iface: String, poll_secs: u64, pers_file_path: Option<&PathBuf>) -> Self {
        let pers = if let Some(fp) = pers_file_path {
            Persistence::new(fp).expect("File should be valid")
        } else {
            Persistence::default()
        };
        let ip = match pers.load_ip() {
            Ok(a) => Some(a),
            Err(e) => match e {
                persistence::Error::Io(error) => {
                    panic!("Unable to use persistence for the first time: {error:?}")
                }
                persistence::Error::Parse(addr_parse_error) => {
                    log::warn!("Error parsing saved IP, using none: {addr_parse_error:?}");
                    None
                }
                _ => unreachable!(),
            },
        };
        let grabber = IpGrabber::new(iface, ip).unwrap();
        Self {
            grabber,
            poll_secs,
            pers,
        }
    }
    pub async fn run(self) {
        let Runner {
            grabber,
            poll_secs: _,
            pers,
        } = self;

        let (sender, mut receiver) = channel(10000);

        tokio::spawn(async move { grabber.run(sender, self.poll_secs).await });

        while let Some(ip) = receiver.recv().await {
            if let Err(e) = pers.replace_ip(&ip) {
                log::error!("Error when saving IP: {e:?}");
            }
        }
    }
}

#[cfg(test)]
mod tests {}
