use std::path::PathBuf;

use tokio::sync::mpsc::channel;

use crate::{
    ip_grabber::IpGrabber,
    persistence::{self, Persistence},
};

pub struct Runner {
    grabber: IpGrabber,
    poll_secs: u64,
    pers: Persistence,
    dns_token: String,
}

impl Runner {
    pub fn new(
        iface: String,
        poll_secs: u64,
        pers_file_path: Option<&PathBuf>,
        dns_token: String,
    ) -> Self {
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
            dns_token,
        }
    }

    pub async fn run(self) {
        let Runner {
            grabber,
            poll_secs: _,
            pers,
            dns_token,
        } = self;

        let (sender, mut receiver) = channel(10000);

        tokio::spawn(async move { grabber.run(sender, self.poll_secs).await });

        while let Some(ip) = receiver.recv().await {
            if let Err(e) = pers.replace_ip(&ip) {
                log::error!("Error when saving IP: {e:?}");
            }

            log::info!("Detected new IP: {}, updating FreeDNS...", ip);

            let update_url = format!(
                "https://freedns.afraid.org/dynamic/update.php?{}&address={}",
                dns_token, ip
            );

            // Send the request
            match reqwest::get(&update_url).await {
                Ok(resp) => {
                    if resp.status().is_success() {
                        log::info!("FreeDNS update successful for {}", ip);
                    } else {
                        log::error!("FreeDNS update failed: Status {}", resp.status());
                    }
                }
                Err(e) => log::error!("Failed to send request to FreeDNS: {:?}", e),
            }
        }
    }
}
