use crate::hashing::{to_hex_str, Hash};
use crate::serialization::{slice_to_array, varint, Result, SerializationError};
use core::fmt;
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::ops::Range;

type OpFn = fn(&mut Stack) -> bool;

lazy_static! {
    // let ops = Vec<(u8, (&'static str, fn(&mut Stack) -> bool))> ;
    static ref OPCODES: HashMap<u8, (&'static str, OpFn)> = HashMap::from([
        // Constants
        (0x00_u8, ("OP_FALSE", op::push_0 as OpFn)),
        (0x4c, ("OP_PUSHDATA1", op::nop as OpFn)),
        (0x4d, ("OP_PUSHDATA2", op::nop as OpFn)),
        (0x4e, ("OP_PUSHDATA4", op::nop as OpFn)),
        (0x4f, ("OP_1NEGATE", op::push_1_negate as OpFn)),
        (0x51, ("OP_TRUE", op::push_1 as OpFn)),
        (0x52, ("OP_2", op::push_2 as OpFn)),
        (0x53, ("OP_3", op::push_3 as OpFn)),
        (0x54, ("OP_4", op::push_4 as OpFn)),
        (0x55, ("OP_5", op::push_5 as OpFn)),
        (0x56, ("OP_6", op::push_6 as OpFn)),
        (0x57, ("OP_7", op::push_7 as OpFn)),
        (0x58, ("OP_8", op::push_8 as OpFn)),
        (0x59, ("OP_9", op::push_9 as OpFn)),
        (0x5a, ("OP_10", op::push_10 as OpFn)),
        (0x5b, ("OP_11", op::push_11 as OpFn)),
        (0x5c, ("OP_12", op::push_12 as OpFn)),
        (0x5d, ("OP_13", op::push_13 as OpFn)),
        (0x5e, ("OP_14", op::push_14 as OpFn)),
        (0x5f, ("OP_15", op::push_15 as OpFn)),
        (0x60, ("OP_16", op::push_16 as OpFn)),
        // Flow control
        (0x61, ("OP_NOP", op::nop as OpFn)),
        (0x63, ("OP_IF", op::unimplemented as OpFn)),
        (0x64, ("OP_NOTIF", op::unimplemented as OpFn)),
        (0x65, ("OP_VERIF", op::unimplemented as OpFn)),
        (0x67, ("OP_ELSE", op::unimplemented as OpFn)),
        (0x68, ("OP_ENDIF", op::unimplemented as OpFn)),
        (0x69, ("OP_VERIFY", op::verify as OpFn)),
        (0x6a, ("OP_RETURN", op::ret as OpFn)),
        // Stack
        (0x6b, ("OP_TOALTSTACK", op::to_alt_stack as OpFn)),
        (0x6c, ("OP_FROMALTSTACK", op::from_alt_stack as OpFn)),
        (0x6d, ("OP_2DROP", op::drop_2 as OpFn)),
        (0x6e, ("OP_2DUP", op::dup_2 as OpFn)),
        (0x6f, ("OP_3DUP", op::dup_3 as OpFn)),
        (0x70, ("OP_2OVER", op::over_2 as OpFn)),
        (0x71, ("OP_2ROT", op::rot_2 as OpFn)),
        (0x72, ("OP_2SWAP", op::swap_2 as OpFn)),
        (0x73, ("OP_IFDUP", op::if_dup as OpFn)),
        (0x74, ("OP_DEPTH", op::depth as OpFn)),
        (0x75, ("OP_DROP", op::drop as OpFn)),
        (0x76, ("OP_DUP", op::dup as OpFn)),
        (0x77, ("OP_NIP", op::nip as OpFn)),
        (0x78, ("OP_OVER", op::over as OpFn)),
        (0x79, ("OP_PICK", op::pick as OpFn)),
        (0x7a, ("OP_ROLL", op::roll as OpFn)),
        (0x7b, ("OP_ROT", op::rot as OpFn)),
        (0x7c, ("OP_SWAP", op::swap as OpFn)),
        (0x7d, ("OP_TUCK", op::tuck as OpFn)),
        // Splice
        (0x82, ("OP_SIZE", op::size as OpFn)),
        // Bitwise logic
        (0x87, ("OP_EQUAL", op::equal as OpFn)),
        (0x88, ("OP_EQUALVERIFY", op::equal_verify as OpFn)),
        // Arithmetic
        (0x8b, ("OP_1ADD", op::add_1 as OpFn)),
        (0x8c, ("OP_1SUB", op::sub_1 as OpFn)),
        // (0x8d, ("OP_2MUL", op::nop as OpFn)),
        // (0x8e, ("OP_2DIV", op::nop as OpFn)),
        (0x8f, ("OP_NEGATE", op::negate as OpFn)),
        (0x90, ("OP_ABS", op::abs as OpFn)),
        (0x91, ("OP_NOT", op::not as OpFn)),
        (0x92, ("OP_0NOTEQUAL", op::not_equal_0 as OpFn)),
        (0x93, ("OP_ADD", op::add as OpFn)),
        (0x94, ("OP_SUB", op::sub as OpFn)),
        // (0x95, ("OP_MUL", op::nop as OpFn)),
        // (0x96, ("OP_DIV", op::nop as OpFn)),
        // (0x97, ("OP_MOD", op::nop as OpFn)),
        // (0x98, ("OP_LSHIFT", op::nop as OpFn)),
        // (0x99, ("OP_RSHIFT", op::nop as OpFn)),
        (0x9a, ("OP_BOOLAND", op::bool_and as OpFn)),
        (0x9b, ("OP_BOOLOR", op::bool_or as OpFn)),
        (0x9c, ("OP_NUMEQUAL", op::num_equal as OpFn)),
        (0x9d, ("OP_NUMEQUALVERIFY", op::num_equal_verify as OpFn)),
        (0x9e, ("OP_NUMNOTEQUAL", op::num_not_equal as OpFn)),
        (0x9f, ("OP_LESSTHAN", op::less_than as OpFn)),
        (0xa0, ("OP_GREATERTHAN", op::greater_than as OpFn)),
        (0xa1, ("OP_LESSTHANOREQUAL", op::less_than_or_equal as OpFn)),
        (0xa2, ("OP_GREATERTHANOREQUAL", op::greater_than_or_equal as OpFn)),
        (0xa3, ("OP_MIN", op::min as OpFn)),
        (0xa4, ("OP_MAX", op::max as OpFn)),
        (0xa5, ("OP_WITHIN", op::within as OpFn)),
        // Crypto
        (0xa6, ("OP_RIPEMD160", op::ripemd160 as OpFn)),
        (0xa7, ("OP_SHA1", op::nop as OpFn)),
        (0xa8, ("OP_SHA256", op::sha256 as OpFn)),
        (0xa9, ("OP_HASH160", op::hash160 as OpFn)),
        (0xaa, ("OP_HASH256", op::hash256 as OpFn)),
        (0xab, ("OP_CODESEPARATOR", op::nop as OpFn)),
        (0xac, ("OP_CHECKSIG", op::check_sig as OpFn)),
        (0xad, ("OP_CHECKSIGVERIFY", op::check_sig_verify as OpFn)),
        (0xae, ("OP_CHECKMULTISIG", op::unimplemented as OpFn)),
        (0xaf, ("OP_CHECKMULTISIGVERIFY", op::unimplemented as OpFn)),
        (0xba, ("OP_CHECKSIGADD", op::unimplemented as OpFn)),
        // Locktime
        (0xb1, ("OP_CHECKLOCKTIMEVERIFY", op::unimplemented as OpFn)),
        (0xb2, ("OP_CHECKSEQUENCEVERIFY", op::unimplemented as OpFn)),
        // Reserved words
        (0x50, ("OP_RESERVED", op::nop as OpFn)),
        (0x62, ("OP_VER", op::nop as OpFn)),
        (0x65, ("OP_VERIF", op::nop as OpFn)),
        (0x66, ("OP_VERNOTIF", op::nop as OpFn)),
        (0x89, ("OP_RESERVED1", op::nop as OpFn)),
        (0x8a, ("OP_RESERVED2", op::nop as OpFn)),
        (0xb0, ("OP_NOP1", op::nop as OpFn)),
        (0xb3, ("OP_NOP4", op::nop as OpFn)),
        (0xb4, ("OP_NOP5", op::nop as OpFn)),
        (0xb5, ("OP_NOP6", op::nop as OpFn)),
        (0xb6, ("OP_NOP7", op::nop as OpFn)),
        (0xb7, ("OP_NOP8", op::nop as OpFn)),
        (0xb8, ("OP_NOP9", op::nop as OpFn)),
        (0xb9, ("OP_NOP10", op::nop as OpFn)),
    ]);
}

