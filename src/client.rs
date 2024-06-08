use std::{
    io::{BufRead, BufReader},
    net::{Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, TcpListener, TcpStream},
};

// - client constructor
// - URL parser
// - client.get API
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
        let (protocol, url) = url
            .split_once("://")
            .ok_or(format!("invalid URL passed: {url}"))?;
        // resove IP address
        let addr = self.dns_client.resolve(url)?;
        println!("ip address: {:?}", addr);
        // let protocol: Protocol = protocol.try_into()?;
        let stream = match protocol.try_into()? {
            Protocol::HTTP => TcpStream::connect((addr, 80)).or_else(|e| Err(e.to_string()))?,
            Protocol::HTTPS => unimplemented!(),
        };
        let reader = BufReader::new(stream);
        let payload = reader.lines().map_while(Result::ok).collect();
        Ok(payload)
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
