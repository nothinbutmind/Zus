use bignum::{BigNum, Secp256k1_Fq as Secp256k1Fq};
use keccak256::keccak256;
use noir_bigcurve::BigCurve;
use noir_bigcurve::curves::secp256k1::{Secp256k1, Secp256k1Scalar};
use poseidon::poseidon2::Poseidon2::hash;
use trees::merkle::MerkleTree;
use trees::types::{MT_Creator, MembershipProver};

global DEPTH: u32 = 12;
global COMPRESSED_SIZE_BYTES: u32 = 33;

fn hasher(leaves: [Field; 2]) -> Field {
    hash(leaves, 2)
}

fn compress_point(point: Secp256k1) -> [u8; COMPRESSED_SIZE_BYTES] {
    let x = point.x.to_le_bytes();
    let y = point.y.to_le_bytes();
    let mut compressed = [0 as u8; COMPRESSED_SIZE_BYTES];

    compressed[0] = (y[0] & 1) + 2;
    for i in 0..32 {
        compressed[32 - i] = x[i];
    }

    compressed
}

fn plume_message<let L: u32>(message: [u8; L], pubkey: Secp256k1) -> [u8; L + COMPRESSED_SIZE_BYTES] {
    let mut result = [0 as u8; L + COMPRESSED_SIZE_BYTES];

    for i in 0..L {
        result[i] = message[i];
    }

    let compressed = compress_point(pubkey);
    for i in 0..COMPRESSED_SIZE_BYTES {
        result[L + i] = compressed[i];
    }

    result
}

fn verify_plume_v2<let L: u32>(
    message: [u8; L],
    c: Secp256k1Scalar,
    s: Secp256k1Scalar,
    pubkey: Secp256k1,
    nullifier: Secp256k1,
) {
    let s_point = Secp256k1::one().mul(s);
    let _r_point = s_point - pubkey.mul(c);

    let h = Secp256k1::hash_to_curve(plume_message(message, pubkey));
    let _hashed_to_curve_r = h.mul(s) - nullifier.mul(c);
}

fn to_eth_address(pub_key_x: [u8; 32], pub_key_y: [u8; 32]) -> Field {
    let mut pub_key = [0 as u8; 64];

    for i in 0..32 {
        pub_key[i] = pub_key_x[i];
        pub_key[32 + i] = pub_key_y[i];
    }

    let hashed = keccak256(pub_key, 64);
    let mut address: Field = 0;

    for i in 0..20 {
        address = (address * 256) + (hashed[12 + i] as Field);
    }

    address
}

fn main(
    pub_key_x: [u8; 32],
    pub_key_y: [u8; 32],
    message: pub [u8; 8],
    c: [u8; 32],
    s: [u8; 32],
    nullifier_x: pub [u8; 32],
    nullifier_y: pub [u8; 32],
    eligible_root: pub Field,
    eligible_path: [Field; DEPTH],
    eligible_index: Field,
    claimer: Field,
) -> pub Field {
    let scalar_c = Secp256k1Scalar::from_bignum(Secp256k1Fq::from_be_bytes(c));
    let scalar_s = Secp256k1Scalar::from_bignum(Secp256k1Fq::from_be_bytes(s));

    let pubkey = Secp256k1 {
        x: Secp256k1Fq::from_be_bytes(pub_key_x),
        y: Secp256k1Fq::from_be_bytes(pub_key_y),
        is_infinity: false,
    };

    let nullifier = Secp256k1 {
        x: Secp256k1Fq::from_be_bytes(nullifier_x),
        y: Secp256k1Fq::from_be_bytes(nullifier_y),
        is_infinity: false,
    };

    verify_plume_v2(message, scalar_c, scalar_s, pubkey, nullifier);

    let address = to_eth_address(pub_key_x, pub_key_y);

    let mt = MerkleTree::from(eligible_root, hasher);
    mt.membership(address, eligible_index, eligible_path);

    claimer
}