#[derive(Clone)]
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
            Self::Op(opcode) => write!(f, "{}", OPCODES.get(opcode).unwrap().0),
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

#[derive(Clone)]
pub struct Script {
    code: Vec<Element>,
}

impl Default for Script {
    fn default() -> Script {
        Script { code: vec![] }
    }
}

impl Script {
    pub fn parse(data: &[u8]) -> Result<(Script, usize)> {
        let (length, offset) = varint::parse(data)?;
        let start = offset as usize;
        let end = start + length as usize;
        log::info!("Parsing script (size: {} bytes)...", length);
        Ok((
            Script {
                code: Script::parse_code(&data[start..end])?,
            },
            end,
        ))
    }

    pub fn serialize(&self) -> Vec<u8> {
        if self.code.len() == 0 {
            // Blank script is a zeroed byte
            return vec![0x00];
        }

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
                    log::trace!("Push {} bytes", value);
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
                Element::Op(opcode) => code.push((*OPCODES.get(&opcode).unwrap().0).to_string()),
                Element::Data(opcode, data) => {
                    if *opcode != 0 {
                        code.push((*OPCODES.get(&opcode).unwrap().0).to_string())
                    }
                    code.push(format!("{}", to_hex_str(&data)));
                }
            }
        }
        code.join(" ")
    }
}

