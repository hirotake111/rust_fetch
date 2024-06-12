#![allow(dead_code)]

use std::{
    collections::HashMap,
    fmt::Display,
    io::{BufRead, BufReader, Read, Write},
    str::FromStr,
};

#[derive(Debug, Clone)]
pub enum Protocol {
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

#[derive(Debug, Clone)]
pub struct HTTPRequest {
    request_line: RequestLine,
    headers: HTTPHeaders,
    body: Option<String>,
}

impl HTTPRequest {
    pub fn new(method: Method, hostname: &str, url: &str, body: Option<String>) -> Self {
        let request_line = RequestLine::new(method, url);
        let headers: HTTPHeaders = vec![("Host".to_string(), hostname.to_string())].into();
        Self {
            request_line,
            headers,
            body,
        }
    }
}

impl Display for HTTPRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.body {
            Some(body) => write!(f, "{}{}\r\n{}\r\n", self.request_line, self.headers, body),
            None => write!(f, "{}{}\r\n", self.request_line, self.headers),
        }
    }
}

impl<R: Read> TryFrom<BufReader<R>> for HTTPRequest {
    type Error = String;

    fn try_from(reader: BufReader<R>) -> Result<Self, Self::Error> {
        let mut iterator = reader.lines().map_while(Result::ok).peekable();
        let request_line = iterator
            .next()
            .ok_or("failed to get request line")?
            .parse()?;
        let headers = HTTPHeaders::new(&mut iterator)?;
        let body = if iterator.peek().is_some() {
            Some(iterator.collect())
        } else {
            None
        };

        Ok(HTTPRequest {
            request_line,
            headers,
            body,
        })
    }
}

#[derive(Debug, Clone)]
pub struct RequestLine {
    method: Method,
    request_target: String,
    http_version: String,
}

impl RequestLine {
    pub fn new(method: Method, url: &str) -> Self {
        Self {
            method,
            request_target: url.to_string(),
            http_version: "HTTP/1.1".to_string(),
        }
    }
}

impl Display for RequestLine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {} {}\r\n",
            self.method, self.request_target, self.http_version
        )
    }
}

impl FromStr for RequestLine {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut iterator = s.split(' ');
        let method: Method = iterator
            .next()
            .ok_or("failed to get HTTP method")?
            .parse()?;
        let request_target = iterator
            .next()
            .ok_or("failed to get request target")?
            .to_string();
        let http_version = iterator
            .next()
            .ok_or("failed to get HTTP version")?
            .to_string();
        Ok(RequestLine {
            method,
            request_target,
            http_version,
        })
    }
}

#[derive(Debug, Clone)]
struct HTTPHeaders(HashMap<String, String>);

impl From<Vec<(String, String)>> for HTTPHeaders {
    fn from(value: Vec<(String, String)>) -> Self {
        let hm = value.into_iter().collect::<HashMap<String, String>>();
        HTTPHeaders(hm)
    }
}

impl Display for HTTPHeaders {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = String::from("");
        for (k, v) in &self.0 {
            s += &format!("{}: {}\r\n", k, v);
        }
        write!(f, "{}", s)
    }
}

impl HTTPHeaders {
    pub fn new(iterator: &mut impl Iterator<Item = String>) -> Result<HTTPHeaders, String> {
        let mut headers = HashMap::new();
        for line in iterator {
            if line.is_empty() {
                break;
            }
            let mut line = line.split(": ");
            let key = line.next().ok_or("failed to get key")?.trim().to_string();
            let value = line
                .next()
                .ok_or(format!("failed to get value for key: {key}"))?
                .to_string();
            headers.insert(key, value);
        }
        Ok(HTTPHeaders(headers))
    }
}

#[derive(Debug, Clone)]
pub enum Method {
    GET,
    POST,
    HEAD,
    OPTIONS,
    DELETE,
    PUT,
    CONNECT,
    TRACE,
}

impl Display for Method {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Method::GET => "GET",
            Method::POST => "POST",
            Method::PUT => "PUT",
            Method::DELETE => "DELETE",
            Method::HEAD => "HEAD",
            Method::CONNECT => "CONNECT",
            Method::TRACE => "TRACE",
            Method::OPTIONS => "OPTIONS",
        };
        write!(f, "{}", s)
    }
}
// impl Method {
//     fn to_string(&self) -> String {
//         let s = match self {
//             Method::GET => "GET",
//             Method::POST => "POST",
//             Method::PUT => "PUT",
//             Method::DELETE => "DELETE",
//             Method::HEAD => "HEAD",
//             Method::CONNECT => "CONNECT",
//             Method::TRACE => "TRACE",
//             Method::OPTIONS => "OPTIONS",
//         };
//         s.to_string()
//     }
// }

