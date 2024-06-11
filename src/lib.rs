pub mod dns;
pub mod http;

use std::{
    fs::File,
    io::{BufReader, Read, Write},
    net::TcpStream,
};

use crate::http::{HTTPRequest, HTTPResponse, Method, Protocol};

pub struct Client {
    dns_client: dns::Resolver,
}

impl Client {
    pub fn new() -> Self {
        Self {
            dns_client: dns::Resolver::new(),
        }
    }
}

impl Client {
    pub fn perform(
        &self,
        method: Method,
        url: String,
        body: Option<String>,
    ) -> Result<String, String> {
        let (protocol, url) = url.split_once("://").unwrap_or(("http", &url));
        let (hostname, url) = match url.split_once("/") {
            Some((hostname, url)) => (hostname, format!("{url}/")),
            None => (url, "/".to_string()),
        };
        // resove IP address
        let id = get_random_u16();
        let addr = self.dns_client.resolve(id, hostname)?;
        let protocol: Protocol = protocol.try_into()?;
        println!("protocol: {:?}, ip address: {:?}", protocol, addr);
        // connet to a server
        let mut stream = match protocol {
            Protocol::HTTP => TcpStream::connect((addr, 80)).map_err(|e| e.to_string())?,
            Protocol::HTTPS => unimplemented!(),
        };
        println!("connection established: {:?}", stream);
        // send HTTP request
        let request = HTTPRequest::new(method, &hostname, &url, body);
        let n = stream
            .write(request.to_string().as_bytes())
            .map_err(|e| e.to_string())?;
        println!("sent {n} bytes");
        // receive HTTP request
        let reader = BufReader::new(stream);
        println!("buf reader created");
        let response = HTTPResponse::try_from(reader)?;
        // let response = reader.
        // let payload: HTTPResponse = reader.lines().map_while(Result::ok).collect();
        println!("{:?}", response);
        Ok("done".to_string())
    }
}

fn get_random_u16() -> u16 {
    // TODO: this should not work on Windows
    let mut file = File::open("/dev/urandom").unwrap();
    let mut buffer = [0u8; 2];
    file.read_exact(&mut buffer).unwrap();
    ((buffer[0] as u16) << 8) + (buffer[1] as u16)
}
