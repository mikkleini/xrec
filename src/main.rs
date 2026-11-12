use std::env;

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    if let Err(error) = xrec::run(&args) {
        eprintln!("xrec: {error}");
        std::process::exit(2);
    }
}