impl FromStr for Method {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "GET" | "get" => Ok(Method::GET),
            "POST" | "post" => Ok(Method::POST),
            "PUT" | "put" => Ok(Method::PUT),
            "DELETE" | "delete" => Ok(Method::DELETE),
            "HEAD" | "head" => Ok(Method::HEAD),
            "OPTIONS" | "options" => Ok(Method::OPTIONS),
            "CONNECT" | "connect" => Ok(Method::CONNECT),
            "TRACE" | "trace" => Ok(Method::TRACE),
            _ => Err(format!("invalid HTTP method: {s}")),
        }
    }
}

#[derive(Debug, Clone)]
pub struct HTTPResponse {
    status_line: StatusLine,
    headers: HTTPHeaders,
    pub body: Option<String>,
}

impl<R: Read> TryFrom<BufReader<R>> for HTTPResponse {
    type Error = String;

    fn try_from(reader: BufReader<R>) -> Result<Self, Self::Error> {
        let mut iterator = reader.lines().map_while(Result::ok).peekable();
        let status_line: StatusLine = iterator
            .next()
            .ok_or("failed to get status line")?
            .parse()?;
        let headers = HTTPHeaders::new(&mut iterator)?;
        let mut length = headers
            .0
            .get("Content-Length")
            .ok_or("no content-length header in the respnse from the server")?
            .parse::<usize>()
            .map_err(|e| e.to_string())?;
        let mut body = vec![];
        for data in iterator {
            println!("{}, {}", length, data.len());
            println!("{}", data);
            println!("{:?}", data.as_bytes());
            if data.is_empty() || data.len() >= length {
                break;
            }
            length -= data.len() + 1;
            body.push(data);
        }
        Ok(HTTPResponse {
            status_line,
            headers,
            body: Some(body.join("\n")),
        })
    }
}

#[derive(Debug, Clone)]
pub struct StatusLine {
    http_version: HTTPVersion,
    status_code: StatusCode,
    status_text: String,
}

impl FromStr for StatusLine {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut iterator = s.split(' ');
        let http_version: HTTPVersion = iterator
            .next()
            .ok_or("failed to get HTTP version")?
            .parse()?;
        let status_code: StatusCode = iterator
            .next()
            .ok_or("no status code to be parsed")?
            .parse()?;
        let status_text = iterator
            .next()
            .ok_or("failed to get status text")?
            .to_string();
        Ok(StatusLine {
            http_version,
            status_code,
            status_text,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
struct HTTPVersion(String);

impl TryFrom<&[u8]> for HTTPVersion {
    type Error = String;

    fn try_from(v: &[u8]) -> Result<Self, Self::Error> {
        let s = String::from_utf8(v.to_vec()).map_err(|e| e.to_string())?;
        if s.starts_with("HTTP/") {
            Ok(HTTPVersion(s))
        } else {
            Err(format!("invalid HTTP Version: {}", s))
        }
    }
}

impl FromStr for HTTPVersion {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.starts_with("HTTP/") {
            Ok(HTTPVersion(s.to_string()))
        } else {
            Err(format!("invalid HTTP Version: {}", s))
        }
    }
}

#[derive(Debug, Clone)]
struct StatusCode(u16);
impl FromStr for StatusCode {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.parse::<u16>()
            .or(Err(format!("error parsing status code: {}", s)))
            .map(StatusCode)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_http_request_line_to_string() {
        let req_line = RequestLine::new(Method::GET, "/hello");
        assert_eq!(req_line.to_string(), "GET /hello HTTP/1.1\r\n".to_string());
    }

    #[test]
    fn test_http_header_to_string() {
        let headers = HTTPHeaders::from(vec![("foo".to_string(), "bar".to_string())]);
        assert_eq!(headers.to_string(), "foo: bar\r\n".to_string());
        let headers = HTTPHeaders::from(vec![
            ("foo".to_string(), "bar".to_string()),
            ("age".to_string(), "55".to_string()),
        ]);
        // assert_eq!(headers.to_string(), "foo: bar\r\nage: 55\r\n".to_string());
        assert!(headers.to_string().contains("foo: bar\r\n"));
        assert!(headers.to_string().contains("age: 55\r\n"));
    }

    #[test]
    fn test_http_request_to_string() {
        let req = HTTPRequest::new(Method::GET, "example.com", "/hello", None);
        assert_eq!(
            req.to_string(),
            "GET /hello HTTP/1.1\r\nHost: example.com\r\n\r\n".to_string()
        );
    }

    #[test]
    fn test_http_version_from_vecu8() {
        let v: &[u8] = b"HTTP/1.1";
        assert_eq!(
            Ok(HTTPVersion(String::from("HTTP/1.1"))),
            HTTPVersion::try_from(v)
        );
    }
}
