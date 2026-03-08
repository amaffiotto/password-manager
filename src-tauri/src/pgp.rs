use pgp::composed::key::SecretKeyParamsBuilder;
use pgp::composed::{KeyType, Message, SignedPublicKey, SignedSecretKey};
use pgp::crypto::{hash::HashAlgorithm, sym::SymmetricKeyAlgorithm};
use pgp::types::{CompressionAlgorithm, PublicKeyTrait, SecretKeyTrait};
use pgp::ArmorOptions;
use pgp::Deserializable;
use rand::thread_rng;
use smallvec::smallvec;
use std::io::Cursor;

/// Result of PGP key generation.
#[derive(serde::Serialize)]
pub struct PgpKeyPair {
    pub public_key_armored: String,
    pub private_key_armored: String,
    pub fingerprint: String,
}

/// Generate a PGP key pair.
/// key_type: "rsa4096" or "ed25519"
pub fn generate_key_pair(
    name: &str,
    email: &str,
    key_type: &str,
    passphrase: &str,
) -> Result<PgpKeyPair, String> {
    let mut rng = thread_rng();

    let kt = match key_type {
        "rsa4096" => KeyType::Rsa(4096),
        "ed25519" => KeyType::EdDSALegacy,
        _ => return Err(format!("Unsupported key type: {}", key_type)),
    };

    let sub_kt = match key_type {
        "rsa4096" => KeyType::Rsa(4096),
        "ed25519" => KeyType::ECDH(pgp::crypto::ecc_curve::ECCCurve::Curve25519),
        _ => unreachable!(),
    };

    let secret_key_params = SecretKeyParamsBuilder::default()
        .key_type(kt)
        .can_certify(true)
        .can_sign(true)
        .primary_user_id(format!("{} <{}>", name, email))
        .preferred_symmetric_algorithms(smallvec![SymmetricKeyAlgorithm::AES256])
        .preferred_hash_algorithms(smallvec![HashAlgorithm::SHA2_256])
        .preferred_compression_algorithms(smallvec![CompressionAlgorithm::ZLIB])
        .subkey(
            pgp::composed::key::SubkeyParamsBuilder::default()
                .key_type(sub_kt)
                .can_encrypt(true)
                .build()
                .map_err(|e| e.to_string())?,
        )
        .passphrase(Some(passphrase.to_string()))
        .build()
        .map_err(|e| e.to_string())?;

    let secret_key = secret_key_params
        .generate(&mut rng)
        .map_err(|e| e.to_string())?;

    let signed_secret_key = secret_key
        .sign(&mut rng, || passphrase.to_string())
        .map_err(|e| e.to_string())?;

    let public_key = signed_secret_key.public_key();
    let signed_public_key = public_key
        .sign(&mut rng, &signed_secret_key, || passphrase.to_string())
        .map_err(|e| e.to_string())?;

    let fingerprint = format!("{:?}", signed_public_key.fingerprint());

    let public_key_armored = signed_public_key
        .to_armored_string(ArmorOptions::default())
        .map_err(|e| e.to_string())?;

    let private_key_armored = signed_secret_key
        .to_armored_string(ArmorOptions::default())
        .map_err(|e| e.to_string())?;

    Ok(PgpKeyPair {
        public_key_armored,
        private_key_armored,
        fingerprint,
    })
}

/// Encrypt a message with a PGP public key.
pub fn encrypt_message(plaintext: &str, public_key_armored: &str) -> Result<String, String> {
    let mut rng = thread_rng();

    let (public_key, _) =
        SignedPublicKey::from_armor_single(Cursor::new(public_key_armored.as_bytes()))
            .map_err(|e| e.to_string())?;

    let msg = Message::new_literal("msg", plaintext);
    let encrypted = msg
        .encrypt_to_keys_seipdv1(&mut rng, SymmetricKeyAlgorithm::AES256, &[&public_key])
        .map_err(|e: pgp::errors::Error| e.to_string())?;

    encrypted
        .to_armored_string(ArmorOptions::default())
        .map_err(|e: pgp::errors::Error| e.to_string())
}

/// Decrypt a PGP-encrypted message with a private key.
pub fn decrypt_message(
    encrypted_armored: &str,
    private_key_armored: &str,
    passphrase: &str,
) -> Result<String, String> {
    let (secret_key, _) =
        SignedSecretKey::from_armor_single(Cursor::new(private_key_armored.as_bytes()))
            .map_err(|e| e.to_string())?;

    let (msg, _) = Message::from_armor_single(Cursor::new(encrypted_armored.as_bytes()))
        .map_err(|e| e.to_string())?;

    let (decrypted_msg, _) = msg
        .decrypt(|| passphrase.to_string(), &[&secret_key])
        .map_err(|e| e.to_string())?;

    // The decrypted message should be a literal data message
    match decrypted_msg {
        Message::Literal(data) => {
            String::from_utf8(data.data().to_vec()).map_err(|e| e.to_string())
        }
        Message::Compressed(compressed) => {
            // Read the compressed data
            let mut decompressor = compressed.decompress().map_err(|e| e.to_string())?;
            let mut buf = Vec::new();
            std::io::Read::read_to_end(&mut decompressor, &mut buf).map_err(|e| e.to_string())?;
            // Try to parse as a literal message
            String::from_utf8(buf).map_err(|e| e.to_string())
        }
        _ => Err("No plaintext found in decrypted message".to_string()),
    }
}

/// Sign a message with a PGP private key. Returns armored signed message.
pub fn sign_message(
    message: &str,
    private_key_armored: &str,
    passphrase: &str,
) -> Result<String, String> {
    let mut rng = thread_rng();

    let (secret_key, _) =
        SignedSecretKey::from_armor_single(Cursor::new(private_key_armored.as_bytes()))
            .map_err(|e| e.to_string())?;

    let msg = Message::new_literal("msg", message);
    let signed = msg
        .sign(&mut rng, &secret_key, || passphrase.to_string(), HashAlgorithm::SHA2_256)
        .map_err(|e| e.to_string())?;

    signed
        .to_armored_string(ArmorOptions::default())
        .map_err(|e: pgp::errors::Error| e.to_string())
}

/// Verify a signed PGP message against a public key.
/// Returns (is_valid, message_content).
pub fn verify_message(
    signed_armored: &str,
    public_key_armored: &str,
) -> Result<(bool, String), String> {
    let (public_key, _) =
        SignedPublicKey::from_armor_single(Cursor::new(public_key_armored.as_bytes()))
            .map_err(|e| e.to_string())?;

    let (msg, _) = Message::from_armor_single(Cursor::new(signed_armored.as_bytes()))
        .map_err(|e| e.to_string())?;

    let verified = msg.verify(&public_key).is_ok();

    // Extract the literal data
    let content = match &msg {
        Message::Literal(data) => {
            String::from_utf8(data.data().to_vec()).unwrap_or_default()
        }
        Message::Signed { message, .. } => {
            if let Some(inner) = message {
                if let Message::Literal(data) = inner.as_ref() {
                    String::from_utf8(data.data().to_vec()).unwrap_or_default()
                } else {
                    String::new()
                }
            } else {
                String::new()
            }
        }
        _ => String::new(),
    };

    Ok((verified, content))
}