impl fmt::Display for Script {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.to_string())
    }
}

struct Stack {
    main: Vec<Vec<u8>>,
    alt: Vec<Vec<u8>>,
    sig_hash: Hash,
}

impl Stack {
    fn new(sig_hash: Hash) -> Stack {
        Stack {
            main: vec![],
            alt: vec![],
            sig_hash,
        }
    }
}

fn run(script: &Script, stack: &mut Stack) -> bool {
    for op in &script.code {
        let success = match op {
            Element::Op(opcode) => {
                let op = OPCODES.get(&opcode).unwrap();
                let result = op.1(stack);
                log::trace!("Op: {}; Result: {}", op.0, result);
                result
            }
            Element::Data(_, data) => {
                log::trace!("Op: push data; Length: {}", data.len());
                stack.main.push(data.clone());
                true
            }
        };
        if !success {
            return false;
        }
    }
    return true;
}

pub fn evaluate(sig_hash: &Hash, script_pubkey: &Script, script_sig: &Script) -> bool {
    log::debug!("Evaluating scripts:");
    log::debug!("Sig: {}", script_sig);
    log::debug!("Pubkey: {}", script_pubkey);
    let mut stack = Stack::new(sig_hash.clone());
    let success = run(script_sig, &mut stack);
    if !success {
        return false;
    }

    let success = run(script_pubkey, &mut stack);
    if !success {
        return false;
    }

    match stack.main.pop() {
        None => false,
        Some(value) => op::decode_num(value.as_slice()) != 0,
    }
}

mod op {
    use crate::hashing;

    use super::*;

    fn encode_num(num: i32) -> Vec<u8> {
        if num == 0 {
            return vec![];
        }

        let mut abs = num.unsigned_abs();
        let negative = num < 0;
        let mut result = vec![];
        while abs > 0 {
            result.push((abs & 0xff) as u8);
            abs >>= 8;
        }

        let last = result.last_mut().unwrap();
        if *last & 0x80 > 0 {
            if negative {
                result.push(0x80);
            } else {
                result.push(0x00);
            }
        } else if negative {
            *last |= 0x80;
        }
        return result;
    }

    pub fn decode_num(data: &[u8]) -> i32 {
        if data.is_empty() {
            return 0;
        }

        let big_endian: Vec<u8> = data.iter().copied().rev().collect();
        let (negative, mut result) = if big_endian[0] & 0x80 > 0 {
            (true, (big_endian[0] & 0x7f) as i32)
        } else {
            (false, big_endian[0] as i32)
        };

        for v in &big_endian[1..] {
            result <<= 9;
            result += *v as i32;
        }

        if negative {
            -result
        } else {
            result
        }
    }

