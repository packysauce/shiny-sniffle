use config::config;

type TypedefVector = Vec<usize>;

config! {
    /// Typedef'd vector as a cvar
    TD_VECTOR: TypedefVector = TypedefVector::new();

    /// A vector as a cvar
    VECTOR: Vec<usize> = Vec::new();

    /// Macro-initialized vector as a cvar
    MACRO_INIT_VECTOR: Vec<String> = vec!["hello world".to_string()];

    /// A pretty sweet map
    MY_MAP: std::collections::BTreeMap<String, i32> = {
        let mut map = std::collections::BTreeMap::new();
        map.insert("val".to_string(), 15);
        map
    };
}

fn main() {}
