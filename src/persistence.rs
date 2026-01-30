use std::fs;
use std::fs::OpenOptions;
use std::io;
use std::net::{AddrParseError, IpAddr};
use std::path::PathBuf;

#[derive(Debug)]
pub enum CreateError {
    NoFileNames,
    CannotUseFile(String),
}

#[derive(Debug)]
pub enum Error {
    CE(CreateError),
    Io(io::Error),
    Parse(AddrParseError),
    NoFileNames,
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
    pub file_paths: Vec<PathBuf>,
}

impl Persistence {
    pub fn new<'a, T: IntoIterator<Item = &'a str>>(file_names: T) -> Result<Self, Error> {
        let fps: Result<Vec<PathBuf>, CreateError> = file_names
            .into_iter()
            .map(|name| {
                let pb = PathBuf::from(name);
                let _ = OpenOptions::new()
                    .write(true)
                    .create(true) // Create if it doesn't exist
                    .truncate(false) // Do NOT wipe the file if it exists
                    .open(&pb)
                    .map_err(|e| CreateError::CannotUseFile(e.to_string()))?;
                Ok(pb)
            })
            .collect();
        let fps = fps.map_err(Error::CE)?;
        if fps.is_empty() {
            Err(Error::CE(CreateError::NoFileNames))?
        }
        Ok(Self { file_paths: fps })
    }

    fn match_file_name(&self, file_name: &str) -> Result<&PathBuf, Error> {
        self.file_paths
            .iter()
            .filter_map(|fp| {
                fp.to_str().and_then(|s| {
                    if s.ends_with(&file_name.to_string()) {
                        Some(fp)
                    } else {
                        None
                    }
                })
            })
            .next()
            .ok_or(Error::NoFileNames)
    }

    /// Overwrites the file with the new IP address
    pub async fn replace_ip(&self, ip: &IpAddr, file_name: &str) -> Result<(), Error> {
        let fp = self.match_file_name(file_name)?;
        tokio::fs::write(fp, ip.to_string()).await?;
        Ok(())
    }

    /// Reads the IP from the file
    pub fn load_ip(&self, file_name: &str) -> Result<IpAddr, Error> {
        let fp = self.match_file_name(file_name)?;
        let content = fs::read_to_string(fp)?;
        let ip = content.trim().parse()?;
        Ok(ip)
    }
}
