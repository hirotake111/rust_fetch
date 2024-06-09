use rust_fetch::client;
use std::process::exit;

fn main() {
    let mut args = std::env::args();
    let program = args.next().unwrap();
    let url = args.next().unwrap_or_else(|| {
        display_usage(&program);
        exit(1);
    });
    let client = client::new();
    let response = client.get(url).unwrap();

    println!("{response}");
}

fn display_usage(program_name: &str) {
    println!(
        "USAGE:
    {program_name} <domain_name>
    "
    );
}
