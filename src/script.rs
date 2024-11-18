use crate::hashing::to_hex_str;
use crate::serialization::{slice_to_array, varint, Result, SerializationError};
use core::fmt;
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::ops::Range;

lazy_static! {
    static ref OPCODES: HashMap<u8, &'static str> = HashMap::from([
        // Constants
        (0x00, "OP_FALSE"),
        (0x4c, "OP_PUSHDATA1"),
        (0x4d, "OP_PUSHDATA2"),
        (0x4e, "OP_PUSHDATA4"),
        (0x4f, "OP_1NEGATE"),
        (0x51, "OP_TRUE"),
        (0x52, "OP_2"),
        (0x53, "OP_3"),
        (0x54, "OP_4"),
        (0x55, "OP_5"),
        (0x56, "OP_6"),
        (0x57, "OP_7"),
        (0x58, "OP_8"),
        (0x59, "OP_9"),
        (0x5a, "OP_10"),
        (0x5b, "OP_11"),
        (0x5c, "OP_12"),
        (0x5d, "OP_13"),
        (0x5e, "OP_14"),
        (0x5f, "OP_15"),
        (0x60, "OP_16"),
        // Flow control
        (0x61, "OP_NOP"),
        (0x63, "OP_IF"),
        (0x64, "OP_NOTIF"),
        (0x65, "OP_VERIF"),
        (0x67, "OP_ELSE"),
        (0x68, "OP_ENDIF"),
        (0x69, "OP_VERIFY"),
        (0x6a, "OP_RETURN"),
        // Stack
        (0x6b, "OP_TOALTSTACK"),
        (0x6c, "OP_FROMALTSTACK"),
        (0x6d, "OP_2DROP"),
        (0x6e, "OP_2DUP"),
        (0x6f, "OP_3DUP"),
        (0x70, "OP_2OVER"),
        (0x71, "OP_2ROT"),
        (0x72, "OP_2SWAP"),
        (0x73, "OP_IFDUP"),
        (0x74, "OP_DEPTH"),
        (0x75, "OP_DROP"),
        (0x76, "OP_DUP"),
        (0x77, "OP_NIP"),
        (0x78, "OP_OVER"),
        (0x79, "OP_PICK"),
        (0x7a, "OP_ROLL"),
        (0x7b, "OP_ROT"),
        (0x7c, "OP_SWAP"),
        (0x7d, "OP_TUCK"),
        // Splice
        (0x82, "OP_SIZE"),
        // Bitwise logic
        (0x87, "OP_EQUAL"),
        (0x88, "OP_EQUALVERIFY"),
        // Arithmetic
        (0x8b, "OP_1ADD"),
        (0x8c, "OP_1SUB"),
        // (0x8d, "OP_2MUL"),
        // (0x8e, "OP_2DIV"),
        (0x8f, "OP_NEGATE"),
        (0x90, "OP_ABS"),
        (0x91, "OP_NOT"),
        (0x92, "OP_0NOTEQUAL"),
        (0x93, "OP_ADD"),
        (0x94, "OP_SUB"),
        // (0x95, "OP_MUL"),
        // (0x96, "OP_DIV"),
        // (0x97, "OP_MOD"),
        // (0x98, "OP_LSHIFT"),
        // (0x99, "OP_RSHIFT"),
        (0x9a, "OP_BOOLAND"),
        (0x9b, "OP_BOOLOR"),
        (0x9c, "OP_NUMEQUAL"),
        (0x9d, "OP_NUMEQUALVERIFY"),
        (0x9e, "OP_NUMNOTEQUAL"),
        (0x9f, "OP_LESSTHAN"),
        (0xa0, "OP_GREATERTHAN"),
        (0xa1, "OP_LESSTHANOREQUAL"),
        (0xa2, "OP_GREATERTHANOREQUAL"),
        (0xa3, "OP_MIN"),
        (0xa4, "OP_MAX"),
        (0xa5, "OP_WITHIN"),
        // Crypto
        (0xa6, "OP_RIPEMD160"),
        (0xa7, "OP_SHA1"),
        (0xa8, "OP_SHA256"),
        (0xa9, "OP_HASH160"),
        (0xaa, "OP_HASH256"),
        (0xab, "OP_CODESEPARATOR"),
        (0xac, "OP_CHECKSIG"),
        (0xad, "OP_CHECKSIGVERIFY"),
        (0xae, "OP_CHECKMULTISIG"),
        (0xaf, "OP_CHECKMULTISIGVERIFY"),
        (0xba, "OP_CHECKSIGADD"),
        // Locktime
        (0xb1, "OP_CHECKLOCKTIMEVERIFY"),
        (0xb2, "OP_CHECKSEQUENCEVERIFY"),
        // Reserved words
        (0x50, "OP_RESERVED"),
        (0x62, "OP_VER"),
        (0x65, "OP_VERIF"),
        (0x66, "OP_VERNOTIF"),
        (0x89, "OP_RESERVED1"),
        (0x8a, "OP_RESERVED2"),
        (0xb0, "OP_NOP1"),
        (0xb3, "OP_NOP4"),
        (0xb4, "OP_NOP5"),
        (0xb5, "OP_NOP6"),
        (0xb6, "OP_NOP7"),
        (0xb7, "OP_NOP8"),
        (0xb8, "OP_NOP9"),
        (0xb9, "OP_NOP10"),
    ]);
}

enum Element {
    Op(u8),
    Data(u8, Vec<u8>),
}

