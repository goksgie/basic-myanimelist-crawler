use std::net::Ipv4Addr;
use std::convert::{Into, From};
use super::buffer::{ByteBuffer, ErrorType};

#[derive(Debug, PartialEq, Clone, Eq)]
pub enum HeaderType {
    Query = 0,
    Response = 1,
    Unimplemented = 2,
}

impl From<u8> for HeaderType {
    fn from(code: u8) -> Self {
        match code {
            0 => Self::Query,
            1 => Self::Response,
            _ => Self::Unimplemented,
        }
    }
}

#[derive(Debug, PartialEq, Clone, Eq)]
pub enum OperationCode {
    StandardQuery = 0,
    InverseQuery = 1,
    ServerStatusRequest = 2,
    Reserved
}

impl From<u8> for OperationCode {
    fn from(code: u8) -> Self {
        match code {
            0 => Self::StandardQuery,
            1 => Self::InverseQuery,
            3 => Self::ServerStatusRequest,
            _ => Self::Reserved,
        }
    }
}

#[derive(Debug, PartialEq, Clone, Eq)]
pub enum ResponseCode {
    Success = 0,

    /// Format error
    FormatError = 1,

    /// Server failure,
    ServerFailure = 2,

    /// Means that the domain name being
    /// referenced does not exists.
    NameError = 3,

    NotImplemented = 4,

    /// Due to policy reasons, the server
    /// refures to perform any operation
    /// for the query.
    Refused = 5
}

impl From<u8> for ResponseCode {
    fn from(code: u8) -> Self {
        match code {
            0 => Self::Success,
            1 => Self::FormatError,
            2 => Self::ServerFailure,
            3 => Self::NameError,
            4 => Self::NotImplemented,
            5 => Self:: Refused,
            _ => Self::NotImplemented,
        }
    }
}


#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DnsHeader {
    /// The header itself should be 12 bytes in total.
    pub id: u16,
    pub qr: HeaderType,
    pub opcode: OperationCode,

    /// Authoritative answer:
    pub is_auth_answer: bool,

    /// If the message length exceeds 512, this flag
    /// is set to true. It is a hint to use TCP for
    /// request. 
    pub is_truncated: bool,

    /// Set by sender. It is an indicator for server
    /// to search the answer recursively if it is not
    /// known to it.
    pub should_recurse: bool,

    /// Set by the server. Indicates if recursive queries
    /// are allowed or not.
    pub recursion_available: bool,

    /// originally 3 bits and reserved for later use. Currently
    /// used for DNSSEC queries.
    pub z_flag: bool, 

    pub response_code: ResponseCode,

    /// the number of entries in the question section
    pub question_count: u16,

    /// the number of entries in the answer section
    pub answer_count: u16,

    /// the number of entries in the authority section
    pub authority_count: u16,

    /// the number of entries in the additional section
    pub additional_count: u16 

}

impl DnsHeader {
    /// The reason why we might need a default new function for this struct
    /// is that we might want to construct Header ourselves as we build our
    /// query.
    pub fn new() -> Self {
        DnsHeader {
            id: 0,
            qr: HeaderType::Query,
            opcode: OperationCode::StandardQuery,
            is_auth_answer: false,
            is_truncated: false,
            should_recurse: false,
            recursion_available: false,
            z_flag: false,
            response_code: ResponseCode::Success,
            question_count: 0,
            answer_count: 0,
            authority_count: 0,
            additional_count: 0

        }
    }
    /// This function read from the buffer. The exact location of byte ordering
    /// can be found in any DNS related documents.
    pub fn read(buffer: &mut ByteBuffer) -> Result<Self, ErrorType> {        
        let id = buffer.read_mut_u16()?;
        let flags = buffer.read_mut_u16()?;

        let f_left = (flags >> 8) as u8;
        let f_right = (flags & 0x00FF) as u8;

        Ok(DnsHeader {
            id,
            qr: HeaderType::from(f_left >> 7),
            opcode: OperationCode::from((f_left >> 3) & 0x0F),
            is_auth_answer: ((f_left << 1) & 0x0F) & 0x08 == 8,
            is_truncated: ((f_left << 1) & 0x0F) & 0x04 == 4,
            should_recurse: ((f_left << 1) & 0x0F) & 0x02 == 2,

            recursion_available: f_right >> 7 == 1,
            z_flag: ((f_right & 0x70) >> 4) > 0,
            response_code: ResponseCode::from(f_right & 0x0F), 
            
            question_count: buffer.read_mut_u16()?,
            answer_count: buffer.read_mut_u16()?,
            authority_count: buffer.read_mut_u16()?,
            additional_count: buffer.read_mut_u16()?,
        }
        )
    }
}


