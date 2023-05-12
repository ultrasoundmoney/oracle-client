mod price_provider;
use price_provider::{Price, PriceProvider, gofer::GoferPriceProvider};
mod signature_provider;
use signature_provider::{SignatureProvider, private_key::PrivateKeySignatureProvider};

use ssz::{Decode, Encode};

fn main() {
    let gofer = GoferPriceProvider::new(None);
    let price = gofer.get_price();
    println!("Price: {:?}", price);
    let price_ssz: Vec<u8> = price.as_ssz_bytes();
    println!("Price encoded: {:?}", price_ssz);
    // TODO: Why do I have to remove the first byte when decoding? (otherwise I get a panic)
    let price_decoded = Price::from_ssz_bytes(&price_ssz[1..]).unwrap();
    println!("Price decoded: {:?}", price_decoded);

    let signature_provider = PrivateKeySignatureProvider::random();
    let signature = signature_provider.sign(&price_ssz);

    println!("Signature: {:?}", signature);
}
