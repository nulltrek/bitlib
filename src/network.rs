use crate::definitions::Network;
use crate::hashing::from_hex_str;
use crate::tx::TxId;
use std::collections::HashMap;
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
    fn fetch_tx(&mut self, id: &TxId) -> Result<Vec<u8>>;
}

pub struct NetFetcher {
    network: Network,
    cache: HashMap<String, Vec<u8>>,
}

impl TxFetcher for NetFetcher {
    fn new(network: Network) -> Self {
        Self {
            network,
            cache: HashMap::new(),
        }
    }
    fn fetch_tx(&mut self, id: &TxId) -> Result<Vec<u8>> {
        if let Some(tx) = self.cache.get(&id.to_string()) {
            return Ok(tx.clone());
        }

        let body = fetch(self.network, &format!("/tx/{}/hex", id))?;
        match from_hex_str(&body) {
            Err(_) => Err(NetworkError::ParsingError),
            Ok(bytes) => {
                self.cache.insert(id.to_string(), bytes.clone());
                Ok(bytes)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tx::TxId;
    use crate::u256::U256;
    use hex_literal::hex;

    #[test]
    fn fetch_genesis() {
        let mut fetcher = NetFetcher::new(Network::Main);
        let genesis_id = "4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b";
        let genesis = hex!("01000000010000000000000000000000000000000000000000000000000000000000000000ffffffff4d04ffff001d0104455468652054696d65732030332f4a616e2f32303039204368616e63656c6c6f72206f6e206272696e6b206f66207365636f6e64206261696c6f757420666f722062616e6b73ffffffff0100f2052a01000000434104678afdb0fe5548271967f1a67130b7105cd6a828e03909a67962e0ea1f61deb649f6bc3f4cef38c4f35504e51ec112de5c384df7ba0b8d578a4c702b6bf11d5fac00000000");

        assert_eq!(
            fetcher
                .fetch_tx(&TxId::from(U256::from_hex(genesis_id,)))
                .unwrap(),
            genesis
        );

        assert_eq!(
            fetcher.cache.get(genesis_id),
            Some(genesis.to_vec().as_ref())
        );
    }
}
