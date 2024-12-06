use crate::block::{BlockError, BlockHeader, Result as BlockResult};
use crate::definitions::Network;
use crate::hashing::{from_hex_str, to_hex_str, Hash};
use crate::tx::{Result as TxResult, Tx, TxError, TxFetcher, TxId, TxSerType};
use crate::u256::U256;
use base64::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use ureq;

#[derive(Debug)]
pub enum NetworkError {
    RequestError(ureq::Error),
    IoError(std::io::Error),
    ParsingError(serde_json::Error),
    InvalidData,
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

impl From<serde_json::Error> for NetworkError {
    fn from(error: serde_json::Error) -> Self {
        NetworkError::ParsingError(error)
    }
}

pub type Result<T> = std::result::Result<T, NetworkError>;

pub struct NetFetcher {
    network: Network,
    cache: HashMap<String, Tx>,
}

impl NetFetcher {
    pub fn new(network: Network) -> Self {
        Self {
            network,
            cache: HashMap::new(),
        }
    }

    fn get_host(network: Network) -> &'static str {
        match network {
            Network::Main => "https://blockstream.info/api",
            Network::Test => "https://blockstream.info/testnet/api",
        }
    }

    fn fetch(network: Network, path: &str) -> Result<String> {
        Ok(ureq::get(&format!("{}{}", Self::get_host(network), path))
            .call()?
            .into_string()?)
    }
}

impl TxFetcher for NetFetcher {
    fn fetch_tx(&mut self, id: &TxId) -> TxResult<Tx> {
        log::info!("Fetching tx from network: {}", id);
        if let Some(tx) = self.cache.get(&id.to_string()) {
            log::debug!("Tx found in cache");
            return Ok(tx.clone());
        }

        let body = Self::fetch(self.network, &format!("/tx/{}/hex", id))?;
        match from_hex_str(&body) {
            Err(_) => Err(TxError::NetworkError(NetworkError::InvalidData)),
            Ok(bytes) => {
                let tx = match Tx::parse(&bytes) {
                    Err(err) => return Err(err),
                    Ok(tx) => tx,
                };
                self.cache.insert(id.to_string(), tx.clone());
                Ok(tx)
            }
        }
    }
}

pub struct NodeClient {
    url: String,
    cookie: String,
    txs: HashMap<String, Tx>,
}

#[derive(Debug, Serialize, Deserialize)]
struct RPCResult<T> {
    result: T,
    error: Option<String>,
    id: String,
}

impl NodeClient {
    pub fn new(url: &str, cookie: &Path) -> Result<NodeClient> {
        let mut client = Self {
            url: url.to_string(),
            cookie: fs::read_to_string(cookie)?,
            txs: HashMap::new(),
        };

        #[derive(Debug, Deserialize)]
        struct ChainTip {
            height: u32,
            hash: String,
            branchlen: u32,
            status: String,
        }
        let body = client.call("getchaintips", json!([]))?;
        let data = match serde_json::from_str::<RPCResult<Vec<ChainTip>>>(&body) {
            Err(_) => return Ok(client),
            Ok(data) => data,
        };

        #[derive(Debug, Deserialize)]
        struct Block {
            previousblockhash: Option<String>,
            tx: Vec<String>,
        }
        let mut block_hash = data.result[0].hash.clone();
        loop {
            let body = client.call("getblock", json!([&block_hash]))?;
            let block = match serde_json::from_str::<RPCResult<Block>>(&body) {
                Err(err) => panic!("{}", err),
                Ok(data) => data.result,
            };
            if block.previousblockhash.is_none() {
                break;
            }
            for tx_hash in block.tx {
                let tx = client
                    .fetch_tx_from_block(
                        &Hash::from(U256::from_hex(&tx_hash)),
                        &Hash::from(U256::from_hex(&block_hash)),
                    )
                    .unwrap();
                client.txs.insert(tx_hash, tx);
            }
            block_hash = block.previousblockhash.unwrap();
        }

        Ok(client)
    }

