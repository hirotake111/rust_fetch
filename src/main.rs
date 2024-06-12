use fetch::http::Method;
use fetch::Client;
use std::process::exit;

fn main() {
    let mut args = std::env::args();
    let program = args.next().unwrap();
    if args.len() <= 1 {
        display_usage(&program);
        exit(1);
    }
    let method: Method = args.next().unwrap().parse().unwrap();
    let url = args.next().unwrap();
    let body = args.next();
    let client = Client::new();
    let response = client.perform(method, url, body).unwrap();
    println!("{response}");
}

fn display_usage(program_name: &str) {
    println!(
        "USAGE:
    {program_name} <domain_name>
    "
    );
}
