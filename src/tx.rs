use crate::ecdsa::Hash;
use crate::hashing::{hash256, to_hex_str};
use crate::script::Script;
use crate::serialization::{slice_to_array, varint, Result};
use crate::u256::U256;
use core::fmt;

type TxId = Hash;

struct TxInput {
    prev_tx: TxId,
    prev_tx_index: u32,
    script_sig: Script,
    sequence: u32,
}

impl TxInput {
    pub fn parse(data: &[u8]) -> Result<(TxInput, usize)> {
        let prev_tx = Hash::from(U256::from_little_endian(&data[0..32]));
        let prev_tx_index = u32::from_le_bytes(slice_to_array(&data[32..36]));
        let (script_sig, script_sig_len) = Script::parse(&data[36..])?;
        let seq_offset = 36 + script_sig_len;
        let end_offset = seq_offset + 4;
        let sequence = u32::from_le_bytes(slice_to_array(&data[seq_offset..end_offset]));

        Ok((
            TxInput {
                prev_tx,
                prev_tx_index,
                script_sig,
                sequence,
            },
            end_offset,
        ))
    }
}

impl fmt::Display for TxInput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "prev_tx: {}\nprev_tx_index: {}\nscript_sig: {}\nsequence: {}\n",
            self.prev_tx, self.prev_tx_index, self.script_sig, self.sequence
        )
    }
}

struct TxOutput {
    amount: u64,
    script_pubkey: Script,
}

impl TxOutput {
    pub fn parse(data: &[u8]) -> Result<(TxOutput, usize)> {
        let amount = u64::from_le_bytes(slice_to_array(&data[0..8]));
        let (script_pubkey, script_pubkey_length) = Script::parse(&data[8..])?;
        let end_offset = 8 + script_pubkey_length;
        Ok((
            TxOutput {
                amount,
                script_pubkey,
            },
            end_offset,
        ))
    }
}

impl fmt::Display for TxOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "amount: {}\nscript_pubkey: {}\n",
            self.amount, self.script_pubkey
        )
    }
}

struct Tx {
    version: u32,
    inputs: Vec<TxInput>,
    outputs: Vec<TxOutput>,
    locktime: u32,
}

impl Tx {
    pub fn id(&self) -> String {
        let hash = hash256(self.serialize())
            .into_iter()
            .rev()
            .collect::<Vec<u8>>();
        to_hex_str(hash.as_slice())
    }

    pub fn serialize(&self) -> Vec<u8> {
        vec![]
    }

    pub fn parse(data: &[u8]) -> Result<Tx> {
        let version = u32::from_le_bytes(slice_to_array(&data[0..4]));

        let mut offset = if data[4] == 0 { 6 } else { 4 };

        let (input_count, varint_length) = varint::parse(&data[offset..])?;
        offset += varint_length;
        let mut inputs = vec![];
        for i in 0..input_count {
            let (input, input_length) = TxInput::parse(&data[offset..])?;
            offset += input_length;
            inputs.push(input)
        }

        let (output_count, varint_length) = varint::parse(&data[offset..])?;
        offset += varint_length;
        let mut outputs = vec![];
        for i in 0..output_count {
            let (output, output_length) = TxOutput::parse(&data[offset..])?;
            offset += output_length;
            outputs.push(output)
        }

        let locktime = u32::from_le_bytes(slice_to_array(&data[offset..offset + 4]));

        Ok(Tx {
            version,
            inputs,
            outputs,
            locktime,
        })
    }
}