    pub fn unimplemented(_: &mut Stack) -> bool {
        log::info!("Not implemented");
        return false;
    }

    pub fn nop(_: &mut Stack) -> bool {
        true
    }
    fn push_n(stack: &mut Stack, n: i32) -> bool {
        stack.main.push(encode_num(n));
        true
    }
    pub fn push_0(stack: &mut Stack) -> bool {
        push_n(stack, 0)
    }
    pub fn push_1_negate(stack: &mut Stack) -> bool {
        push_n(stack, -1)
    }
    pub fn push_1(stack: &mut Stack) -> bool {
        push_n(stack, 1)
    }
    pub fn push_2(stack: &mut Stack) -> bool {
        push_n(stack, 2)
    }
    pub fn push_3(stack: &mut Stack) -> bool {
        push_n(stack, 3)
    }
    pub fn push_4(stack: &mut Stack) -> bool {
        push_n(stack, 4)
    }
    pub fn push_5(stack: &mut Stack) -> bool {
        push_n(stack, 5)
    }
    pub fn push_6(stack: &mut Stack) -> bool {
        push_n(stack, 6)
    }
    pub fn push_7(stack: &mut Stack) -> bool {
        push_n(stack, 7)
    }
    pub fn push_8(stack: &mut Stack) -> bool {
        push_n(stack, 8)
    }
    pub fn push_9(stack: &mut Stack) -> bool {
        push_n(stack, 9)
    }
    pub fn push_10(stack: &mut Stack) -> bool {
        push_n(stack, 10)
    }
    pub fn push_11(stack: &mut Stack) -> bool {
        push_n(stack, 11)
    }
    pub fn push_12(stack: &mut Stack) -> bool {
        push_n(stack, 12)
    }
    pub fn push_13(stack: &mut Stack) -> bool {
        push_n(stack, 13)
    }
    pub fn push_14(stack: &mut Stack) -> bool {
        push_n(stack, 14)
    }
    pub fn push_15(stack: &mut Stack) -> bool {
        push_n(stack, 15)
    }
    pub fn push_16(stack: &mut Stack) -> bool {
        push_n(stack, 16)
    }

    pub fn verify(stack: &mut Stack) -> bool {
        if stack.main.len() < 1 {
            return false;
        }
        decode_num(&stack.main.pop().unwrap()) != 0
    }
    pub fn ret(_: &mut Stack) -> bool {
        false
    }

