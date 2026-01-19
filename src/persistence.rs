use std::fs;
use std::fs::OpenOptions;
use std::io;
use std::net::{AddrParseError, Ipv6Addr};
use std::path::Path;
use std::path::PathBuf;

#[derive(Debug)]
pub enum Error {
    CannotUseFile(String),
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
    pub fn new(file_path: &Path) -> Result<Self, Error> {
        let _ = OpenOptions::new()
            .write(true)
            .create(true) // Create if it doesn't exist
            .truncate(false) // Do NOT wipe the file if it exists
            .open(file_path)
            .map_err(|e| Error::CannotUseFile(e.to_string()))?;

        Ok(Self {
            file_path: file_path.to_path_buf(),
        })
    }
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

