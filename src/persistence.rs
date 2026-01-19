use std::fs;
use std::io;
use std::net::{AddrParseError, Ipv6Addr};
use std::path::PathBuf;

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    Parse(AddrParseError),
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error::Io(err)
    }
}

impl From<AddrParseError> for Error {
    fn from(err: AddrParseError) -> Self {
        Error::Parse(err)
    }
}

pub struct Persistence {
    pub file_path: PathBuf,
}

impl Default for Persistence {
    fn default() -> Self {
        Self {
            file_path: PathBuf::from("./current_ipv6.txt"),
        }
    }
}

impl Persistence {
    /// Overwrites the file with the new IP address
    pub fn replace_ip(&self, ip: &Ipv6Addr) -> Result<(), Error> {
        fs::write(&self.file_path, ip.to_string())?;
        Ok(())
    }

    /// Reads the IP from the file
    pub fn load_ip(&self) -> Result<Ipv6Addr, Error> {
        let content = fs::read_to_string(&self.file_path)?;
        let ip = content.trim().parse()?;
        Ok(ip)
    }
}
