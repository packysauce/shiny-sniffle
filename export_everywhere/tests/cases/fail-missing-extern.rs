use export_everywhere::export_everywhere;

#[export_everywhere]
pub fn my_function() {
    println!("I do stuff");
}

fn main() {}
