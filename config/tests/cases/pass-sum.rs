use config::config;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
enum SumType {
    Simple,
    Wrappy(usize),
    Structy { key: String, value: usize },
}

config! {
    /// A demonstration of a sum type assignment in a cvar
    SIMPLE: SumType = SumType::Simple;
    /// A demonstration of a sum type assignment in a cvar
    WRAPPY: SumType = SumType::Wrappy(400);
    /// A demonstration of a sum type assignment in a cvar
    STRUCTY: SumType = SumType::Structy {
        key: "hello".to_string(),
        value: 15
    };
}

fn main() {}