    pub fn call(&self, method: &str, params: Value) -> Result<String> {
        let resp = ureq::post(&self.url)
            .set("Content-Type", "text/plain")
            .set(
                "Authorization",
                &std::format!("Basic {}", BASE64_STANDARD.encode(self.cookie.as_bytes())),
            )
            .send_string(
                &json!({
                     "jsonrpc": "1.0",
                     "id": "bitlib",
                     "method": method,
                     "params": params
                })
                .to_string(),
            )?
            .into_string()?;
        Ok(resp)
    }

    pub fn fetch_block_header(&self, id: &Hash) -> BlockResult<BlockHeader> {
        let body = self.call("getblockheader", json!([&id.to_string(), false]))?;
        let data: RPCResult<String> = match serde_json::from_str(&body) {
            Err(err) => return Err(BlockError::NetworkError(NetworkError::ParsingError(err))),
            Ok(data) => data,
        };
        match from_hex_str(&data.result) {
            Err(_) => Err(BlockError::NetworkError(NetworkError::InvalidData)),
            Ok(bytes) => {
                let tx = match BlockHeader::parse(&bytes) {
                    Err(err) => return Err(err),
                    Ok(tx) => tx,
                };
                Ok(tx)
            }
        }
    }

    pub fn fetch_tx_from_block(&self, id: &TxId, block_id: &Hash) -> TxResult<Tx> {
        log::info!("Fetching tx from node: {}", id);
        let body = self.call(
            "getrawtransaction",
            json!([&id.to_string(), false, &block_id.to_string()]),
        )?;
        let data: RPCResult<String> = match serde_json::from_str(&body) {
            Err(err) => return Err(TxError::NetworkError(NetworkError::ParsingError(err))),
            Ok(data) => data,
        };
        match from_hex_str(&data.result) {
            Err(_) => Err(TxError::NetworkError(NetworkError::InvalidData)),
            Ok(bytes) => {
                let tx = match Tx::parse(&bytes) {
                    Err(err) => return Err(err),
                    Ok(tx) => tx,
                };
                Ok(tx)
            }
        }
    }

    pub fn send_tx(&self, tx: &Tx) -> Result<String> {
        println!("{}", to_hex_str(tx.serialize(TxSerType::Full)));
        log::info!("Sending tx to node: {}", tx.id());
        let body = self.call(
            "sendrawtransaction",
            json!([to_hex_str(tx.serialize(TxSerType::Full))]),
        )?;
        match serde_json::from_str::<RPCResult<String>>(&body) {
            Err(err) => Err(NetworkError::ParsingError(err)),
            Ok(data) => Ok(data.result),
        }
    }
}

impl TxFetcher for NodeClient {
    fn fetch_tx(&mut self, id: &TxId) -> TxResult<Tx> {
        Ok(self.txs.get(&id.to_string()).unwrap().clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::u256::U256;
    use hex_literal::hex;

    #[test]
    fn fetch_genesis() {
        let mut fetcher = NetFetcher::new(Network::Main);
        let genesis_id = "4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b";
        let genesis = Tx::parse(&hex!("01000000010000000000000000000000000000000000000000000000000000000000000000ffffffff4d04ffff001d0104455468652054696d65732030332f4a616e2f32303039204368616e63656c6c6f72206f6e206272696e6b206f66207365636f6e64206261696c6f757420666f722062616e6b73ffffffff0100f2052a01000000434104678afdb0fe5548271967f1a67130b7105cd6a828e03909a67962e0ea1f61deb649f6bc3f4cef38c4f35504e51ec112de5c384df7ba0b8d578a4c702b6bf11d5fac00000000")).unwrap();

        assert_eq!(
            fetcher
                .fetch_tx(&TxId::from(U256::from_hex(genesis_id)))
                .unwrap(),
            genesis
        );

        assert_eq!(fetcher.cache.get(genesis_id), Some(&genesis));
    }
}
