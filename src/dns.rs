use std::fmt::Display;
use std::net::{IpAddr, SocketAddr, UdpSocket};

///
/// DNS resolver struct that resolve IP address for passed URL
///
pub struct Resolver {}

impl Resolver {
    pub fn new() -> Self {
        Self {}
    }

    pub fn resolve(&self, id: u16, host: &str) -> Result<IpAddr, String> {
        let server: SocketAddr = ([8, 8, 8, 8], 53).into();
        let client = SocketAddr::from(([0, 0, 0, 0], 0));
        let sock = UdpSocket::bind(client).map_err(|err| err.to_string())?;
        let query = Query::new(id, host);
        // let buf: Vec<u8> = query.into();
        sock.send_to(&Vec::from(query), server)
            .map_err(|err| err.to_string())?;
        let mut buf = [0; 512];
        sock.recv_from(&mut buf).map_err(|err| err.to_string())?;
        let response = Response::try_from(&buf)?;

        if response.answers.len() > 0 {
            match response.answers[0].rdata {
                RData::A(v) => Ok(IpAddr::V4(v.into())),
                RData::AAAA(_) => unimplemented!(),
            }
        } else {
            Err("no answers in the DNS response from the server".to_string())
        }
    }
}

/**
 * Query is a query format for DNS communication
 */
#[derive(Debug)]
pub struct Query {
    header: Header,
    questions: Vec<Question>,
}

impl Query {
    pub fn new(id: u16, host: &str) -> Self {
        let header = Header::new_query(id, 1);
        let questions = vec![Question::new(host, QueryType::A)];
        Query { header, questions }
    }
}

impl From<Query> for Vec<u8> {
    fn from(query: Query) -> Self {
        query
            .questions
            .into_iter()
            .fold(query.header.into(), |mut acc, cur| {
                acc.extend::<Vec<u8>>(cur.into());
                acc
            })
    }
}

/**
 * Each DNS message has one header section
 */
#[derive(Debug)]
pub struct Header {
    id: u16,      // transaction ID
    qr: bool,     // 0: query, 1: response
    opcode: u8,   // 0: standard query, 1: inverse query, 2: server status request
    aa: bool,     // 0: not authoritative, 1: authoritative. This bit is valid in responses.
    tc: bool,     // 0: not truncated, 1: truncated
    rd: bool,     // 0: not recursion desired, 1: recursion desired
    ra: bool,     // 0: not recursion available, 1: recursion available
    z: u8,        // reserved for future use. Must be zero in all queries and responses.
    rcode: u8, // 0: no error, 1: format error, 2: server failure, 3: name error, 4: not implemented, 5: refused
    qdcount: u16, // number of question entries
    ancount: u16, // number of answer entries
    nscount: u16, // number of authority entries
    arcount: u16, // number of additional entries
}

#[allow(clippy::too_many_arguments)]
impl Header {
    fn new(
        id: u16,
        qr: bool,
        opcode: u8,
        aa: bool,
        tc: bool,
        rd: bool,
        ra: bool,
        rcode: u8,
        qdcount: u16,
        ancount: u16,
        nscount: u16,
        arcount: u16,
    ) -> Header {
        Header {
            id,
            qr,
            opcode,
            aa,
            tc,
            rd,
            ra,
            z: 0,
            rcode,
            qdcount,
            ancount,
            nscount,
            arcount,
        }
    }
    pub fn new_query(id: u16, qdcount: u16) -> Header {
        Header::new(id, false, 0, false, false, true, false, 0, qdcount, 0, 0, 0)
    }
}

impl From<Header> for Vec<u8> {
    fn from(header: Header) -> Self {
        vec![
            (header.id >> 8) as u8,
            header.id as u8,
            ((header.qr as u8) << 7)
                | (header.opcode << 3)
                | ((header.aa as u8) << 2)
                | ((header.tc as u8) << 1)
                | (header.rd as u8),
            (header.ra as u8) << 7 | (header.z << 4) | (header.rcode),
            (header.qdcount >> 8) as u8,
            header.qdcount as u8,
            (header.ancount >> 8) as u8,
            header.ancount as u8,
            (header.nscount >> 8) as u8,
            header.nscount as u8,
            (header.arcount >> 8) as u8,
            header.arcount as u8,
        ]
    }
}