    pub fn to_alt_stack(stack: &mut Stack) -> bool {
        if stack.main.len() < 1 {
            return false;
        }
        stack.alt.push(stack.main.pop().unwrap());
        return true;
    }
    pub fn from_alt_stack(stack: &mut Stack) -> bool {
        if stack.alt.len() < 1 {
            return false;
        }
        stack.main.push(stack.alt.pop().unwrap());
        return true;
    }
    pub fn if_dup(stack: &mut Stack) -> bool {
        if stack.main.len() < 1 {
            return false;
        }

        let top = stack.main.last().unwrap();
        let x = decode_num(&top);
        if x != 0 {
            stack.main.push(top.clone());
        }
        return true;
    }
    pub fn depth(stack: &mut Stack) -> bool {
        stack.main.push(encode_num(stack.main.len() as i32));
        return true;
    }
    pub fn drop(stack: &mut Stack) -> bool {
        if stack.main.len() < 1 {
            return false;
        }
        stack.main.pop();
        return true;
    }
    pub fn dup(stack: &mut Stack) -> bool {
        if stack.main.len() < 1 {
            return false;
        }

        let top = stack.main.last().unwrap();
        stack.main.push(top.clone());
        return true;
    }
    pub fn nip(stack: &mut Stack) -> bool {
        if stack.main.len() < 2 {
            return false;
        }

        let x2 = stack.main.pop().unwrap();
        stack.main.pop().unwrap();
        stack.main.push(x2);
        return true;
    }
    pub fn over(stack: &mut Stack) -> bool {
        if stack.main.len() < 2 {
            return false;
        }

        let x = stack.main[stack.main.len() - 2].clone();
        stack.main.push(x);
        return true;
    }
    pub fn pick(stack: &mut Stack) -> bool {
        if stack.main.len() < 1 {
            return false;
        }
        let n = decode_num(&stack.main.pop().unwrap()) as usize;

        if stack.main.len() < n {
            return false;
        }
        let pick = stack.main[stack.main.len() - n].clone();
        stack.main.push(pick);
        return true;
    }
    pub fn roll(stack: &mut Stack) -> bool {
        if stack.main.len() < 1 {
            return false;
        }
        let n = decode_num(&stack.main.pop().unwrap()) as usize;

        if stack.main.len() < n {
            return false;
        }
        let pick = stack.main.remove(stack.main.len() - n);
        stack.main.push(pick);
        return true;
    }
    pub fn rot(stack: &mut Stack) -> bool {
        if stack.main.len() < 3 {
            return false;
        }

        let pick = stack.main.remove(stack.main.len() - 3);
        stack.main.push(pick);
        return true;
    }
    pub fn swap(stack: &mut Stack) -> bool {
        if stack.main.len() < 2 {
            return false;
        }

        let pick = stack.main.remove(stack.main.len() - 2);
        stack.main.push(pick);
        return true;
    }
    pub fn tuck(stack: &mut Stack) -> bool {
        if stack.main.len() < 2 {
            return false;
        }

        let pick = stack.main.last().unwrap().clone();
        stack.main.insert(stack.main.len() - 2, pick);
        return true;
    }
    pub fn drop_2(stack: &mut Stack) -> bool {
        if stack.main.len() < 2 {
            return false;
        }

        stack.main.pop();
        stack.main.pop();
        return true;
    }
    fn dup_n(stack: &mut Stack, count: usize) -> bool {
        if stack.main.len() < count {
            return false;
        }

        for _ in 0..count {
            let x = stack.main[stack.main.len() - count].clone();
            stack.main.push(x);
        }
        return true;
    }
    pub fn dup_2(stack: &mut Stack) -> bool {
        dup_n(stack, 2)
    }
    pub fn dup_3(stack: &mut Stack) -> bool {
        dup_n(stack, 3)
    }
    pub fn over_2(stack: &mut Stack) -> bool {
        if stack.main.len() < 4 {
            return false;
        }

        let x1 = stack.main[stack.main.len() - 4].clone();
        let x2 = stack.main[stack.main.len() - 3].clone();
        stack.main.push(x1);
        stack.main.push(x2);
        return true;
    }
    pub fn rot_2(stack: &mut Stack) -> bool {
        if stack.main.len() < 6 {
            return false;
        }

        let x2 = stack.main.remove(stack.main.len() - 5);
        let x1 = stack.main.remove(stack.main.len() - 5);
        stack.main.push(x1);
        stack.main.push(x2);
        return true;
    }
    pub fn swap_2(stack: &mut Stack) -> bool {
        if stack.main.len() < 4 {
            return false;
        }

        let x2 = stack.main.remove(stack.main.len() - 3);
        let x1 = stack.main.remove(stack.main.len() - 3);
        stack.main.push(x1);
        stack.main.push(x2);
        return true;
    }

    pub fn size(stack: &mut Stack) -> bool {
        if stack.main.len() < 1 {
            return false;
        }

        let x = stack.main.last().unwrap();
        stack.main.push(encode_num(x.len() as i32));
        return true;
    }
    pub fn equal(stack: &mut Stack) -> bool {
        if stack.main.len() < 2 {
            return false;
        }
        let x2 = decode_num(&stack.main.pop().unwrap());
        let x1 = decode_num(&stack.main.pop().unwrap());
        let result = if x1 == x2 { 1 } else { 0 };
        stack.main.push(encode_num(result));
        return true;
    }
    pub fn equal_verify(stack: &mut Stack) -> bool {
        let success = equal(stack);
        success && verify(stack)
    }

