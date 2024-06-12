# fetch

A dead simple HTTP client with no 3rd party library dependencies.
Currently it has limited capabilities as follows:

- Limited HTTP/1.1 support (no keep-alive, etc.)
- IPv4 support
- no localhost lookup

## Usage in bash

```bash
fetch get example.com
```

## API Usage example

```rust
use fetch::{http::Method, Client};

let client = Client::new(); // initialize HTTP client
let response = client.perform(Method::GET, "http://example.com".to_string(), None); // perform HTTP request
println!("{response:?}");
```
