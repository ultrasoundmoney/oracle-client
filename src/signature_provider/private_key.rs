use eyre::Result;
use sha3::{Digest, Sha3_256};

use crate::signature_provider::SignatureProvider;
use bls::{AggregateSignature, Hash256, PublicKey, SecretKey, Signature};

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

    pub fn get_message_digest(&self, msg: &[u8]) -> Hash256 {
        Hash256::from_slice(&Sha3_256::digest(msg))
    }
}

impl Clone for PrivateKeySignatureProvider {
    fn clone(&self) -> Self {
        PrivateKeySignatureProvider {
            private_key: self.private_key.clone(),
        }
    }
}

impl SignatureProvider for PrivateKeySignatureProvider {
    fn sign(&self, msg: &[u8]) -> Result<Signature> {
        Ok(self.private_key.sign(self.get_message_digest(msg)))
    }

    fn get_public_key(&self) -> Result<PublicKey> {
        Ok(self.private_key.public_key())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn can_verify_signed_message() {
        let msg = b"Hello, world!";
        let signature_provider = PrivateKeySignatureProvider::random();
        let signature = signature_provider.sign(msg).unwrap();
        let public_key = signature_provider.get_public_key().unwrap();
        assert!(signature.verify(&public_key, signature_provider.get_message_digest(msg)));
    }

    #[tokio::test]
    async fn can_aggregate_signatures_from_multiple_signers() {
        let msg = b"Hello, world!";
        let num_signatures = 5;

        let signature_providers = (0..num_signatures)
            .map(|_| PrivateKeySignatureProvider::random())
            .collect::<Vec<_>>();

        let msg_hash = signature_providers[0].get_message_digest(msg);

        let signatures = signature_providers
            .iter()
            .map(|signature_provider| signature_provider.sign(msg).unwrap())
            .collect::<Vec<_>>();

        let pub_keys = signature_providers
            .iter()
            .map(|signature_provider| signature_provider.get_public_key().unwrap())
            .collect::<Vec<_>>();

        let pub_keys_refs = pub_keys.iter().collect::<Vec<_>>();

        let mut aggregate_signature = AggregateSignature::infinity();
        signatures.iter().for_each(|signature| {
            aggregate_signature.add_assign(&signature);
        });

        assert!(aggregate_signature.fast_aggregate_verify(msg_hash, &pub_keys_refs));
    }
}
