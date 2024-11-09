use crate::curves::{add as cadd, mul as cmul, Curve, Point};
use crate::fields::{add as fadd, div as fdiv, mul as fmul, Field, FiniteFieldU256};
use crate::hashing::{hash256, hmac};
use crate::u256::U256;
use num_traits::cast::FromPrimitive;
use std::ops::Deref;

#[derive(Debug, PartialEq)]
pub struct Signature {
    pub r: U256,
    pub s: U256,
}

pub struct Hash(U256);

impl Hash {
    fn hash256(data: impl AsRef<[u8]>) -> Hash {
        Hash(U256::from_big_endian(hash256(data).as_slice()))
    }
}

impl From<U256> for Hash {
    fn from(num: U256) -> Self {
        Hash(num)
    }
}

pub struct PrivateKey(U256);

impl PrivateKey {
    pub fn new(value: U256) -> PrivateKey {
        PrivateKey(value)
    }
}

impl Deref for PrivateKey {
    type Target = U256;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug)]
pub struct PublicKey(Point<U256>);

impl Deref for PublicKey {
    type Target = Point<U256>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct Secp256k1 {
    pub curve: Curve<U256, FiniteFieldU256>,
    g: Point<U256>,
    n: U256,
}

impl Secp256k1 {
    pub fn new() -> Secp256k1 {
        let field = FiniteFieldU256::new(U256::from_hex(
            "fffffffffffffffffffffffffffffffffffffffffffffffffffffffefffffc2f",
        ));
        let curve = Curve::<U256, FiniteFieldU256>::new(
            field,
            U256::from_u32(0).unwrap(),
            U256::from_u32(7).unwrap(),
        );

        let g = Point::coords(
            U256::from_hex("79be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798"),
            U256::from_hex("483ada7726a3c4655da4fbfc0e1108a8fd17b448a68554199c47d08ffb10d4b8"),
        );
        let n = U256::from_hex("fffffffffffffffffffffffffffffffebaaedce6af48a03bbfd25e8cd0364141");
        Secp256k1 { curve, g, n }
    }

    pub fn get_pubkey(&self, privkey: &PrivateKey) -> PublicKey {
        PublicKey(cmul!(self.curve, privkey.0, self.g))
    }

    pub fn sign(&self, hash: &Hash, privkey: &PrivateKey) -> Signature {
        let field = FiniteFieldU256::new(self.n);

        let k = self.gen_k(&hash, &privkey);
        let r = cmul!(self.curve, k, self.g);
        let rx = match r {
            Point::Inf => panic!("Result for signature random challenge is a point at infinity."),
            Point::Coords { x, .. } => x,
        };
        let mut s = fdiv!(field, fadd!(field, hash.0, fmul!(field, rx, privkey.0)), k);
        // Only use low-s values for malleability reasons
        // n >> 1 == n / 2
        if s > self.n >> 1 {
            s = self.n - s;
        }
        Signature { r: rx, s: s }
    }

    fn gen_k(&self, hash: &Hash, privkey: &PrivateKey) -> U256 {
        let mut z = hash.0;
        if z > self.n {
            z = z - self.n;
        }
        let z = z.to_big_endian();
        let p = privkey.0.to_big_endian();

        let k = U256::default().to_big_endian().to_vec();
        let v = vec![
            1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
            1, 1, 1,
        ];

        let k = hmac(
            &k,
            [v.as_slice(), &[0_u8], p.as_slice(), z.as_slice()]
                .concat()
                .as_slice(),
        )
        .to_vec();
        let v = hmac(&k, &v).to_vec();
        let k = hmac(
            &k,
            [v.as_slice(), &[1_u8], p.as_slice(), z.as_slice()]
                .concat()
                .as_slice(),
        )
        .to_vec();
        let v = hmac(&k, &v).to_vec();

        loop {
            let v = hmac(&k, &v).to_vec();
            let candidate = U256::from_big_endian(&v);
            if !candidate.is_zero() && candidate < self.n {
                return candidate;
            }
            let k = hmac(&k, [v.as_slice(), &[0_u8]].concat().as_slice()).to_vec();
            let v = hmac(&k, &v).to_vec();
        }
    }

    pub fn verify(&self, hash: &Hash, signature: &Signature, pubkey: &PublicKey) -> bool {
        let curve = &self.curve;
        let field = FiniteFieldU256::new(self.n);

        let u = fdiv!(field, hash.0, signature.s);
        let v = fdiv!(field, signature.r, signature.s);
        let result = cadd!(curve, cmul!(curve, u, self.g), cmul!(curve, v, pubkey.0));
        match result {
            Point::Inf => false,
            Point::Coords { x, .. } => x == signature.r,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::curves::mul;

    #[test]
    fn curve_construction() {
        let secp = Secp256k1::new();
        let n = U256::from_hex("fffffffffffffffffffffffffffffffebaaedce6af48a03bbfd25e8cd0364141");
        assert_eq!(mul!(secp.curve, n, secp.g), Point::Inf);
    }

    #[test]
    fn pubkey_computation() {
        let secp = Secp256k1::new();
        assert_eq!(
            *secp.get_pubkey(&PrivateKey::new(U256::from_dec("33466154331649568"))),
            Point::coords(
                U256::from_hex("027f3da1918455e03c46f659266a1bb5204e959db7364d2f473bdf8f0a13cc9d"),
                U256::from_hex("ff87647fd023c13b4a4994f17691895806e1b40b57f4fd22581a4f46851f3b06"),
            )
        )
    }

    #[test]
    fn signature_verification() {
        let secp = Secp256k1::new();
        let pubkey = PublicKey(Point::coords(
            U256::from_hex("887387e452b8eacc4acfde10d9aaf7f6d9a0f975aabb10d006e4da568744d06c"),
            U256::from_hex("61de6d95231cd89026e286df3b6ae4a894a3378e393e93a0f45b666329a0ae34"),
        ));

        let hash = Hash(U256::from_hex(
            "ec208baa0fc1c19f708a9ca96fdeff3ac3f230bb4a7ba4aede4942ad003c0f60",
        ));
        let signature = Signature {
            r: U256::from_hex("ac8d1c87e51d0d441be8b3dd5b05c8795b48875dffe00b7ffcfac23010d3a395"),
            s: U256::from_hex("68342ceff8935ededd102dd876ffd6ba72d6a427a3edb13d26eb0781cb423c4"),
        };
        assert!(secp.verify(&hash, &signature, &pubkey));

        let hash = Hash(U256::from_hex(
            "7c076ff316692a3d7eb3c3bb0f8b1488cf72e1afcd929e29307032997a838a3d",
        ));
        let signature = Signature {
            r: U256::from_hex("00eff69ef2b1bd93a66ed5219add4fb51e11a840f404876325a1e8ffe0529a2c"),
            s: U256::from_hex("c7207fee197d27c618aea621406f6bf5ef6fca38681d82b2f06fddbdce6feab6"),
        };
        assert!(secp.verify(&hash, &signature, &pubkey));
    }

    #[test]
    fn sign_data() {
        let secp = Secp256k1::new();
        let privkey = PrivateKey(U256::from_big_endian(b"my secret"));
        let hash = Hash(U256::from_big_endian(b"my data"));
        let signature = secp.sign(&hash, &privkey);
        assert!(secp.verify(&hash, &signature, &secp.get_pubkey(&privkey)))
    }
}
