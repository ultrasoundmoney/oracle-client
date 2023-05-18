use eyre::Result;

use bls_signatures::Signature;
pub mod private_key;

pub trait SignatureProvider {
    fn sign(&self, msg: &[u8]) -> Result<Signature>;
}
