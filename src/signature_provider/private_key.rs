use crate::signature_provider::SignatureProvider;
use bls_signatures::{PrivateKey, Signature};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

pub struct PrivateKeySignatureProvider {
    private_key: PrivateKey,
}

impl PrivateKeySignatureProvider {
    pub fn random() -> PrivateKeySignatureProvider {
        let mut rng = ChaCha8Rng::seed_from_u64(12);
        let private_key = PrivateKey::generate(&mut rng);
        PrivateKeySignatureProvider { private_key }
    }
}

impl SignatureProvider for PrivateKeySignatureProvider {
    fn sign(&self, msg: &[u8]) -> Option<Signature> {
        Some(self.private_key.sign(msg))
    }
}
