use std::process::exit;

fn print_help() {
    println!("Usage: arsync [src] [dest]");
}
fn main() {
    let args = std::env::args().collect::<Vec<String>>();
    if args.len() < 3 {
        print_help();
        exit(1);
    }
    println!("copying from {} to {}", args[1], args[2]);
}
