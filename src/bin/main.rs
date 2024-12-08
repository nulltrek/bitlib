use bitlib::block::BlockId;
use bitlib::builders::TxBuilder;
use bitlib::definitions::{Compression, Network};
use bitlib::ecdsa::{PrivateKey, Secp256k1};
use bitlib::hashing::hash256;
use bitlib::http::{HttpError, NodeClient};
use bitlib::tx::{TxFetcher, TxId};
use bitlib::u256::U256;
use std::env;
use ureq::Error;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Please specify cookie path");
        return;
    }

    println!("Generating testnet address");
    let secp = Secp256k1::new();
    let secret_hash = hash256("Wherever Two Or More Are Gathered... As The Earth Kissed The Moon");
    let privkey = PrivateKey::new(U256::from_little_endian(&secret_hash));
    let pubkey = secp.get_pubkey(&privkey);
    println!("{}", pubkey.to_address(Compression::Yes, Network::Test));
    // migshLXU9p2VFDzeV3AwKC6EHQuVXuUGeQ

    let mut client =
        NodeClient::new("http://127.0.0.1:18443", std::path::Path::new(&args[1])).unwrap();
    let block = client
        .fetch_block_header(&BlockId::from(U256::from_hex(
            "2b21398988ebe31b4bec42f385763802c555537d23a5a1f35fc00f73f9bc4c0e",
        )))
        .unwrap();
    println!("Block: {}\n", block);
    let prev_tx = client
        .fetch_tx(&TxId::from(U256::from_hex(
            "d31df99a4c65b9ef257a84217bb7b7588b573789cb6a21ecd274e5e8100c5844",
        )))
        .unwrap();
    println!("{}", prev_tx.id());

    let tx = TxBuilder::new()
        .add_input(prev_tx.hash(), 0, 0)
        .add_output(2500000000, &pubkey)
        .add_output(2499999500, &pubkey)
        .build(&mut client, &privkey)
        .unwrap();
    println!("{}", tx);

    let result = client.send_tx(&tx);
    match result {
        Err(HttpError::RequestError(Error::Status(code, response))) => {
            println!("{} - {:?}", code, response.into_string())
        }
        other => println!("{:?}", other),
    };
}
