use std::{
    fs::File,
    io::{BufReader, Read},
    net::TcpStream,
};

use crate::dns;

pub struct Client {
    dns_client: dns::Resolver,
}

pub fn new() -> Client {
    Client {
        dns_client: dns::Resolver::new(),
    }
}

impl Client {
    pub fn get(&self, url: String) -> Result<String, String> {
        let (protocol, host) = url
            .split_once("://")
            .ok_or(format!("invalid URL passed: {url}"))?;
        // resove IP address
        let id = get_random_u16();
        let addr = self.dns_client.resolve(id, host)?;
        println!("ip address: {:?}", addr);
        let protocol: Protocol = protocol.try_into()?;
        let stream = match protocol {
            Protocol::HTTP => TcpStream::connect((addr, 80)).or_else(|e| Err(e.to_string()))?,
            Protocol::HTTPS => unimplemented!(),
        };
        let _reader = BufReader::new(stream);
        // let payload: HTTPResponse = reader.lines().map_while(Result::ok).collect();
        Ok("done".to_string())
    }
}

enum Protocol {
    HTTP,
    HTTPS,
}

impl TryFrom<&str> for Protocol {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "http" => Ok(Protocol::HTTP),
            "https" => Ok(Protocol::HTTPS),
            _ => Err("invalid protocol schema".to_string()),
        }
    }
}

fn get_random_u16() -> u16 {
    // TODO: this should not work on Windows
    let mut file = File::open("/dev/urandom").unwrap();
    let mut buffer = [0u8; 2];
    file.read_exact(&mut buffer).unwrap();
    ((buffer[0] as u16) << 8) + (buffer[1] as u16)
}
