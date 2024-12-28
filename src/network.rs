use crate::block::{BlockHeader, BlockId};
use crate::definitions::Network;
use crate::hashing::{hash256, to_hex_str};
use crate::merkle::{Bitfield, MerkleTree};
use crate::serialization::{slice_to_array, varint, varstr, SerializationError};
use crate::tx::TxId;
use crate::u256::U256;
use std::fmt;
use std::io::{self, Read};

#[derive(PartialEq, Debug)]
pub enum NetworkError {
    InvalidData,
    ParsingError(Option<SerializationError>),
    IoError(String),
}

impl From<SerializationError> for NetworkError {
    fn from(error: SerializationError) -> Self {
        NetworkError::ParsingError(Some(error))
    }
}

impl From<io::Error> for NetworkError {
    fn from(error: io::Error) -> Self {
        NetworkError::IoError(error.to_string())
    }
}

pub type Result<T> = std::result::Result<T, NetworkError>;

#[derive(PartialEq, Debug)]
pub struct Envelope {
    pub network: Network,
    pub command: [u8; 12],
    pub payload_length: u32,
    pub payload: Vec<u8>,
}

impl Envelope {
    pub fn new(network: Network, command: &str, payload: Vec<u8>) -> Result<Envelope> {
        let cmd = Self::str_to_command(command)?;
        Ok(Envelope {
            network,
            command: cmd.try_into().unwrap(),
            payload_length: payload.len() as u32,
            payload,
        })
    }

    fn get_magic_bytes(network: Network) -> [u8; 4] {
        match network {
            Network::Main => [0xf9, 0xbe, 0xb4, 0xd9],
            Network::Test => [0x0b, 0x11, 0x09, 0x07],
        }
    }

    fn detect_magic_bytes(data: &[u8]) -> Result<Network> {
        if data.len() < 4 {
            return Err(NetworkError::ParsingError(None));
        }

        if data[0..4] == Self::get_magic_bytes(Network::Main) {
            return Ok(Network::Main);
        } else if data[0..4] == Self::get_magic_bytes(Network::Test) {
            return Ok(Network::Test);
        }

        Err(NetworkError::ParsingError(None))
    }

    fn str_to_command(string: &str) -> Result<Vec<u8>> {
        if string.len() > 12 {
            return Err(NetworkError::InvalidData);
        }

        let mut cmd: [u8; 12] = [0; 12];
        let bytes = string.as_bytes();
        for i in 0..bytes.len() {
            cmd[i] = bytes[i];
        }
        Ok(cmd.to_vec())
    }

    fn command_to_str(cmd: &[u8]) -> String {
        let mut string = String::from_utf8(cmd.to_vec()).unwrap();
        if let Some(i) = string.find('\0') {
            string.truncate(i);
        }
        string
    }

    pub fn command(&self) -> String {
        Self::command_to_str(&self.command)
    }

    fn checksum(&self) -> [u8; 4] {
        hash256(&self.payload)[0..4].try_into().unwrap()
    }

    pub fn parse<Reader: Read>(stream: &mut Reader) -> Result<Envelope> {
        log::debug!("Parsing Envelope...");
        log::debug!("Get first batch of data...");
        let mut data: [u8; 24] = [0; 24];
        stream.read_exact(&mut data[0..24])?;

        log::debug!("data {:?}", to_hex_str(&data));

        let network = Self::detect_magic_bytes(&data)?;
        log::debug!("Network: {}", network);
        let mut command = [0; 12];
        command.clone_from_slice(&data[4..16]);
        log::debug!("Command: {}", Self::command_to_str(&command));
        let payload_length = u32::from_le_bytes(slice_to_array(&data[16..20]));
        let mut payload_checksum = [0; 4];
        payload_checksum.clone_from_slice(&data[20..24]);

        let mut data = vec![0; payload_length as usize];
        stream.read_exact(&mut data)?;

        let payload = data;

        if payload_checksum.as_slice() != &hash256(&payload)[0..4] {
            return Err(NetworkError::InvalidData);
        }

        Ok(Envelope {
            network,
            command,
            payload_length,
            payload,
        })
    }

    pub fn serialize(&self) -> Vec<u8> {
        let network = Self::get_magic_bytes(self.network);
        let payload_length = self.payload_length.to_le_bytes();
        let payload_checksum = self.checksum();

        [
            network.as_slice(),
            self.command.as_slice(),
            payload_length.as_slice(),
            payload_checksum.as_slice(),
            self.payload.as_slice(),
        ]
        .concat()
    }
}

