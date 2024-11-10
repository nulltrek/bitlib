use crate::serialization::{varint, Result, SerializationError};
use core::fmt;

pub struct Script {
    data: Vec<u8>,
}

impl Script {
    pub fn parse(data: &[u8]) -> Result<(Script, usize)> {
        if data.as_ref().len() < 5 {
            return Err(SerializationError::NotEnoughData);
        }
        let (length, offset) = varint::parse(data)?;
        let start = offset as usize;
        let end = start + length as usize;
        Ok((
            Script {
                data: data[start..end].to_vec(),
            },
            end,
        ))
    }

    pub fn serialize(&self) -> Vec<u8> {
        [varint::encode(self.data.len() as u64), self.data.clone()].concat()
    }
}

impl fmt::Display for Script {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<script, len: {}>", self.data.len())
    }
}
