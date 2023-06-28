use sha2::{Sha256, Digest};
use rand_core::OsRng;
use ripemd::Ripemd160;
use base58::{FromBase58, ToBase58};

use k256::{ecdsa::{SigningKey, Signature, signature::Signer}, PublicKey};
use k256::{ecdsa::{VerifyingKey, signature::Verifier}};
use k256::ecdsa::signature::SignatureEncoding;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum CryptoError {
    Base58DecodeError,
    InvalidPubKey,
    InvalidSignature
}

pub type Result<T> = std::result::Result<T, CryptoError>;

impl std::fmt::Display for CryptoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "crypto error")
    }
}


pub fn calculate_sha256_hash(data: &[u8], buf : &mut [u8]) {
    let mut hasher = Sha256::new();
    hasher.update(data);
    buf.copy_from_slice(&hasher.finalize());
}

pub fn leading_zeros_count(hash: &str) -> u8 {
    let mut count = 0;
    let mut iter = hash.chars();

    while iter.next() == Some('0') {
        count += 1;
    }

    count
}

// --- Interface for keys and address
pub fn create_signing_key() -> SigningKey {
    let signing_key = SigningKey::random(&mut OsRng);
    signing_key
}

pub fn get_private_key(signing_key: &SigningKey) -> [u8; 32] {
    signing_key.to_bytes().into()
}

pub fn get_public_key(signing_key: &SigningKey) -> Vec<u8> {
    let public_key = VerifyingKey::from(signing_key);
    public_key.to_encoded_point(false).as_bytes().to_vec()
}

pub fn get_address(signing_key: SigningKey) -> String {
    let pub_key_hash = get_public_key_hash(&signing_key);
    // Create a variable result and apply changes to it until we get the final address
    let mut result = add_prefix_to_public_key_hash(0, &pub_key_hash);
    get_check_sum(&result).iter().for_each(|b| result.push(*b));
    result.to_base58()
}

pub fn address_to_public_key_hash(address: &String) -> Result<[u8; 20]> {
    if let Ok(mut pub_key_hash) = address.from_base58() {
        if pub_key_hash.len() == 25 {
            pub_key_hash.remove(0);
            pub_key_hash.drain(pub_key_hash.len()-4..);

            // Convert pub_key_hash to a 20 bytes array
            let mut result = [0u8; 20];
            result.copy_from_slice(pub_key_hash.as_slice());
            return Ok(result);
        }
    }
    Err(CryptoError::Base58DecodeError)
}

// --- Keys/Address creation
pub fn add_prefix_to_public_key_hash(prefix: u8, hash: &[u8]) -> Vec<u8> {
    let mut result = vec![prefix];
    result.extend_from_slice(hash);
    result
}

// Calculate and return ripemd160 hash from 'data'
pub fn get_ripemd_hash(data: &[u8]) -> [u8; 20] {
    let mut hasher = Ripemd160::new();
    hasher.update(data);
    hasher.finalize().into()
}

// Hash public key (sha256) and convert it to a 160 bytes hash (ripemd160)
pub fn get_public_key_hash(signing_key: &SigningKey) -> [u8; 20] {
    let mut buffer = [0u8; 32];
    calculate_sha256_hash(&get_public_key(signing_key), &mut buffer);
    get_ripemd_hash(&buffer)
}

// Get and return first four bytes from 'hash'
pub fn get_check_sum(hash: &Vec<u8>) -> [u8; 4] {
    let mut buffer = [0u8; 32];
    calculate_sha256_hash(hash, &mut buffer);
    calculate_sha256_hash(&buffer.clone(), &mut buffer);

    let mut result= [0u8; 4];
    result.copy_from_slice(&buffer[0..4]);
    result
}

pub fn get_signature(signing_key: &SigningKey, data: &[u8]) -> Vec<u8> {
    let signature: Signature = signing_key.sign(data);
    signature.to_der().to_vec()
}


// der_signature must be encoded as hexadecimal
pub fn verify_signature(public_key: &[u8], der_signature: &[u8], message: &[u8]) -> Result<bool> {
    if let Ok(pub_key) = PublicKey::from_sec1_bytes(public_key) {
        if let Ok(signature) = Signature::from_der(der_signature) {
            let verifying_key = VerifyingKey::from(pub_key);
            return Ok(verifying_key.verify(message, &signature).is_ok());
        }
        return Err(CryptoError::InvalidSignature);
    }
    Err(CryptoError::InvalidPubKey)
}

#[cfg(test)]
mod tests {
    use k256::ecdsa::{DerSignature, Signature, SigningKey, VerifyingKey};
    use k256::ecdsa::signature::Verifier;


    use super::*;

    //#[test]
    fn test_address_to_pub_key_hash_conversion() {
        let address = String::from("128GaUUoKKnEgioDsm5Pa9FxmXtzQMk3F9");
        let pub_key_hash = address_to_public_key_hash(&address).unwrap();

        assert_eq!(hex::encode(pub_key_hash), String::from("0c580a683d25baaa95c412c99f4fe919eacbd88a"))
    }

    //#[test]
    fn test_verify_signature() {
        let signing_key = SigningKey::from_slice(hex::decode("ae1af0af67c13ee57a00d770c157247f55bf793769e73f05ebc7be08062ea347").unwrap().as_slice()).unwrap();
        let signature = get_signature(&signing_key, b"data"); // Signature as String (as it will be stored as String)

        //let signature_hex = hex::decode(signature).unwrap();
        assert!(verify_signature(get_public_key(&signing_key).as_slice(), signature.as_slice(), b"data").unwrap())
    }
}
