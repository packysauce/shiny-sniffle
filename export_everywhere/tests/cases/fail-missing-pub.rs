use export_everywhere::export_everywhere;

#[export_everywhere]
extern fn my_function() {
    println!("I do stuff");
}

fn main() {}