impl Element {
    fn push_op_bytes(opcode: u8) -> Option<u64> {
        match opcode {
            0x4c => Some(1),
            0x4d => Some(2),
            0x4e => Some(4),
            _ => None,
        }
    }

    fn size_to_bytes(opcode: u8, value: usize) -> Option<Vec<u8>> {
        let result = match Self::push_op_bytes(opcode)? {
            1 => vec![value as u8],
            2 => (value as u16).to_le_bytes().to_vec(),
            4 => (value as u32).to_le_bytes().to_vec(),
            _ => return None,
        };
        Some(result)
    }
}

impl fmt::Display for Element {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Op(opcode) => write!(f, "{}", OPCODES.get(opcode).unwrap()),
            Self::Data(opcode, data) => write!(
                f,
                "PUSHDATA_{} <data, len {}>",
                Self::push_op_bytes(*opcode).unwrap(),
                data.len()
            ),
        }
    }
}

impl fmt::Debug for Element {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        <Self as fmt::Display>::fmt(self, f)
    }
}

pub struct Script {
    code: Vec<Element>,
}

impl Script {
    pub fn parse(data: &[u8]) -> Result<(Script, usize)> {
        let (length, offset) = varint::parse(data)?;
        let start = offset as usize;
        let end = start + length as usize;
        Ok((
            Script {
                code: Script::parse_code(&data[start..end])?,
            },
            end,
        ))
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut code: Vec<u8> = vec![];
        for element in &self.code {
            match element {
                Element::Op(opcode) => code.push(*opcode),
                Element::Data(opcode, data) => {
                    if *opcode == 0 {
                        code.push(data.len() as u8);
                    } else {
                        code.push(*opcode);
                        code.extend_from_slice(
                            &Element::size_to_bytes(*opcode, data.len()).unwrap(),
                        );
                    }
                    code.extend_from_slice(&data)
                }
            }
        }
        [varint::encode(code.len() as u64), code].concat()
    }

    fn parse_code(data: &[u8]) -> Result<Vec<Element>> {
        let mut code = vec![];
        let mut i = 0;
        loop {
            if let Some(&value) = data.get(i) {
                // Check for implicit push data op
                if value > 0 && value <= 75 {
                    let start = i + 1;
                    let end = start + value as usize;
                    code.push(Element::Data(
                        0,
                        Script::read_script_data(data, start..end)?,
                    ));
                    i = end;
                }
                // Check for explicit pushdata opcodes
                else if value > 75 && value <= 78 {
                    let len_word_start = i + 1;
                    let len_word_end = len_word_start
                        + match Element::push_op_bytes(value) {
                            Some(count) => count as usize,
                            None => return Err(SerializationError::InvalidState),
                        };

                    let len_bytes = Script::read_script_data(data, len_word_start..len_word_end)?;
                    let data_len = usize::from_le_bytes(slice_to_array(&len_bytes));
                    let data_start = len_word_end;
                    let data_end = data_start + data_len;
                    code.push(Element::Data(
                        value,
                        Script::read_script_data(data, data_start..data_end)?,
                    ));
                    i += data_len;
                }
                // Check for regular opcodes
                else {
                    match OPCODES.get(&value) {
                        Some(_) => code.push(Element::Op(value)),
                        None => return Err(SerializationError::InvalidOpcode(value)),
                    }
                    i += 1;
                }
            } else {
                break Ok(code);
            }
        }
    }

    fn read_script_data(data: &[u8], range: Range<usize>) -> Result<Vec<u8>> {
        match data.get(range) {
            Some(bytes) => Ok(bytes.to_vec()),
            None => Err(SerializationError::NotEnoughData),
        }
    }

    fn to_string(&self) -> String {
        let mut code: Vec<String> = vec![];
        for element in &self.code {
            match element {
                Element::Op(opcode) => code.push((*OPCODES.get(&opcode).unwrap()).to_string()),
                Element::Data(opcode, data) => {
                    if *opcode != 0 {
                        code.push((*OPCODES.get(&opcode).unwrap()).to_string())
                    }
                    code.push(to_hex_str(&data));
                }
            }
        }
        code.join(" ")
    }
}

impl fmt::Display for Script {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hex_literal::hex;

    #[test]
    fn script_parsing() {
        let code = hex!("04 55 93 59 87");
        assert_eq!(Script::parse(&code).unwrap().0.serialize(), code);

        let code = hex!("1976a914bc3b654dca7e56b04dca18f2566cdaf02e8d9ada88ac");
        assert_eq!(Script::parse(&code).unwrap().0.serialize(), code);
    }

    #[test]
    fn script_printing() {
        let code = hex!("04 55 93 59 87");
        assert_eq!(
            Script::parse(&code).unwrap().0.to_string(),
            "OP_5 OP_ADD OP_9 OP_EQUAL"
        );

        let code = hex!("1976a914bc3b654dca7e56b04dca18f2566cdaf02e8d9ada88ac");
        assert_eq!(
            Script::parse(&code).unwrap().0.to_string(),
            "OP_DUP OP_HASH160 bc3b654dca7e56b04dca18f2566cdaf02e8d9ada OP_EQUALVERIFY OP_CHECKSIG"
        );

        let code = hex!("1976a9141c4bc762dd5423e332166702cb75f40df79fea1288ac");
        assert_eq!(
            Script::parse(&code).unwrap().0.to_string(),
            "OP_DUP OP_HASH160 1c4bc762dd5423e332166702cb75f40df79fea12 OP_EQUALVERIFY OP_CHECKSIG"
        );
    }
}
