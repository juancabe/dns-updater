use tokio::sync::mpsc;

use crate::{
    dyn_dns::DynDns,
    ip_grabber::{self, IpGrabber},
    persistence::{self, Persistence},
};

pub type DynGrabber = (Box<dyn DynDns>, IpGrabber);

pub struct Runner {
    pers: Persistence,
    dyn_dnss: Vec<DynGrabber>,
}

#[derive(Debug)]
pub enum Error {
    PersistenceError(persistence::Error),
    GrabberError(ip_grabber::Error),
}

impl Runner {
    pub fn new(iface: String, dyn_dnss: Vec<Box<dyn DynDns>>) -> Result<Self, Error> {
        let pers = Persistence::new(dyn_dnss.iter().map(|dd| dd.file_name()))
            .map_err(Error::PersistenceError)?;

        let dyn_dnss: Result<Vec<DynGrabber>, ip_grabber::Error> = dyn_dnss
            .into_iter()
            .map(|dyn_dns| {
                let ipv = dyn_dns.get_ip_version();
                let ps = dyn_dns.get_poll_secs();
                Ok((dyn_dns, IpGrabber::new(iface.clone(), ipv, ps)?))
            })
            .collect();
        let dyn_dnss = dyn_dnss.map_err(Error::GrabberError)?;

        Ok(Self { pers, dyn_dnss })
    }

    pub async fn run(self) {
        let Runner { pers, dyn_dnss } = self;

        let (sender, mut receiver) = mpsc::channel(10000);

        let it = dyn_dnss.into_iter().map(|(mut dns, mut grabber)| {
            let (gs, mut gr) = mpsc::channel(10000);
            tokio::spawn(async move { grabber.run(gs).await });
            let sender = sender.clone();
            let file_name = dns.file_name().to_string();
            async move {
                while let Some(ip) = gr.recv().await {
                    match dns.update(ip).await {
                        Ok(()) => {
                            // Update successful, now persist the new IP
                            if let Err(e) = sender.send((ip, file_name.clone())).await {
                                log::error!("DNS update succeeded, but failed to send IP to persistence. The IP might be updated again unnecessarily on next check. Error: {e:?}");
                            }
                        }
                        Err(e) => {
                            log::error!("Error updating DNS: {e:?}")
                        }
                    }
                }
            }
        });

        for fut in it {
            tokio::spawn(fut);
        }

        drop(sender);

        while let Some((ip, file_name)) = receiver.recv().await {
            if let Err(e) = pers.replace_ip(&ip, &file_name).await {
                log::error!("Error when saving IP: {e:?}");
            }
        }
    }
}