impl fmt::Display for Envelope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Envelope:\nnetwork {}\ncommand: {}\npayload_length: {}\npayload: {}",
            self.network,
            self.command(),
            self.payload_length,
            if self.payload.len() == 0 {
                "<empty>".to_owned()
            } else {
                to_hex_str(&self.payload)
            },
        )
    }
}

#[derive(PartialEq, Debug)]
pub struct NetAddr {
    pub time: Option<u32>,
    pub service: u64,
    pub ip: [u8; 16],
    pub port: u16,
}

impl NetAddr {
    pub fn parse<Reader: Read>(stream: &mut Reader, exclude_time: bool) -> Result<NetAddr> {
        let mut data = [0; 30];
        let end = if exclude_time { 26 } else { 30 };
        stream.read_exact(&mut data[0..end])?;

        let (time, offset) = if exclude_time {
            (None, 0)
        } else {
            (Some(u32::from_le_bytes(slice_to_array(&data[0..4]))), 4)
        };
        Ok(NetAddr {
            time,
            service: u64::from_le_bytes(slice_to_array(&data[offset..offset + 8])),
            ip: slice_to_array(&data[offset + 8..offset + 8 + 16]),
            port: u16::from_be_bytes(slice_to_array(&data[offset + 8 + 16..offset + 8 + 16 + 2])),
        })
    }

    pub fn serialize(&self) -> Vec<u8> {
        let time = match self.time {
            Some(t) => t.to_le_bytes().to_vec(),
            None => vec![],
        };
        let service = self.service.to_le_bytes();
        let port = self.port.to_be_bytes();

        [
            time.as_slice(),
            service.as_slice(),
            self.ip.as_slice(),
            port.as_slice(),
        ]
        .concat()
    }
}

impl fmt::Display for NetAddr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Net Address\n")?;

        if let Some(time) = self.time {
            write!(f, "time: {}", time)?;
        }

        write!(
            f,
            "service: {:032b}\nip: {:?}\nport: {}",
            self.service, self.ip, self.port,
        )
    }
}

pub struct VersionMessage {
    pub version: i32,
    pub services: u64,
    pub timestamp: i64,
    pub addr_recv: NetAddr,
    pub addr_from: NetAddr,
    pub nonce: u64,
    pub user_agent: String,
    pub start_height: i32,
    pub relay: Option<bool>,
}

impl VersionMessage {
    pub fn parse<Reader: Read>(stream: &mut Reader) -> Result<VersionMessage> {
        let mut data = [0; 20];
        stream.read_exact(&mut data[0..20])?;

        let version = i32::from_le_bytes(slice_to_array(&data[0..4]));
        let services = u64::from_le_bytes(slice_to_array(&data[4..12]));
        let timestamp = i64::from_le_bytes(slice_to_array(&data[12..20]));

        let addr_recv = NetAddr::parse(stream, true)?;
        let addr_from = NetAddr::parse(stream, true)?;

        stream.read_exact(&mut data[0..8])?;
        let nonce = u64::from_le_bytes(slice_to_array(&data[0..8]));

        let user_agent = varstr::parse(stream)?;

        stream.read_exact(&mut data[0..4])?;
        let start_height = i32::from_le_bytes(slice_to_array(&data[0..4]));

        let relay = if version > 70001 {
            stream.read_exact(&mut data[0..1])?;
            Some(data[0] == 1)
        } else {
            None
        };

        Ok(VersionMessage {
            version,
            services,
            timestamp,
            addr_recv,
            addr_from,
            nonce,
            user_agent,
            start_height,
            relay,
        })
    }

    pub fn serialize(&self) -> Vec<u8> {
        let version = self.version.to_le_bytes();
        let services = self.services.to_le_bytes();
        let timestamp = self.timestamp.to_le_bytes();
        let addr_recv = self.addr_recv.serialize();
        let addr_from = self.addr_from.serialize();
        let nonce = self.nonce.to_le_bytes();
        let user_agent = varstr::encode(&self.user_agent);
        let start_height = self.start_height.to_le_bytes();
        let relay = match self.relay {
            None => vec![],
            Some(value) => {
                if value {
                    vec![1]
                } else {
                    vec![0]
                }
            }
        };

        [
            version.as_slice(),
            services.as_slice(),
            timestamp.as_slice(),
            addr_recv.as_slice(),
            addr_from.as_slice(),
            nonce.as_slice(),
            user_agent.as_slice(),
            start_height.as_slice(),
            relay.as_slice(),
        ]
        .concat()
    }
}

