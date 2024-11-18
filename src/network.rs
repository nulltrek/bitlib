use crate::definitions::Network;
use crate::hashing::from_hex_str;
use crate::tx::TxId;
use ureq;

#[derive(Debug)]
pub enum NetworkError {
    RequestError(ureq::Error),
    IoError(std::io::Error),
    ParsingError,
}

impl From<ureq::Error> for NetworkError {
    fn from(error: ureq::Error) -> Self {
        NetworkError::RequestError(error)
    }
}

impl From<std::io::Error> for NetworkError {
    fn from(error: std::io::Error) -> Self {
        NetworkError::IoError(error)
    }
}

pub type Result<T> = std::result::Result<T, NetworkError>;

fn get_host(network: Network) -> &'static str {
    match network {
        Network::Main => "https://blockstream.info/api",
        Network::Test => "https://blockstream.info/testnet/api",
    }
}

fn fetch(network: Network, path: &str) -> Result<String> {
    Ok(ureq::get(&format!("{}{}", get_host(network), path))
        .call()?
        .into_string()?)
}

pub trait TxFetcher {
    fn new(network: Network) -> Self;
    fn fetch_tx(&self, id: &TxId) -> Result<Vec<u8>>;
}

pub struct NetFetcher {
    network: Network,
}

impl TxFetcher for NetFetcher {
    fn new(network: Network) -> Self {
        Self { network }
    }
    fn fetch_tx(&self, id: &TxId) -> Result<Vec<u8>> {
        let body = fetch(self.network, &format!("/tx/{}/hex", id))?;
        match from_hex_str(&body) {
            Err(_) => Err(NetworkError::ParsingError),
            Ok(bytes) => Ok(bytes),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tx::TxId;
    use crate::u256::U256;

    #[test]
    fn fetch_genesis() {
        NetFetcher::new(Network::Main)
            .fetch_tx(&TxId::from(U256::from_hex(
                "4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b",
            )))
            .unwrap();
    }
}
