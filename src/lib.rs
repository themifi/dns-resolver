use rand::prelude::*;

pub fn resolve_domain(domain: String) -> std::net::Ipv4Addr {
    let mut ns_server = std::net::Ipv4Addr::new(198, 41, 0, 4);
    loop {
        println!("Querying {} for {}", ns_server, domain);
        let packet = send_query(ns_server, &domain, 1);
        if let Some(ip) = packet.get_ip() {
            return ip;
        } else if let Some(ns_ip) = packet.get_nameserver_ip() {
            ns_server = ns_ip;
        } else if let Some(ns_domain) = packet.get_nameserver() {
            ns_server = resolve_domain(ns_domain);
        } else {
            panic!("No IP address found for {}", domain);
        }
    }
}

fn send_query(addr: std::net::Ipv4Addr, domain: &str, record_type: u16) -> DNSPacket {
    let query = build_query(domain.to_owned(), record_type);
    let sock = std::net::UdpSocket::bind("0.0.0.0:12000").unwrap();
    let socket_addr = std::net::SocketAddrV4::new(addr, 53);
    sock.send_to(&query, socket_addr).unwrap();
    let mut response = [0; 1024];
    sock.recv(&mut response).unwrap();

    let mut reader = std::io::Cursor::new(&response);
    DNSPacket::parse(&mut reader)
}

fn build_query(domain_name: String, record_type: u16) -> Vec<u8> {
    let id = random();
    let flags = 0;
    let header = DNSHeader {
        id,
        flags,
        num_questions: 1,
        num_answers: 0,
        num_authorities: 0,
        num_additionals: 0,
    };
    let question = DNSQuestion {
        name: domain_name,
        type_: record_type,
        class: 1,
    };
    let mut bytes = header.to_be_bytes();
    bytes.extend(question.to_be_bytes());
    bytes
}

trait SeekReader: std::io::Read + std::io::Seek {}
impl<T: std::io::Read + std::io::Seek> SeekReader for T {}

#[derive(Debug, PartialEq)]
struct DNSHeader {
    id: u16,
    flags: u16,
    num_questions: u16,
    num_answers: u16,
    num_authorities: u16,
    num_additionals: u16,
}

impl DNSHeader {
    fn to_be_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::<u8>::new();
        bytes.extend(self.id.to_be_bytes());
        bytes.extend(self.flags.to_be_bytes());
        bytes.extend(self.num_questions.to_be_bytes());
        bytes.extend(self.num_answers.to_be_bytes());
        bytes.extend(self.num_authorities.to_be_bytes());
        bytes.extend(self.num_additionals.to_be_bytes());
        bytes
    }

    fn parse(reader: &mut dyn std::io::Read) -> Self {
        let mut buffer = [0; 12];
        reader.read_exact(&mut buffer).unwrap();
        Self {
            id: u16::from_be_bytes([buffer[0], buffer[1]]),
            flags: u16::from_be_bytes([buffer[2], buffer[3]]),
            num_questions: u16::from_be_bytes([buffer[4], buffer[5]]),
            num_answers: u16::from_be_bytes([buffer[6], buffer[7]]),
            num_authorities: u16::from_be_bytes([buffer[8], buffer[9]]),
            num_additionals: u16::from_be_bytes([buffer[10], buffer[11]]),
        }
    }
}

#[derive(Debug, PartialEq)]
struct DNSQuestion {
    name: String,
    type_: u16,
    class: u16,
}

impl DNSQuestion {
    fn to_be_bytes(&self) -> Vec<u8> {
        let mut bytes = encode_dns_name(&self.name);
        bytes.extend(self.type_.to_be_bytes());
        bytes.extend(self.class.to_be_bytes());
        bytes
    }

    fn parse<R: SeekReader>(reader: &mut R) -> Self {
        let name = decode_dns_name(reader);
        let mut buffer = [0; 4];
        reader.read_exact(&mut buffer).unwrap();
        Self {
            name,
            type_: u16::from_be_bytes([buffer[0], buffer[1]]),
            class: u16::from_be_bytes([buffer[2], buffer[3]]),
        }
    }
}