impl fmt::Display for VersionMessage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Version Message:\nversion {}\nservices: {:032b}\ntimestamp: {}\naddr recv: {}\naddr from: {}\nuser agent: {}\nstart height: {}",
            self.version,
            self.services,
            self.timestamp,
            self.addr_recv,
            self.addr_from,
            self.user_agent,
            self.start_height,
        )
    }
}

pub struct GetHeadersMessage {
    pub version: i32,
    pub num_hashes: u64,
    pub start_block: BlockId,
    pub end_block: BlockId,
}

impl GetHeadersMessage {
    pub fn new(
        version: i32,
        num_hashes: u64,
        start_block: BlockId,
        end_block: BlockId,
    ) -> GetHeadersMessage {
        GetHeadersMessage {
            version,
            num_hashes,
            start_block,
            end_block,
        }
    }

    pub fn parse<Reader: Read>(stream: &mut Reader) -> Result<GetHeadersMessage> {
        let mut data = [0; 32];
        stream.read_exact(&mut data[0..4])?;

        let version = i32::from_le_bytes(slice_to_array(&data[0..4]));
        let num_hashes = varint::parse_stream(stream)?;

        stream.read_exact(&mut data[0..])?;
        let start_block = BlockId::new(U256::from_little_endian(&data));
        stream.read_exact(&mut data[0..])?;
        let end_block = BlockId::new(U256::from_little_endian(&data));

        Ok(GetHeadersMessage {
            version,
            num_hashes,
            start_block,
            end_block,
        })
    }

    pub fn serialize(&self) -> Vec<u8> {
        let version = self.version.to_le_bytes();
        let num_hashes = varint::encode(self.num_hashes);
        let start_block = self.start_block.to_little_endian();
        let end_block = self.end_block.to_little_endian();

        [
            version.as_slice(),
            num_hashes.as_slice(),
            start_block.as_slice(),
            end_block.as_slice(),
        ]
        .concat()
    }
}

pub struct HeadersMessage {
    pub headers: Vec<BlockHeader>,
}

impl HeadersMessage {
    pub fn parse<Reader: Read>(stream: &mut Reader) -> Result<HeadersMessage> {
        let num_headers = varint::parse_stream(stream)? as usize;

        let mut headers = vec![];
        let mut data = [0; 81];
        for _ in 0..num_headers {
            stream.read_exact(&mut data)?;

            // The number of transactions must be zero
            if data[80] != 0 {
                return Err(NetworkError::ParsingError(None));
            }

            let block = match BlockHeader::parse(&data) {
                Err(_) => return Err(NetworkError::ParsingError(None)),
                Ok(block) => block,
            };
            headers.push(block);
        }

        Ok(HeadersMessage { headers })
    }
}

pub struct MerkleBlockMessage {
    pub header: BlockHeader,
    total_txs: u32,
    hashes: Vec<TxId>,
    flags: Vec<u8>,
}

impl MerkleBlockMessage {
    pub fn parse<Reader: Read>(stream: &mut Reader) -> Result<MerkleBlockMessage> {
        let mut data = [0; 80];
        stream.read_exact(&mut data)?;
        let header = match BlockHeader::parse(&data) {
            Err(_) => return Err(NetworkError::ParsingError(None)),
            Ok(block) => block,
        };

        stream.read_exact(&mut data[0..4])?;

        let total_txs = u32::from_le_bytes(slice_to_array(&data[0..4]));
        let num_hashes = varint::parse_stream(stream)?;
        let mut hashes = vec![];
        for _ in 0..num_hashes {
            stream.read_exact(&mut data[0..32])?;
            let hash = TxId::new(U256::from_little_endian(&data[0..32]));
            hashes.push(hash);
        }

        let flag_bytes = varint::parse_stream(stream)? as usize;
        stream.read_exact(&mut data[0..flag_bytes])?;
        let flags = data[0..flag_bytes].to_vec();

        Ok(MerkleBlockMessage {
            header,
            total_txs,
            hashes,
            flags,
        })
    }

