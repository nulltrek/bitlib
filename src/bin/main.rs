use bitlib::definitions::{Compression, Network};
use bitlib::ecdsa::{PrivateKey, Secp256k1};
use bitlib::hashing::hash256;
use bitlib::u256::U256;

fn main() {
    println!("Generating testnet address");
    let secp = Secp256k1::new();
    let secret_hash = hash256("Wherever Two Or More Are Gathered... As The Earth Kissed The Moon");
    let pubkey = secp.get_pubkey(&PrivateKey::new(U256::from_big_endian(&secret_hash)));
    // mvjLH9YikB9kuWbw8C6uE1SSY54QqvNzQX
    // n3ueoiZVnRaGqxDCdZtQcKYeMa4oznzmiY
    println!("{}", pubkey.to_address(Compression::No, Network::Test));
}
