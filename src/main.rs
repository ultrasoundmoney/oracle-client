mod price_provider;
use price_provider::{PriceProvider, gofer::GoferPriceProvider};

fn main() {
    let gofer = GoferPriceProvider::new(None);
    let price = gofer.get_price();
    println!("Price: {:?}", price);
}
