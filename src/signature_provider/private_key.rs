use eyre::Result;
use sha3::{Sha3_256, Digest};

use crate::signature_provider::SignatureProvider;
use bls::{Hash256, SecretKey, Signature};

pub struct PrivateKeySignatureProvider {
    private_key: SecretKey,
}

impl PrivateKeySignatureProvider {
    pub fn random() -> PrivateKeySignatureProvider {
        let private_key = SecretKey::random();
        log::debug!(
            "Generated random private key associated with public key: {:?}",
            private_key.public_key()
        );
        PrivateKeySignatureProvider { private_key }
    }
}

impl SignatureProvider for PrivateKeySignatureProvider {
    fn sign(&self, msg: &[u8]) -> Result<Signature> {
        let msg_hash = Hash256::from_slice(&Sha3_256::digest(msg));
        Ok(self.private_key.sign(msg_hash))
    }
}