    pub fn is_valid(&self) -> bool {
        let flags = Bitfield::from_little_endian(&self.flags);
        let hashes: Vec<TxId> = self.hashes.iter().map(|h| h.reverse()).collect();
        let tree = MerkleTree::new(self.total_txs as usize);
        let root = tree.hash(&flags, &hashes);
        root.reverse() == self.header.merkle_root
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hex_literal::hex;

    fn init_logging() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    #[test]
    fn magic_bytes() {
        assert!(Envelope::detect_magic_bytes(&hex!("ab1221ab")).is_err());
        assert!(Envelope::detect_magic_bytes(&hex!("f9beb4aa")).is_err());
        assert!(Envelope::detect_magic_bytes(&hex!("0b110911")).is_err());
        assert_eq!(
            Envelope::detect_magic_bytes(&hex!("f9beb4d9")),
            Ok(Network::Main)
        );
        assert_eq!(
            Envelope::detect_magic_bytes(&hex!("0b110907")),
            Ok(Network::Test)
        );
    }

    #[test]
    fn envelope_serialization() {
        let bytes = hex!("f9beb4d976657261636b000000000000000000005df6e0e2");
        let envelope = Envelope::parse(&mut bytes.as_slice()).unwrap();
        assert_eq!(envelope.network, Network::Main);
        assert_eq!(envelope.command(), "verack".to_owned());
        assert_eq!(envelope.payload_length, 0);
        assert_eq!(envelope.payload.len(), 0);

        let bytes = hex!("f9beb4d976657261636b000000000000000000004df6e0e2");
        let result = Envelope::parse(&mut bytes.as_slice());
        assert_eq!(result, Err(NetworkError::InvalidData));

        let bytes = hex!("f9beb4d976657261636b000000000000000000005df6e0e2");
        let envelope = Envelope::parse(&mut bytes.as_slice()).unwrap();
        assert_eq!(envelope.serialize(), bytes);
    }

    #[test]
    fn version_message() {
        let bytes = hex!(
            "62ea0000
             0100000000000000
             11b2d05000000000
             010000000000000000000000000000000000ffff000000000000
             010000000000000000000000000000000000ffff000000000000
             3b2eb35d8ce61765
             0f2f5361746f7368693a302e372e322f
             c03e0300"
        );
        let message = VersionMessage::parse(&mut bytes.as_slice()).unwrap();
        assert_eq!(message.version, 60002);
        assert_eq!(message.services, 1);
        assert_eq!(message.timestamp, 1355854353);
        assert_eq!(
            message.addr_recv,
            NetAddr {
                time: None,
                service: 1,
                ip: hex!("00000000000000000000ffff00000000"),
                port: 0,
            }
        );
        assert_eq!(
            message.addr_from,
            NetAddr {
                time: None,
                service: 1,
                ip: hex!("00000000000000000000ffff00000000"),
                port: 0,
            }
        );
        assert_eq!(message.nonce, 0x6517e68c5db32e3b);
        assert_eq!(message.user_agent, "/Satoshi:0.7.2/");
        assert_eq!(message.start_height, 212672);
        assert_eq!(message.relay, None);

        assert_eq!(message.serialize(), bytes);
    }

    #[test]
    fn get_headers_message() {
        let bytes = hex!(
            "7f11010001
            a35bd0ca2f4a88c4eda6d213e2378a5758dfcd6af43712000000000000000000
            0000000000000000000000000000000000000000000000000000000000000000"
        );
        let message = GetHeadersMessage::parse(&mut bytes.as_slice()).unwrap();
        assert_eq!(message.version, 70015);
        assert_eq!(message.num_hashes, 1);
        assert_eq!(
            message.start_block,
            BlockId::new(U256::from_hex(
                "0000000000000000001237f46acddf58578a37e213d2a6edc4884a2fcad05ba3"
            ))
        );
        assert_eq!(message.end_block, BlockId::new(U256::default()));

        assert_eq!(message.serialize(), bytes);
    }

    #[test]
    fn headers_message() {
        let bytes = hex!(
            "02
            020000208ec39428b17323fa0ddec8e887b4a7c53b8c0a0a220cfd0000000000000000005b0750fce0a889502d40508d39576821155e9c9e3f5c3157f961db38fd8b25be1e77a759e93c0118a4ffd71d00
            020000208ec39428b17323fa0ddec8e887b4a7c53b8c0a0a220cfd0000000000000000005b0750fce0a889502d40508d39576821155e9c9e3f5c3157f961db38fd8b25be1e77a759e93c0118a4ffd71d00"
        );
        let message = HeadersMessage::parse(&mut bytes.as_slice()).unwrap();
        assert_eq!(message.headers.len(), 2);
        assert_eq!(message.headers[0], message.headers[1]);
        let header = &message.headers[0];
        assert_eq!(header.version, 0x20000002);
        assert_eq!(
            header.prev_block,
            BlockId::from(U256::from_hex(
                "000000000000000000fd0c220a0a8c3bc5a7b487e8c8de0dfa2373b12894c38e"
            ))
        );
    }

    #[test]
    fn merkle_block_message() {
        let bytes = hex!(
            "00000020df3b053dc46f162a9b00c7f0d5124e2676d47bbe7c5d0793a500000000000000ef445fef2ed495c275892206ca533e7411907971013ab83e3b47bd0d692d14d4dc7c835b67d8001ac157e670bf0d00000aba412a0d1480e370173072c9562becffe87aa661c1e4a6dbc305d38ec5dc088a7cf92e6458aca7b32edae818f9c2c98c37e06bf72ae0ce80649a38655ee1e27d34d9421d940b16732f24b94023e9d572a7f9ab8023434a4feb532d2adfc8c2c2158785d1bd04eb99df2e86c54bc13e139862897217400def5d72c280222c4cbaee7261831e1550dbb8fa82853e9fe506fc5fda3f7b919d8fe74b6282f92763cef8e625f977af7c8619c32a369b832bc2d051ecd9c73c51e76370ceabd4f25097c256597fa898d404ed53425de608ac6bfe426f6e2bb457f1c554866eb69dcb8d6bf6f880e9a59b3cd053e6c7060eeacaacf4dac6697dac20e4bd3f38a2ea2543d1ab7953e3430790a9f81e1c67f5b58c825acf46bd02848384eebe9af917274cdfbb1a28a5d58a23a17977def0de10d644258d9c54f886d47d293a411cb6226103b55635"
        );
        let message = MerkleBlockMessage::parse(&mut bytes.as_slice()).unwrap();
        assert_eq!(message.header.version, 0x20000000);
        assert_eq!(
            message.header.merkle_root,
            U256::from_little_endian(&hex!(
                "ef445fef2ed495c275892206ca533e7411907971013ab83e3b47bd0d692d14d4"
            ))
            .into()
        );
        assert_eq!(
            message.header.prev_block,
            U256::from_little_endian(&hex!(
                "df3b053dc46f162a9b00c7f0d5124e2676d47bbe7c5d0793a500000000000000"
            ))
            .into()
        );
        assert_eq!(
            message.header.timestamp,
            u32::from_le_bytes(hex!("dc7c835b"))
        );
        assert_eq!(message.header.bits, hex!("67d8001a"));
        assert_eq!(message.header.nonce, hex!("c157e670"));

        assert_eq!(message.total_txs, u32::from_le_bytes(hex!("bf0d0000")));

        let hashes: [TxId; 10] = [
            U256::from_little_endian(&hex!(
                "ba412a0d1480e370173072c9562becffe87aa661c1e4a6dbc305d38ec5dc088a"
            ))
            .into(),
            U256::from_little_endian(&hex!(
                "7cf92e6458aca7b32edae818f9c2c98c37e06bf72ae0ce80649a38655ee1e27d"
            ))
            .into(),
            U256::from_little_endian(&hex!(
                "34d9421d940b16732f24b94023e9d572a7f9ab8023434a4feb532d2adfc8c2c2"
            ))
            .into(),
            U256::from_little_endian(&hex!(
                "158785d1bd04eb99df2e86c54bc13e139862897217400def5d72c280222c4cba"
            ))
            .into(),
            U256::from_little_endian(&hex!(
                "ee7261831e1550dbb8fa82853e9fe506fc5fda3f7b919d8fe74b6282f92763ce"
            ))
            .into(),
            U256::from_little_endian(&hex!(
                "f8e625f977af7c8619c32a369b832bc2d051ecd9c73c51e76370ceabd4f25097"
            ))
            .into(),
            U256::from_little_endian(&hex!(
                "c256597fa898d404ed53425de608ac6bfe426f6e2bb457f1c554866eb69dcb8d"
            ))
            .into(),
            U256::from_little_endian(&hex!(
                "6bf6f880e9a59b3cd053e6c7060eeacaacf4dac6697dac20e4bd3f38a2ea2543"
            ))
            .into(),
            U256::from_little_endian(&hex!(
                "d1ab7953e3430790a9f81e1c67f5b58c825acf46bd02848384eebe9af917274c"
            ))
            .into(),
            U256::from_little_endian(&hex!(
                "dfbb1a28a5d58a23a17977def0de10d644258d9c54f886d47d293a411cb62261"
            ))
            .into(),
        ];

        assert_eq!(message.hashes, hashes);
        assert_eq!(message.flags, hex!("b55635").to_vec());

        assert!(message.is_valid());
    }
}