impl fmt::Display for Tx {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "tx: {}\nversion: {}\ntx_inputs:\n",
            self.id(),
            self.version,
        )?;

        for (i, tx_in) in self.inputs.iter().enumerate() {
            write!(f, "{}:\n{}\n", i, tx_in)?;
        }
        write!(f, "tx_outputs:\n",)?;
        for (i, tx_out) in self.outputs.iter().enumerate() {
            write!(f, "{}: {}\n", i, tx_out)?;
        }

        write!(f, "locktime: {}\n", self.locktime,)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hex_literal::hex;

    #[test]
    fn tx_version() {
        let bytes = hex!("0100000001813f79011acb80925dfe69b3def355fe914bd1d96a3f5f71bf8303c6a989c7d1000000006b483045022100ed81ff192e75a3fd2304004dcadb746fa5e24c5031ccfcf21320b0277457c98f02207a986d955c6e0cb35d446a89d3f56100f4d7f67801c31967743a9c8e10615bed01210349fc4e631e3624a545de3f89f5d8684c7b8138bd94bdd531d2e213bf016b278afeffffff02a135ef01000000001976a914bc3b654dca7e56b04dca18f2566cdaf02e8d9ada88ac99c39800000000001976a9141c4bc762dd5423e332166702cb75f40df79fea1288ac19430600");
        let tx = Tx::parse(&bytes).unwrap();
        assert_eq!(tx.version, 1)
    }

    #[test]
    fn tx_inputs() {
        let bytes = hex!("0100000001813f79011acb80925dfe69b3def355fe914bd1d96a3f5f71bf8303c6a989c7d1000000006b483045022100ed81ff192e75a3fd2304004dcadb746fa5e24c5031ccfcf21320b0277457c98f02207a986d955c6e0cb35d446a89d3f56100f4d7f67801c31967743a9c8e10615bed01210349fc4e631e3624a545de3f89f5d8684c7b8138bd94bdd531d2e213bf016b278afeffffff02a135ef01000000001976a914bc3b654dca7e56b04dca18f2566cdaf02e8d9ada88ac99c39800000000001976a9141c4bc762dd5423e332166702cb75f40df79fea1288ac19430600");
        let tx = Tx::parse(&bytes).unwrap();
        assert_eq!(tx.inputs.len(), 1);
        assert_eq!(
            tx.inputs[0].prev_tx,
            Hash::from(U256::from_hex(
                "d1c789a9c60383bf715f3f6ad9d14b91fe55f3deb369fe5d9280cb1a01793f81"
            ))
        );
        assert_eq!(tx.inputs[0].prev_tx_index, 0);
        assert_eq!(tx.inputs[0].script_sig.serialize(), hex!("6b483045022100ed81ff192e75a3fd2304004dcadb746fa5e24c5031ccfcf21320b0277457c98f02207a986d955c6e0cb35d446a89d3f56100f4d7f67801c31967743a9c8e10615bed01210349fc4e631e3624a545de3f89f5d8684c7b8138bd94bdd531d2e213bf016b278a"));
        assert_eq!(tx.inputs[0].sequence, 0xfffffffe);
    }

    #[test]
    fn tx_outputs() {
        let bytes = hex!("0100000001813f79011acb80925dfe69b3def355fe914bd1d96a3f5f71bf8303c6a989c7d1000000006b483045022100ed81ff192e75a3fd2304004dcadb746fa5e24c5031ccfcf21320b0277457c98f02207a986d955c6e0cb35d446a89d3f56100f4d7f67801c31967743a9c8e10615bed01210349fc4e631e3624a545de3f89f5d8684c7b8138bd94bdd531d2e213bf016b278afeffffff02a135ef01000000001976a914bc3b654dca7e56b04dca18f2566cdaf02e8d9ada88ac99c39800000000001976a9141c4bc762dd5423e332166702cb75f40df79fea1288ac19430600");
        let tx = Tx::parse(&bytes).unwrap();
        assert_eq!(tx.outputs.len(), 2);
        assert_eq!(tx.outputs[0].amount, 32454049);
        assert_eq!(
            tx.outputs[0].script_pubkey.serialize(),
            hex!("1976a914bc3b654dca7e56b04dca18f2566cdaf02e8d9ada88ac")
        );

        assert_eq!(tx.outputs[1].amount, 10011545);
        assert_eq!(
            tx.outputs[1].script_pubkey.serialize(),
            hex!("1976a9141c4bc762dd5423e332166702cb75f40df79fea1288ac")
        );
    }

    #[test]
    fn tx_locktime() {
        let bytes = hex!("0100000001813f79011acb80925dfe69b3def355fe914bd1d96a3f5f71bf8303c6a989c7d1000000006b483045022100ed81ff192e75a3fd2304004dcadb746fa5e24c5031ccfcf21320b0277457c98f02207a986d955c6e0cb35d446a89d3f56100f4d7f67801c31967743a9c8e10615bed01210349fc4e631e3624a545de3f89f5d8684c7b8138bd94bdd531d2e213bf016b278afeffffff02a135ef01000000001976a914bc3b654dca7e56b04dca18f2566cdaf02e8d9ada88ac99c39800000000001976a9141c4bc762dd5423e332166702cb75f40df79fea1288ac19430600");
        let tx = Tx::parse(&bytes).unwrap();
        println!("{}", tx);
        assert_eq!(tx.locktime, 410393);
    }
}
