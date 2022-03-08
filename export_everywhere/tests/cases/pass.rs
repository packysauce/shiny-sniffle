use export_everywhere::export_everywhere;

#[export_everywhere]
pub extern fn my_function() {
    println!("I do stuff");
}

fn main() {}
