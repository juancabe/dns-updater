use std::net::IpAddr;

use async_trait::async_trait;

use crate::{IpVersion, SimpleName};

#[async_trait]
pub trait DynDns: PersistsToFile + Send + Sync {
    // ip is optional because for Ip4Addr APIs auto detect
    async fn update(&mut self, ip: IpAddr) -> Result<(), String>;
    fn get_ip_version(&self) -> IpVersion;
    fn get_poll_secs(&self) -> u64;
}

pub trait PersistsToFile {
    fn file_name(&self) -> &str;
}

#[derive(Debug)]
pub struct FreeDns {
    token: String,
    file_name: String,
    ip_version: IpVersion,
    poll_secs: u64,
}

impl FreeDns {
    pub fn new(token: String, ip_version: IpVersion, poll_secs: u64) -> Self {
        let file_name = format!("FreeDNS_{}_{}", token, ip_version.simple_name());
        let s = Self {
            token,
            file_name,
            ip_version,
            poll_secs,
        };
        log::info!("Created DynDns: {s:?}");
        s
    }
}

impl PersistsToFile for FreeDns {
    fn file_name(&self) -> &str {
        &self.file_name
    }
}

#[async_trait]
impl DynDns for FreeDns {
    async fn update(&mut self, ip: IpAddr) -> Result<(), String> {
        let mut update_url = format!(
            "https://freedns.afraid.org/dynamic/update.php?{}",
            self.token
        );
        if let IpAddr::V6(ip) = ip {
            update_url.push_str("&address=");
            update_url.push_str(&ip.to_string());
        }

        log::info!("Calling HTTP: {update_url}");
        match reqwest::get(&update_url).await {
            Ok(resp) => {
                if resp.status().is_success() {
                    log::info!("FreeDNS update successful for {}", ip);
                    Ok(())
                } else {
                    Err(format!("FreeDNS update failed: Status {}", resp.status()))
                }
            }
            Err(e) => Err(format!("Failed to send request to FreeDNS: {:?}", e)),
        }
    }

    fn get_ip_version(&self) -> IpVersion {
        self.ip_version
    }

    fn get_poll_secs(&self) -> u64 {
        self.poll_secs
    }
}

#[derive(Debug)]
pub struct DuckDns {
    token: String,
    name: String,
    file_name: String,
    ip_version: IpVersion,
    poll_secs: u64,
}

impl DuckDns {
    pub fn new(token: String, name: String, ip_version: IpVersion, poll_secs: u64) -> Self {
        let file_name = format!("DuckDNS_{}_{}", token, name);
        let s = Self {
            token,
            name,
            file_name,
            ip_version,
            poll_secs,
        };
        log::info!("Created DynDns: {s:?}");
        s
    }
}

impl PersistsToFile for DuckDns {
    fn file_name(&self) -> &str {
        &self.file_name
    }
}

#[async_trait]
impl DynDns for DuckDns {
    async fn update(&mut self, ip: IpAddr) -> Result<(), String> {
        let mut update_url = format!(
            "https://www.duckdns.org/update?domains={}&token={}",
            self.name, self.token
        );
        if let IpAddr::V6(ip) = ip {
            update_url.push_str("&ipv6=");
            update_url.push_str(&ip.to_string());
        }
        log::info!("Calling HTTP: {update_url}");
        match reqwest::get(&update_url).await {
            Ok(resp) => {
                if resp.status().is_success() {
                    log::info!("DuckDNS update successful for {}", ip);
                    Ok(())
                } else {
                    Err(format!("DuckDNS update failed: Status {}", resp.status()))
                }
            }
            Err(e) => Err(format!("Failed to send request to DuckDNS: {:?}", e)),
        }
    }

    fn get_ip_version(&self) -> IpVersion {
        self.ip_version
    }

    fn get_poll_secs(&self) -> u64 {
        self.poll_secs
    }
}

pub fn parse_dns_tuples(to_parse: &str) -> Result<Vec<Box<dyn DynDns>>, String> {
    // to_parse := BATCH,BATCH,...

    // let free_dns = FreeDns::new(token, ip_version);
    // ("FD";TOKEN;VERSION;POLL_SECS) = BATCH

    // let duck_dns = DuckDns::new(token, name, ip_version);
    // ("DD";TOKEN;VERSION;POLL_SECS;NAME) = BATCH
    //
    // Parenthesis are not mandatory

    to_parse
        .split(",")
        .map(|s| {
            s.trim()
                .trim_start_matches("(")
                .trim_end_matches(")")
                .split(";")
        })
        .map(|mut parts| match parts.next() {
            None => Err("Empty Batch found".to_string()),
            Some("FD") => {
                let token = parts
                    .next()
                    .ok_or("No TOKEN found in batch".to_string())?
                    .to_string();
                let version: IpVersion = parts
                    .next()
                    .ok_or("No VERSION found in batch".to_string())?
                    .try_into()?;
                let poll_secs: u64 = parts
                    .next()
                    .ok_or("No POLL_SECS found in batch".to_string())?
                    .parse()
                    .map_err(|e| format!("Couldn't parse POLL_SECS error: {e:?}"))?;

                Ok(Box::new(FreeDns::new(token, version, poll_secs)) as Box<dyn DynDns>)
            }
            Some("DD") => {
                let token = parts
                    .next()
                    .ok_or("No TOKEN found in batch".to_string())?
                    .to_string();
                let version: IpVersion = parts
                    .next()
                    .ok_or("No VERSION found in batch".to_string())?
                    .try_into()?;
                let poll_secs: u64 = parts
                    .next()
                    .ok_or("No POLL_SECS found in batch".to_string())?
                    .parse()
                    .map_err(|e| format!("Couldn't parse POLL_SECS error: {e:?}"))?;

                let name = parts
                    .next()
                    .ok_or("No NAME found in batch".to_string())?
                    .to_string();
                Ok(Box::new(DuckDns::new(token, name, version, poll_secs)) as Box<dyn DynDns>)
            }
            Some(t) => Err(format!("Invalid Dynamic Dns Type found: {t}")),
        })
        .collect()
}

#[cfg(test)]
mod test {
    use crate::{SimpleName, dyn_dns::parse_dns_tuples};

    #[test]
    fn test_parse() {
        assert!(parse_dns_tuples("").is_err());

        let fd_example = "(FD;8709122eruoi189014h;ipv4;0),FD;8709122eruoi189014h;ipv6;125;";
        parse_dns_tuples(fd_example).expect("Not fail");
        assert!(parse_dns_tuples(fd_example).is_ok_and(|e| {
            assert_eq!(e[0].get_ip_version().simple_name(), "ipv4");
            assert_eq!(e[1].get_ip_version().simple_name(), "ipv6");
            e.get(2).is_none()
        }));

        let fd_fails = "(FD;8709122eruoi189014h;),FD;8709122eruoi189014h;ipv6";
        assert!(parse_dns_tuples(fd_fails).is_err());

        let dd_example =
            "(DD;8709122eruoi189014h;ipv4;123;jejejej),DD;8709122eruoi189014h;ipv6;0;jheadwwj";
        parse_dns_tuples(dd_example).expect("Not fail");
        assert!(parse_dns_tuples(dd_example).is_ok_and(|e| {
            assert_eq!(e[0].get_ip_version().simple_name(), "ipv4");
            assert_eq!(e[1].get_ip_version().simple_name(), "ipv6");
            e.get(3).is_none()
        }));

        let dd_fails = "(DD;jejejej;;),DD;jajajaj;;ipv6";
        assert!(parse_dns_tuples(dd_fails).is_err());
    }
}