impl From<&[u8; 512]> for Header {
    fn from(value: &[u8; 512]) -> Self {
        let id = (value[0] as u16) << 8 | value[1] as u16;
        let qr = (value[2] & 0x80) != 0;
        let opcode = (value[2] & 0x78) >> 3;
        let aa = (value[2] & 0x04) != 0;
        let tc = (value[2] & 0x02) != 0;
        let rd = (value[2] & 0x01) != 0;
        let ra = (value[3] & 0x80) != 0;
        let rcode = value[3] & 0x0F;
        let qdcount = (value[4] as u16) << 8 | value[5] as u16;
        let ancount = (value[6] as u16) << 8 | value[7] as u16;
        let nscount = (value[8] as u16) << 8 | value[9] as u16;
        let arcount = (value[10] as u16) << 8 | value[11] as u16;
        Header::new(
            id, qr, opcode, aa, tc, rd, ra, rcode, qdcount, ancount, nscount, arcount,
        )
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Question {
    qname: String,
    qtype: QueryType,
    qclass: QueryClass,
}

impl Question {
    pub fn new(domain: &str, qtype: QueryType) -> Self {
        Question {
            qname: domain.to_string(),
            qtype,
            qclass: QueryClass::IN,
        }
    }
}

impl From<Question> for Vec<u8> {
    fn from(question: Question) -> Self {
        let mut v = vec![];
        for c in question.qname.split('.') {
            v.push(c.len() as u8);
            v.extend(c.bytes());
        }
        v.push(0);
        match question.qtype {
            QueryType::A => v.extend_from_slice(&[0, 1]),
            QueryType::AAAA => v.extend_from_slice(&[0, 28]),
        }
        match question.qclass {
            QueryClass::IN => v.extend_from_slice(&[0, 1]),
        }
        v
    }
}

impl TryFrom<(&[u8; 512], &mut usize)> for Question {
    type Error = String;

    fn try_from((bytes, offset): (&[u8; 512], &mut usize)) -> Result<Self, String> {
        let mut qname: Vec<String> = vec![];
        while bytes[*offset] != 0 {
            let n = bytes[*offset] as usize;
            *offset += 1;
            let end = *offset + n;
            if bytes.len() < end {
                return Err("Invalid question".to_string());
            }
            let label = String::from_utf8_lossy(&bytes[*offset..end]).to_ascii_lowercase();
            qname.push(label);
            *offset = end;
        }
        let qname = qname.join(".");
        *offset += 1;
        let qtype = match (&bytes[*offset], &bytes[*offset + 1]) {
            (0, 1) => QueryType::A,
            (0, 28) => QueryType::AAAA,
            _ => return Err("Invalid question".to_string()),
        };
        *offset += 4;
        Ok(Question {
            qname,
            qtype,
            qclass: QueryClass::IN,
        })
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum QueryType {
    A,
    #[allow(dead_code)]
    AAAA,
}

#[derive(Debug, Eq, PartialEq)]
enum QueryClass {
    IN,
}

#[derive(Debug, PartialEq, Eq)]
pub enum RData {
    A([u8; 4]),
    AAAA([u8; 16]),
}

impl Display for RData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RData::A(ip) => write!(
                f,
                "{}",
                ip.iter()
                    .map(|b| b.to_string())
                    .collect::<Vec<String>>()
                    .join(".")
            ),
            RData::AAAA(ip) => write!(
                f,
                "{}",
                ip.iter()
                    .map(|b| b.to_string())
                    .collect::<Vec<String>>()
                    .join(":")
            ),
        }
    }
}

/**
 * Response contains header, question, answer, and possibly authority and additional sections
 */
#[derive(Debug)]
pub struct Response {
    pub header: Header,
    pub questions: Vec<Question>,
    pub answers: Vec<ResourceRecord>,
    pub authorities: Vec<ResourceRecord>,
    pub additionals: Vec<ResourceRecord>,
}

impl TryFrom<&[u8; 512]> for Response {
    type Error = String;

    fn try_from(value: &[u8; 512]) -> Result<Self, String> {
        let header = Header::from(value);
        let mut questions = vec![];
        let mut answers = vec![];
        let mut authorities = vec![];
        let mut additionals = vec![];
        let mut offset = 12; // header is always 6x2 bytes
        for _ in 0..header.qdcount {
            questions.push(Question::try_from((value, &mut offset))?);
        }
        for _ in 0..header.ancount {
            answers.push(ResourceRecord::try_from((value, &mut offset))?);
        }
        for _ in 0..header.nscount {
            authorities.push(ResourceRecord::try_from((value, &mut offset))?);
        }
        for _ in 0..header.arcount {
            additionals.push(ResourceRecord::try_from((value, &mut offset))?);
        }

        Ok(Response {
            header,
            questions,
            answers,
            authorities,
            additionals,
        })
    }
}
/**
 * 4.1.3. Resource record format

The answer, authority, and additional sections all share the same
format: a variable number of resource records, where the number of
records is specified in the corresponding count field in the header.
Each resource record has the following format:
                                    1  1  1  1  1  1
      0  1  2  3  4  5  6  7  8  9  0  1  2  3  4  5
    +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
    |                                               |
    /                                               /
    /                      NAME                     /
    |                                               |
    +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
    |                      TYPE                     |
    +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
    |                     CLASS                     |
    +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
    |                      TTL                      |
    |                                               |
    +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
    |                   RDLENGTH                    |
    +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--|
    /                     RDATA                     /
    /                                               /
    +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
 */
#[derive(Debug, PartialEq, Eq)]
pub struct ResourceRecord {
    pub name: String,
    pub query_type: QueryType,
    query_class: QueryClass,
    pub ttl: u32,
    pub rdlength: u16,
    pub rdata: RData,
}

impl ResourceRecord {
    #[allow(dead_code)]
    fn new(
        name: String,
        query_type: QueryType,
        query_class: QueryClass,
        ttl: u32,
        rdlength: u16,
        rdata: RData,
    ) -> Self {
        ResourceRecord {
            name,
            query_type,
            query_class,
            ttl,
            rdlength,
            rdata,
        }
    }
}

impl TryFrom<(&[u8; 512], &mut usize)> for ResourceRecord {
    type Error = String;

