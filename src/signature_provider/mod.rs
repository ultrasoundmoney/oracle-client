use eyre::Result;

use bls::{PublicKey, Signature};
pub mod private_key;

pub trait SignatureProvider {
    fn sign(&self, msg: &[u8]) -> Result<Signature>;
    fn get_public_key(&self) -> Result<PublicKey>;
    fn clone(&self) -> Box<dyn SignatureProvider + std::marker::Send + std::marker::Sync + 'static>;
}