    fn add_n(stack: &mut Stack, n: i32) -> bool {
        if stack.main.len() < 1 {
            return false;
        }
        let x = decode_num(&stack.main.pop().unwrap());
        stack.main.push(encode_num(x + n));
        return true;
    }
    pub fn add_1(stack: &mut Stack) -> bool {
        add_n(stack, 1)
    }
    pub fn sub_1(stack: &mut Stack) -> bool {
        add_n(stack, -1)
    }
    pub fn negate(stack: &mut Stack) -> bool {
        if stack.main.len() < 1 {
            return false;
        }
        let x = decode_num(&stack.main.pop().unwrap());
        stack.main.push(encode_num(-x));
        return true;
    }
    pub fn abs(stack: &mut Stack) -> bool {
        if stack.main.len() < 1 {
            return false;
        }
        let x = decode_num(&stack.main.pop().unwrap());
        stack.main.push(encode_num(x.abs()));
        return true;
    }
    pub fn not(stack: &mut Stack) -> bool {
        if stack.main.len() < 1 {
            return false;
        }
        let result = match decode_num(&stack.main.pop().unwrap()) {
            1 => 0,
            _ => 1,
        };
        stack.main.push(encode_num(result));
        return true;
    }
    pub fn not_equal_0(stack: &mut Stack) -> bool {
        if stack.main.len() < 1 {
            return false;
        }
        let result = match decode_num(&stack.main.pop().unwrap()) {
            0 => 0,
            _ => 1,
        };
        stack.main.push(encode_num(result));
        return true;
    }
    pub fn add(stack: &mut Stack) -> bool {
        if stack.main.len() < 1 {
            return false;
        }
        let x = decode_num(&stack.main.pop().unwrap());
        add_n(stack, x)
    }
    pub fn sub(stack: &mut Stack) -> bool {
        if stack.main.len() < 1 {
            return false;
        }
        let x = decode_num(&stack.main.pop().unwrap());
        add_n(stack, -x)
    }
    fn bool_op(stack: &mut Stack, result: bool) -> bool {
        let result_num = if result { 1 } else { 0 };
        stack.main.push(encode_num(result_num));
        return true;
    }
    pub fn bool_and(stack: &mut Stack) -> bool {
        if stack.main.len() < 2 {
            return false;
        }
        let b = decode_num(&stack.main.pop().unwrap());
        let a = decode_num(&stack.main.pop().unwrap());
        bool_op(stack, a == 1 && b == 1)
    }
    pub fn bool_or(stack: &mut Stack) -> bool {
        if stack.main.len() < 2 {
            return false;
        }
        let b = decode_num(&stack.main.pop().unwrap());
        let a = decode_num(&stack.main.pop().unwrap());
        bool_op(stack, a == 1 || b == 1)
    }
    pub fn num_equal(stack: &mut Stack) -> bool {
        if stack.main.len() < 2 {
            return false;
        }
        let b = decode_num(&stack.main.pop().unwrap());
        let a = decode_num(&stack.main.pop().unwrap());
        bool_op(stack, a == b)
    }
    pub fn num_equal_verify(stack: &mut Stack) -> bool {
        let success = num_equal(stack);
        success && verify(stack)
    }
    pub fn num_not_equal(stack: &mut Stack) -> bool {
        if stack.main.len() < 2 {
            return false;
        }
        let b = decode_num(&stack.main.pop().unwrap());
        let a = decode_num(&stack.main.pop().unwrap());
        bool_op(stack, a != b)
    }
    pub fn less_than(stack: &mut Stack) -> bool {
        if stack.main.len() < 2 {
            return false;
        }
        let b = decode_num(&stack.main.pop().unwrap());
        let a = decode_num(&stack.main.pop().unwrap());
        bool_op(stack, a < b)
    }
    pub fn greater_than(stack: &mut Stack) -> bool {
        if stack.main.len() < 2 {
            return false;
        }
        let b = decode_num(&stack.main.pop().unwrap());
        let a = decode_num(&stack.main.pop().unwrap());
        bool_op(stack, a > b)
    }
    pub fn less_than_or_equal(stack: &mut Stack) -> bool {
        if stack.main.len() < 2 {
            return false;
        }
        let b = decode_num(&stack.main.pop().unwrap());
        let a = decode_num(&stack.main.pop().unwrap());
        bool_op(stack, a <= b)
    }
    pub fn greater_than_or_equal(stack: &mut Stack) -> bool {
        if stack.main.len() < 2 {
            return false;
        }
        let b = decode_num(&stack.main.pop().unwrap());
        let a = decode_num(&stack.main.pop().unwrap());
        bool_op(stack, a <= b)
    }
    pub fn min(stack: &mut Stack) -> bool {
        if stack.main.len() < 2 {
            return false;
        }
        let b = decode_num(&stack.main.pop().unwrap());
        let a = decode_num(&stack.main.pop().unwrap());
        let result = if a < b { a } else { b };
        stack.main.push(encode_num(result));
        return true;
    }
    pub fn max(stack: &mut Stack) -> bool {
        if stack.main.len() < 2 {
            return false;
        }
        let b = decode_num(&stack.main.pop().unwrap());
        let a = decode_num(&stack.main.pop().unwrap());
        let result = if a > b { a } else { b };
        stack.main.push(encode_num(result));
        return true;
    }
    pub fn within(stack: &mut Stack) -> bool {
        if stack.main.len() < 2 {
            return false;
        }
        let max = decode_num(&stack.main.pop().unwrap());
        let min = decode_num(&stack.main.pop().unwrap());
        let x = decode_num(&stack.main.pop().unwrap());
        bool_op(stack, x >= min && x < max)
    }

