use fetch::{http::Method, Client};

fn main() {
    let client = Client::new();
    let response = client.perform(Method::GET, "http://example.com".to_string(), None);
    println!("{response:?}");
}
