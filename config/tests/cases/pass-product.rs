use config::config;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
struct ProductType {
    key: String,
    value: usize,
}

config! {
    /// A demonstration of a product type assignment in a cvar
    PRODUCT_TYPE: ProductType = ProductType {
        key: "hello".to_string(),
        value: 15
    };
}

fn main() {}