fn encode_dns_name(name: &str) -> Vec<u8> {
    let mut bytes = Vec::<u8>::new();
    for part in name.split('.') {
        bytes.push(part.len() as u8);
        bytes.extend(part.as_bytes());
    }
    bytes.push(0);
    bytes
}

fn decode_dns_name<R: SeekReader>(reader: &mut R) -> String {
    let mut name = String::new();
    loop {
        let mut len_buf = [0];
        reader.read_exact(&mut len_buf).unwrap();
        let len = len_buf[0];
        if len == 0 {
            break;
        }
        if !name.is_empty() {
            name.push('.');
        }
        if len & 0b1100_0000 != 0 {
            name.push_str(&decode_compressed_dns_name(len, reader));
            break;
        }
        let mut part = vec![0; len as usize];
        reader.read_exact(&mut part).unwrap();
        name.push_str(&String::from_utf8(part).unwrap());
    }
    name
}

fn decode_compressed_dns_name<R: SeekReader>(len_first_byte: u8, reader: &mut R) -> String {
    let pointer_first_byte = len_first_byte & 0b0011_1111;
    let mut buffer = [0];
    reader.read_exact(&mut buffer).unwrap();
    let pointer_second_byte = buffer[0];
    let pointer = u16::from_be_bytes([pointer_first_byte, pointer_second_byte]);

    let current_pos = reader.stream_position().unwrap();
    reader
        .seek(std::io::SeekFrom::Start(pointer as u64))
        .unwrap();

    let result = decode_dns_name(reader);

    reader.seek(std::io::SeekFrom::Start(current_pos)).unwrap();
    result
}

#[derive(Debug, PartialEq)]
struct DNSRecord {
    name: String,
    type_: u16,
    class: u16,
    ttl: u32,
    data: Vec<u8>,
}

impl DNSRecord {
    fn parse<R: SeekReader>(reader: &mut R) -> Self {
        let name = decode_dns_name(reader);
        let mut buffer = [0; 10];
        reader.read_exact(&mut buffer).unwrap();
        let type_ = u16::from_be_bytes([buffer[0], buffer[1]]);
        let class = u16::from_be_bytes([buffer[2], buffer[3]]);
        let ttl = u32::from_be_bytes([buffer[4], buffer[5], buffer[6], buffer[7]]);
        let data_len = u16::from_be_bytes([buffer[8], buffer[9]]);
        let mut data = vec![0; data_len as usize];
        reader.read_exact(&mut data).unwrap();
        Self {
            name,
            type_,
            class,
            ttl,
            data,
        }
    }

    fn parse_ip_address(&self) -> std::net::Ipv4Addr {
        if self.type_ != 1 {
            panic!("not an A record");
        }
        if self.data.len() != 4 {
            panic!("invalid IP address length");
        }
        let ip_bytes = [self.data[0], self.data[1], self.data[2], self.data[3]];
        std::net::Ipv4Addr::from(ip_bytes)
    }

    fn parse_domain_name(&self) -> String {
        if self.type_ != 2 {
            panic!("not an NS record");
        }
        decode_dns_name(&mut std::io::Cursor::new(&self.data))
    }
}

#[derive(Debug)]
struct DNSPacket {
    _header: DNSHeader,
    _questions: Vec<DNSQuestion>,
    answers: Vec<DNSRecord>,
    authorities: Vec<DNSRecord>,
    additionals: Vec<DNSRecord>,
}

impl DNSPacket {
    fn parse<R: SeekReader>(reader: &mut R) -> Self {
        let header = DNSHeader::parse(reader);
        let mut questions = Vec::new();
        for _ in 0..header.num_questions {
            questions.push(DNSQuestion::parse(reader));
        }
        let mut answers = Vec::new();
        for _ in 0..header.num_answers {
            answers.push(DNSRecord::parse(reader));
        }
        let mut authorities = Vec::new();
        for _ in 0..header.num_authorities {
            authorities.push(DNSRecord::parse(reader));
        }
        let mut additionals = Vec::new();
        for _ in 0..header.num_additionals {
            additionals.push(DNSRecord::parse(reader));
        }
        DNSPacket {
            _header: header,
            _questions: questions,
            answers,
            authorities,
            additionals,
        }
    }

    fn get_ip(&self) -> Option<std::net::Ipv4Addr> {
        for answer in &self.answers {
            if answer.type_ == 1 {
                return Some(answer.parse_ip_address());
            }
        }
        None
    }

