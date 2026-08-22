//! Cryptographic utilities for AWE_OS (`awe-securityd`).

pub use ed25519_dalek::{SigningKey, VerifyingKey, Signature as Ed25519Signature};
pub use hmac::{Hmac, Mac, KeyInit};
pub use sha2::{Sha256, Sha512, Digest};

pub type HmacSha256 = Hmac<Sha256>;

pub fn sha256(data: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().into()
}

pub fn sha512(data: &[u8]) -> [u8; 64] {
    let mut hasher = Sha512::new();
    hasher.update(data);
    hasher.finalize().into()
}

pub fn hmac_sha256(key: &[u8], data: &[u8]) -> [u8; 32] {
    let mut mac = HmacSha256::new_from_slice(key).expect("HMAC can take key of any size");
    mac.update(data);
    mac.finalize().into_bytes().into()
}

pub fn ed25519_keypair_from_seed(seed: &[u8; 32]) -> ([u8; 32], [u8; 64]) {
    let signing_key = SigningKey::from_bytes(seed);
    let verifying_key = signing_key.verifying_key();

    let mut secret_key_bytes = [0u8; 64];
    secret_key_bytes[..32].copy_from_slice(signing_key.as_bytes());
    secret_key_bytes[32..].copy_from_slice(verifying_key.as_bytes());

    (*verifying_key.as_bytes(), secret_key_bytes)
}

pub fn ed25519_sign(secret_key: &[u8; 64], message: &[u8]) -> [u8; 64] {
    let seed: &[u8; 32] = secret_key[..32].try_into().expect("32 bytes seed");
    let signing_key = SigningKey::from_bytes(seed);
    use ed25519_dalek::Signer;
    let signature = signing_key.sign(message);
    signature.to_bytes()
}

pub fn ed25519_verify(public_key: &[u8; 32], message: &[u8], signature: &[u8; 64]) -> bool {
    let Ok(verifying_key) = VerifyingKey::from_bytes(public_key) else {
        return false;
    };
    let sig = ed25519_dalek::Signature::from_bytes(signature);
    verifying_key.verify_strict(message, &sig).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sha256_rfc_vectors() {
        assert_eq!(
            sha256(b""),
            [
                0xe3, 0xb0, 0xc4, 0x42, 0x98, 0xfc, 0x1c, 0x14, 0x9a, 0xfb, 0xf4, 0xc8, 0x99, 0x6f,
                0xb9, 0x24, 0x27, 0xae, 0x41, 0xe4, 0x64, 0x9b, 0x93, 0x4c, 0xa4, 0x95, 0x99, 0x1b,
                0x78, 0x52, 0xb8, 0x55
            ]
        );
        assert_eq!(
            sha256(b"abc"),
            [
                0xba, 0x78, 0x16, 0xbf, 0x8f, 0x01, 0xcf, 0xea, 0x41, 0x41, 0x40, 0xde, 0x5d, 0xae,
                0x22, 0x23, 0xb0, 0x03, 0x61, 0xa3, 0x96, 0x17, 0x7a, 0x9c, 0xb4, 0x10, 0xff, 0x61,
                0xf2, 0x00, 0x15, 0xad
            ]
        );
    }

    #[test]
    fn test_hmac_sha256_rfc4231_vector1() {
        let key = [0x0bu8; 20];
        let data = b"Hi There";
        let expected = [
            0xb0, 0x34, 0x4c, 0x61, 0xd8, 0xdb, 0x38, 0x53, 0x5c, 0xa8, 0xaf, 0xce, 0xaf, 0x0b,
            0xf1, 0x2b, 0x88, 0x1d, 0xc2, 0x00, 0xc9, 0x83, 0x3d, 0xa7, 0x26, 0xe9, 0x37, 0x6c,
            0x2e, 0x32, 0xcf, 0xf7,
        ];
        assert_eq!(hmac_sha256(&key, data), expected);
    }

    #[test]
    fn test_ed25519_sign_verify_roundtrip() {
        let seed = [0x42u8; 32];
        let (pk, sk) = ed25519_keypair_from_seed(&seed);
        let msg = b"AWE_OS 1.0 Production Key Certification";

        let sig = ed25519_sign(&sk, &msg[..]);
        assert!(ed25519_verify(&pk, &msg[..], &sig));

        let mut tampered_msg = msg.to_vec();
        tampered_msg[0] ^= 1;
        assert!(!ed25519_verify(&pk, &tampered_msg[..], &sig));

        let mut tampered_sig = sig;
        tampered_sig[0] ^= 1;
        assert!(!ed25519_verify(&pk, &msg[..], &tampered_sig));
    }
}