#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QueryType {
   UNKOWN(u16),
   A, // 1 
}

impl From<u16> for QueryType {
    fn from(code: u16) -> Self {
        match code {
            1 => Self::A,
            _ => Self::UNKOWN(code),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)]
pub enum DnsRecord {
    UNKOWN {
        domain: String,
        qtype: QueryType,
        class: u16,
        data_len: u16,
        ttl: u32,
    },
    A {
        domain: String,
        addr: Ipv4Addr,
        ttl: u32,
    }
}

impl DnsRecord {
    pub fn read(buffer: &mut ByteBuffer) -> Result<Self, ErrorType> {
        let mut domain = String::new();
        buffer.read_qname(&mut domain)?;

        let qtype = QueryType::from(buffer.read_mut_u16()?);
        let class = buffer.read_mut_u16()?;
        let ttl   = buffer.read_mut_u32()?;
        let data_len = buffer.read_mut_u16()?;

        match qtype {
            QueryType::A => {
                let raw_addr = buffer.read_mut_u32()?;
                let addr = Ipv4Addr::new(
                    ((raw_addr & 0xFF000000) >> 24) as u8,
                    ((raw_addr & 0x00FF0000) >> 16) as u8,
                    ((raw_addr & 0x0000FF00) >> 8)  as u8,
                    (raw_addr & 0x000000FF) as u8
                );
                Ok(DnsRecord::A {
                    domain,
                    addr,
                    ttl
                })
            },
            _ => {
                Ok(DnsRecord::UNKOWN {
                    domain,
                    qtype,
                    class,
                    data_len,
                    ttl
                })
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuestionHeader {
    pub name: String,
    pub qtype: QueryType,
    pub class: u16,
}

impl QuestionHeader {
    pub fn new(name: String, qtype: QueryType) -> Self {
        QuestionHeader {
            name,
            qtype,
            class: 1
        }
    }

    /// This function generates a QuestionHeader from ByteBuffer.
    /// Could have been implemented as From trait however, we cannot
    /// take the ownership and consume the ByteBuffer.
    pub fn read(buffer: &mut ByteBuffer) -> Result<QuestionHeader, ErrorType> {
        let mut name = String::new();
        buffer.read_qname(&mut name)?;
        let qtype = QueryType::from(buffer.read_mut_u16()?); 
        let class = buffer.read_mut_u16()?;
        Ok(QuestionHeader {
            name,
            qtype,
            class
        })
    }
}



#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DnsPacket {
    pub header: DnsHeader,
    pub questions: Vec<QuestionHeader>,
    pub answers : Vec<DnsRecord>,
    pub authorities: Vec<DnsRecord>,
    pub resources: Vec<DnsRecord>,
}


impl DnsPacket {
    pub fn new() -> Self {
        DnsPacket {
            header: DnsHeader::new(),
            questions: Vec::new(),
            answers: Vec::new(),
            authorities: Vec::new(),
            resources: Vec::new(),
        }
    }
}

impl From<ByteBuffer> for DnsPacket {
    fn from(mut buffer: ByteBuffer) -> Self {
        let header: DnsHeader = DnsHeader::read(&mut buffer).unwrap();

        let mut questions = Vec::new();
        for _ in 0..header.question_count {
            questions.push(QuestionHeader::read(&mut buffer).unwrap());
        }

        let mut answers = Vec::new();
        for _ in 0..header.answer_count {
            answers.push(DnsRecord::read(&mut buffer).unwrap());
        }

        let mut authorities = Vec::new();
        for _ in 0..header.authority_count {
            authorities.push(DnsRecord::read(&mut buffer).unwrap());
        }

        let mut resources = Vec::new();
        for _ in 0..header.additional_count {
            resources.push(DnsRecord::read(&mut buffer).unwrap());
        }

        DnsPacket {
            header,
            questions,
            answers,
            authorities,
            resources,
        }
    }
}


#[test]
fn test_convertions() {
    let rp = ResponseCode::from(4);
    assert_eq!(rp, ResponseCode::NotImplemented);

    let rp_i: ResponseCode = 4.into();

    assert_eq!(rp_i, rp);
    
}

#[test]
fn test_dns_hdr() {
    let test_queries = vec![
        (
            vec![0xc5, 0x09, 0x01, 0x20, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
            DnsHeader {
                id: 50441,
                qr: HeaderType::Query,
                opcode: OperationCode::StandardQuery,
                is_auth_answer: false,
                is_truncated: false,
                should_recurse: true,
                recursion_available: false,
                z_flag: true,
                response_code: ResponseCode::Success,
                question_count: 1,
                answer_count: 0,
                authority_count: 0,
                additional_count: 0,
            }
        ),
        (
            vec![0xc5, 0x09, 0x81, 0x80, 0x00, 0x01, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00],
            DnsHeader {
                id: 50441,
                qr: HeaderType::Response,
                opcode: OperationCode::StandardQuery,
                is_auth_answer: false,
                is_truncated: false,
                should_recurse: true,
                recursion_available: true,
                z_flag: false,
                response_code: ResponseCode::Success,
                question_count: 1,
                answer_count: 1,
                authority_count: 0,
                additional_count: 0,
            }
        )
    ];

    let mut buffer = ByteBuffer::new();
    for (buff, answ) in test_queries.iter() {
        buffer.set_buffer(buff);
        let ans = DnsHeader::read(&mut buffer);

        assert_eq!(ans.is_ok(), true);
        assert_eq!(ans.unwrap(), *answ);

    }
}


#[test]
fn test_dns_record() {
    let vec_test_queries = vec![
        (
            vec![
                0x06, 0x67, 0x6f, 0x6f, 0x67, 0x6c, 0x65,
                0x03, 0x63, 0x6f, 0x6d, 0x00, 0x00, 0x01,
                0x00, 0x01, 0xc0, 0x00, 0x00, 0x01, 0x00,
                0x01, 0x00, 0x00, 0x01, 0x2b, 0x00, 0x04,
                0x8e, 0xfa, 0xbb, 0x8e
            ], 
            (
                QuestionHeader {
                    name: String::from("google.com"),
                    qtype: QueryType::A,
                    class: 1
                },
                DnsRecord::A {
                domain: String::from("google.com"),
                addr: Ipv4Addr::new(0x8e, 0xfa, 0xbb, 0x8e),
                ttl: 0x12b
                }
            )
        ),
    ];
    let mut byte_buffer = ByteBuffer::new();
    for (query_vec, query_out) in vec_test_queries.iter() {
        byte_buffer.set_buffer(query_vec);
        let q_hdr = QuestionHeader::read(&mut byte_buffer).unwrap();
        assert_eq!(q_hdr, query_out.0);

        let record = DnsRecord::read(&mut byte_buffer).unwrap();
        assert_eq!(record, query_out.1);
    }
}

#[test]
fn test_dns_packet() {
    let byte_vec = vec![
        0xc5, 0x09, 0x81, 0x80, 0x00, 0x01, 0x00, 
        0x01, 0x00, 0x00, 0x00, 0x00, 0x06, 0x67, 
        0x6f, 0x6f, 0x67, 0x6c, 0x65, 0x03, 0x63, 
        0x6f, 0x6d, 0x00, 0x00, 0x01, 0x00, 0x01, 
        0xc0, 0x0c, 0x00, 0x01, 0x00, 0x01, 0x00, 
        0x00, 0x01, 0x2b, 0x00, 0x04, 0x8e, 0xfa, 
        0xbb, 0x8e
    ];
    let mut byte_buffer = ByteBuffer::new();

    byte_buffer.set_buffer(&byte_vec);

    let dns_packet = DnsPacket::from(byte_buffer);
    assert_eq!(dns_packet, DnsPacket {
        header: DnsHeader {
                id: 50441,
                qr: HeaderType::Response,
                opcode: OperationCode::StandardQuery,
                is_auth_answer: false,
                is_truncated: false,
                should_recurse: true,
                recursion_available: true,
                z_flag: false,
                response_code: ResponseCode::Success,
                question_count: 1,
                answer_count: 1,
                authority_count: 0,
                additional_count: 0,
        },
        questions: vec![
            QuestionHeader {
                name: String::from("google.com"),
                qtype: QueryType::A,
                class: 1
            }
        ],
        answers: vec![
            DnsRecord::A {
                domain: String::from("google.com"),
                addr: Ipv4Addr::new(0x8e, 0xfa, 0xbb, 0x8e),
                ttl: 0x12b
            }
        ],
        authorities: Vec::new(),
        resources: Vec::new()
    })
}