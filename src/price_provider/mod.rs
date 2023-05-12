pub mod gofer;

#[derive(Debug)]
pub struct Price {
    pub value: f64
}

pub trait PriceProvider {
    fn get_price(&self) -> Option<Price>;
}