    fn try_from((bytes, offset): (&[u8; 512], &mut usize)) -> Result<Self, Self::Error> {
        if *offset + 12 >= bytes.len() {
            return Err("data is too short".to_string());
        }
        let name = if bytes[*offset] == 192 {
            // message compression
            let mut tmp_offset = bytes[*offset + 1] as usize;
            *offset += 2;
            get_name(bytes, &mut tmp_offset)?
        } else {
            get_name(bytes, offset)?
        };
        let query_type = match ((bytes[*offset] as u16) << 8) + (bytes[*offset + 1] as u16) {
            1 => QueryType::A,
            28 => QueryType::AAAA,
            _ => panic!(),
        };
        *offset += 2;
        let query_class = QueryClass::IN;
        *offset += 2;
        let ttl = ((bytes[*offset] as u32) << 24)
            + ((bytes[*offset + 1] as u32) << 16)
            + ((bytes[*offset + 2] as u32) << 8)
            + (bytes[*offset + 3] as u32);
        *offset += 4;
        let rdlength = ((bytes[*offset] as u16) << 8) + bytes[*offset + 1] as u16;
        *offset += 2;
        let rdata = match query_type {
            QueryType::A => RData::A([
                bytes[*offset],
                bytes[*offset + 1],
                bytes[*offset + 2],
                bytes[*offset + 3],
            ]),
            _ => unimplemented!(),
        };
        *offset += rdlength as usize;

        Ok(ResourceRecord {
            name,
            query_type,
            query_class,
            ttl,
            rdlength,
            rdata,
        })
    }
}

fn get_name(bytes: &[u8; 512], offset: &mut usize) -> Result<String, String> {
    let mut name: Vec<String> = vec![];
    while bytes[*offset] != 0 {
        let n = bytes[*offset] as usize;
        *offset += 1;
        let end = *offset + n;
        if bytes.len() < end {
            return Err("Invalid name".to_string());
        }
        let label = String::from_utf8_lossy(&bytes[*offset..end]).to_ascii_lowercase();
        name.push(label);
        *offset = end;
    }
    *offset += 1;
    let name = name.join(".");
    Ok(name)
}

#[cfg(test)]
mod tests {
    use crate::dns::*;

    #[test]
    fn test_from_bytes_to_question() {
        let mut bytes: [u8; 512] = [0; 512];
        let initial_offset = 10;
        let mut offset: usize = initial_offset;
        let header_payload = [
            // example.com: type A, class IN
            0x07, 0x65, 0x78, 0x61, 0x6d, 0x70, 0x6c, 0x65, 0x03, 0x63, 0x6f, 0x6d, 0x00, 0x00,
            0x01, 0x00, 0x01,
        ];
        // fill header payload into zero bytes buffer
        for byte in header_payload {
            bytes[offset] = byte;
            offset += 1;
        }
        offset = initial_offset; // reset offset
        assert_eq!(
            Question::try_from((&bytes, &mut offset)),
            Ok(Question::new("example.com", QueryType::A)),
        );
        assert_eq!(offset, header_payload.len() + initial_offset);
    }

    #[test]
    fn test_from_bytes_to_resource_record() {
        let mut bytes: [u8; 512] = [0; 512];
        let mut start_offset: usize = 28;
        let mut offset = 0;
        let rr_payload = [
            0x9e, 0xd9, 0x81, 0x80, 0x00, 0x01, 0x00, 0x02, 0x00, 0x00, 0x00, 0x00, 0x03, 0x64,
            0x6e, 0x73, 0x06, 0x67, 0x6f, 0x6f, 0x67, 0x6c, 0x65, 0x00, 0x00, 0x01, 0x00, 0x01,
            0xc0, 0x0c, 0x00, 0x01, 0x00, 0x01, 0x00, 0x00, 0x02, 0xb3, 0x00, 0x04, 0x08, 0x08,
            0x04, 0x04, 0xc0, 0x0c, 0x00, 0x01, 0x00, 0x01, 0x00, 0x00, 0x02, 0xb3, 0x00, 0x04,
            0x08, 0x08, 0x08, 0x08,
        ];
        // fill header payload into zero bytes buffer
        for byte in rr_payload {
            bytes[offset] = byte;
            offset += 1;
        }
        assert_eq!(
            ResourceRecord::try_from((&bytes, &mut start_offset)),
            Ok(ResourceRecord::new(
                "dns.google".to_string(),
                QueryType::A,
                QueryClass::IN,
                691,
                4,
                RData::A([8, 8, 4, 4])
            )),
        );
        assert_eq!(offset, rr_payload.len());
    }

    #[test]
    fn test_dns_resolver() {
        let resolver = Resolver::new();
        // FIXME: flaky test
        let result = resolver.resolve(123, "example.com");
        assert!(result.is_ok());
    }
}
