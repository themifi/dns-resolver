use rand::prelude::*;

fn main() {
    let query = build_query("www.example.com".to_string(), 1);
    let sock = std::net::UdpSocket::bind("0.0.0.0:12000").unwrap();
    sock.send_to(&query, "8.8.8.8:53").unwrap();
    let mut response = [0; 1024];
    sock.recv(&mut response).unwrap();
    println!("{:?}", response);
}

fn build_query(domain_name: String, record_type: u16) -> Vec<u8> {
    let id = random();
    let recursion_desired = 1 << 8;
    let header = DNSHeader {
        id,
        flags: recursion_desired,
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
}

struct DNSQuestion {
    name: String,
    type_: u16,
    class: u16,
}

impl DNSQuestion {
    fn to_be_bytes(&self) -> Vec<u8> {
        let mut bytes = Self::encode_dns_name(&self.name);
        bytes.extend(self.type_.to_be_bytes());
        bytes.extend(self.class.to_be_bytes());
        bytes
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_query() {
        let bytes = build_query("www.example.com".to_string(), 1);
        let expected = b"\x01\x00\x00\x01\x00\x00\x00\x00\x00\x00\x03www\x07example\x03com\x00\x00\x01\x00\x01";
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
    fn test_dns_question_to_be_bytes() {
        let question = DNSQuestion {
            name: "www.example.com".to_string(),
            type_: 0x1234,
            class: 0x5678,
        };
        let expected = b"\x03www\x07example\x03com\x00\x12\x34\x56\x78";
        assert_eq!(question.to_be_bytes(), expected);
    }
}