    fn get_nameserver_ip(&self) -> Option<std::net::Ipv4Addr> {
        for answer in &self.additionals {
            if answer.type_ == 1 {
                return Some(answer.parse_ip_address());
            }
        }
        None
    }

    fn get_nameserver(&self) -> Option<String> {
        for answer in &self.authorities {
            if answer.type_ == 2 {
                return Some(answer.parse_domain_name());
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Seek;

    #[test]
    fn test_build_query() {
        let bytes = build_query("www.example.com".to_string(), 1);
        let expected = b"\x00\x00\x00\x01\x00\x00\x00\x00\x00\x00\x03www\x07example\x03com\x00\x00\x01\x00\x01";
        let bytes_without_random_id = &bytes[2..];
        assert_eq!(bytes_without_random_id, expected);
    }

    #[test]
    fn test_dns_header_to_be_bytes() {
        let header = DNSHeader {
            id: 0x1234,
            flags: 0x5678,
            num_questions: 0x9abc,
            num_answers: 0xdef0,
            num_authorities: 0x1234,
            num_additionals: 0x5678,
        };
        let expected = vec![
            0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0, 0x12, 0x34, 0x56, 0x78,
        ];
        assert_eq!(header.to_be_bytes(), expected);
    }

    #[test]
    fn test_dns_reader_parse() {
        let bytes = b"\x12\x34\x56\x78\x9a\xbc\xde\xf0\x12\x34\x56\x78";
        let mut reader = std::io::Cursor::new(bytes);
        let header = DNSHeader::parse(&mut reader);
        let expected = DNSHeader {
            id: 0x1234,
            flags: 0x5678,
            num_questions: 0x9abc,
            num_answers: 0xdef0,
            num_authorities: 0x1234,
            num_additionals: 0x5678,
        };
        assert_eq!(header, expected);
    }

    #[test]
    fn test_dns_question_to_be_bytes() {
        let question = DNSQuestion {
            name: "www.example.com".to_string(),
            type_: 0x1234,
            class: 0x5678,
        };
        let expected = b"\x03www\x07example\x03com\x00\x12\x34\x56\x78";
        assert_eq!(question.to_be_bytes(), expected);
    }

    #[test]
    fn test_parse_dns_question() {
        let bytes = b"\x03www\x07example\x03com\x00\x12\x34\x56\x78";
        let mut reader = std::io::Cursor::new(bytes);
        let question = DNSQuestion::parse(&mut reader);
        let expected = DNSQuestion {
            name: "www.example.com".to_string(),
            type_: 0x1234,
            class: 0x5678,
        };
        assert_eq!(question, expected);
    }

    #[test]
    fn test_encode_dns_name() {
        let name = "www.example.com";
        let bytes = encode_dns_name(name);
        let expected = b"\x03www\x07example\x03com\x00";
        assert_eq!(bytes, expected);
    }

    #[test]
    fn test_decode_dns_name() {
        let bytes = b"\x03www\x07example\x03com\x00";
        let mut reader = std::io::Cursor::new(bytes);
        let name = decode_dns_name(&mut reader);
        let expected = "www.example.com";
        assert_eq!(name, expected);
    }

    #[test]
    fn test_decode_compressed_dns_name() {
        let bytes = b"\x00\x03www\x07example\x03com\x00\xc0\x01";
        let mut reader = std::io::Cursor::new(bytes);
        reader.seek(std::io::SeekFrom::Start(18)).unwrap();
        let name = decode_dns_name(&mut reader);
        let expected = "www.example.com";
        assert_eq!(name, expected);
    }

    #[test]
    fn test_dns_record() {
        let bytes =
            b"\x03www\x07example\x03com\x00\x12\x34\x56\x78\x9a\xbc\xde\xf0\x00\x02\x01\x02";
        let mut reader = std::io::Cursor::new(bytes);
        let record = DNSRecord::parse(&mut reader);
        let expected = DNSRecord {
            name: "www.example.com".to_string(),
            type_: 0x1234,
            class: 0x5678,
            ttl: 0x9abcdef0,
            data: vec![1, 2],
        };
        assert_eq!(record, expected);
    }
}
