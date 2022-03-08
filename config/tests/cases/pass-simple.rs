use config::config;

config! {
    /// A simple integer type
    SIMPLE_I32: i32 = 0;

    /// A simple ieee754 type
    SIMPLE_F32: f32 = 0.0;

    /// A simple string type
    SIMPLE_STRING: String = "derp".to_string();
}

fn main() {}