    pub fn ripemd160(stack: &mut Stack) -> bool {
        if stack.main.len() < 1 {
            return false;
        }
        let value = stack.main.pop().unwrap();
        stack.main.push(hashing::ripemd160(value));
        return true;
    }
    pub fn sha256(stack: &mut Stack) -> bool {
        if stack.main.len() < 1 {
            return false;
        }
        let value = stack.main.pop().unwrap();
        stack.main.push(hashing::sha256(value));
        return true;
    }
    pub fn hash160(stack: &mut Stack) -> bool {
        if stack.main.len() < 1 {
            return false;
        }
        let value = stack.main.pop().unwrap();
        stack.main.push(hashing::hash160(value));
        return true;
    }
    pub fn hash256(stack: &mut Stack) -> bool {
        if stack.main.len() < 1 {
            return false;
        }
        let value = stack.main.pop().unwrap();
        stack.main.push(hashing::hash256(value));
        return true;
    }

    use crate::curves::Point;
    use crate::ecdsa::{Secp256k1, Signature};
    use crate::u256::U256;

    pub fn check_sig(stack: &mut Stack) -> bool {
        if stack.main.len() < 2 {
            return false;
        }
        let secp = Secp256k1::new();

        let sec_pubkey = match Point::<U256>::parse(&secp.curve, &stack.main.pop().unwrap()) {
            None => return false,
            Some(point) => point,
        };

        let der_signature = stack.main.pop().unwrap();
        // Remove hash type byte
        let der_signature = match Signature::parse(&der_signature[..der_signature.len() - 1]) {
            Err(_) => return false,
            Ok(signature) => signature,
        };

        if secp.verify(&stack.sig_hash, &der_signature, &sec_pubkey) {
            stack.main.push(encode_num(1));
        } else {
            stack.main.push(encode_num(0));
        }
        return true;
    }

    pub fn check_sig_verify(stack: &mut Stack) -> bool {
        check_sig(stack) && verify(stack)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hex_literal::hex;

    #[test]
    fn script_default() {
        assert_eq!(Script::default().serialize(), vec![0x00]);
    }

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

    #[test]
    fn script_runner() {
        let code = hex!("05 59 55 93 59 87"); // OP_9 OP_5 OP_ADD OP_9 OP_EQUAL
        let script = Script::parse(&code).unwrap().0;
        let mut stack = Stack::new(Hash::default());
        assert!(run(&script, &mut stack));
        assert_eq!(stack.main.len(), 1);
        assert_eq!(op::decode_num(&stack.main[0]), 0);
    }

    fn script_from_slice(data: &[u8]) -> Script {
        Script::parse(data).unwrap().0
    }

    #[test]
    fn script_evaluation() {
        let script_pubkey = script_from_slice(hex!("04 55 93 59 87").as_slice()); // OP_5 OP_ADD OP_9 OP_EQUAL
        let script_sig = script_from_slice(hex!("01 54").as_slice()); // OP_4
        assert!(evaluate(&Hash::default(), &script_pubkey, &script_sig));

        let wrong_script_sig = script_from_slice(hex!("01 55").as_slice()); // OP_5
        assert!(!evaluate(
            &Hash::default(),
            &script_pubkey,
            &wrong_script_sig
        ));
    }
}
