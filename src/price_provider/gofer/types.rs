// Code in this file was generated with: https://transform.tools/json-to-rust-serde
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Root {
    #[serde(rename = "type")]
    pub type_field: String,
    pub base: String,
    pub quote: String,
    pub price: f64,
    pub bid: f64,
    pub ask: f64,
    pub vol24h: i64,
    pub ts: String,
    pub prices: Vec<Price>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Price {
    #[serde(rename = "type")]
    pub type_field: String,
    pub base: String,
    pub quote: String,
    pub price: f64,
    pub bid: f64,
    pub ask: f64,
    pub vol24h: i64,
    pub ts: String,
    pub params: Params,
    pub prices: Vec<Price2>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Params {
    pub method: String,
    pub minimum_successful_sources: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Price2 {
    #[serde(rename = "type")]
    pub type_field: String,
    pub base: String,
    pub quote: String,
    pub price: f64,
    pub bid: f64,
    pub ask: f64,
    pub vol24h: f64,
    pub ts: String,
    pub params: Params2,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Params2 {
    pub origin: String,
}
