use crate::ecdsa::{PrivateKey, PublicKey, Secp256k1, Signature};
use crate::hashing::hash160;
use crate::script::Script;
use crate::tx::{SigHash, Tx, TxError, TxFetcher, TxId, TxInput, TxOutput};

#[derive(Debug)]
pub enum BuildError {
    TxError(TxError),
}

impl From<TxError> for BuildError {
    fn from(error: TxError) -> Self {
        BuildError::TxError(error)
    }
}

pub type Result<T> = std::result::Result<T, BuildError>;

struct TxBuilder {
    tx: Tx,
}

impl TxBuilder {
    pub fn new() -> TxBuilder {
        TxBuilder {
            tx: Tx {
                version: 1,
                inputs: vec![],
                outputs: vec![],
                locktime: 0,
            },
        }
    }

    pub fn add_input(mut self, prev_tx: TxId, prev_tx_index: u32, sequence: u32) -> TxBuilder {
        let script_sig = Script::default();
        self.tx.inputs.push(TxInput {
            prev_tx,
            prev_tx_index,
            script_sig,
            sequence,
        });
        self
    }

    pub fn add_output(mut self, amount: u64, pubkey: &PublicKey) -> TxBuilder {
        let script_pubkey = P2pkh::script_pubkey(pubkey);
        self.tx.outputs.push(TxOutput {
            amount,
            script_pubkey,
        });
        self
    }

    pub fn build<F: TxFetcher>(mut self, fetcher: &mut F, privkey: &PrivateKey) -> Result<Tx> {
        let secp = Secp256k1::new();
        let pubkey = secp.get_pubkey(privkey);
        for i in 0..self.tx.inputs.len() {
            let signature = self.tx.sign_input(fetcher, i, SigHash::All, privkey)?;
            self.tx.inputs[i].script_sig = P2pkh::script_sig(signature, SigHash::All, pubkey);
        }
        Ok(self.tx)
    }
}

struct P2pkh;

impl P2pkh {
    pub fn script_pubkey(pubkey: &PublicKey) -> Script {
        let pubkey_hash = hash160(&pubkey.to_csec());

        let script = [
            &[0x76, 0xa9], // <script len> OP_DUP OP_HASH160
            pubkey_hash.as_slice(),
            &[0x88, 0xac], // OP_EQUALVERIFY OP_CHECKSIG
        ]
        .concat();
        Script::from_slice(&script).unwrap()
    }

    pub fn script_sig(signature: Signature, flag: SigHash, pubkey: PublicKey) -> Script {
        let der_sig = [signature.to_der().as_slice(), &[flag.to_u8()]].concat();
        let sec_pubkey = pubkey.to_csec();

        let script = [der_sig.as_slice(), sec_pubkey.as_slice()].concat();
        Script::from_slice(&script).unwrap()
    }
}
