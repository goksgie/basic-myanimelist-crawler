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
    id: u16,
    qr: HeaderType,
    opcode: OperationCode,

    /// Authoritative answer:
    is_auth_answer: bool,

    /// If the message length exceeds 512, this flag
    /// is set to true. It is a hint to use TCP for
    /// request. 
    is_truncated: bool,

    /// Set by sender. It is an indicator for server
    /// to search the answer recursively if it is not
    /// known to it.
    should_recurse: bool,

    /// Set by the server. Indicates if recursive queries
    /// are allowed or not.
    recursion_available: bool,

    /// originally 3 bits and reserved for later use. Currently
    /// used for DNSSEC queries.
    z_flag: bool, 

    response_code: ResponseCode,

    /// the number of entries in the question section
    question_count: u16,

    /// the number of entries in the answer section
    answer_count: u16,

    /// the number of entries in the authority section
    authority_count: u16,

    /// the number of entries in the additional section
    additional_count: u16 

}

impl DnsHeader {
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
    pub fn read_from(&mut self, buffer: &mut ByteBuffer) -> Result<(), ErrorType> {        
        self.id = buffer.read_mut_u16()?;

        let flags = buffer.read_mut_u16()?;

        let f_left = (flags >> 8) as u8;
        let f_right = (flags & 0x00FF) as u8;
        
        self.qr = HeaderType::from(f_left >> 7);
        self.opcode = OperationCode::from((f_left >> 3) & 0x0F);
        self.is_auth_answer = ((f_left << 1) & 0x0F) & 0x08 == 8;
        self.is_truncated = ((f_left << 1) & 0x0F) & 0x04 == 4;
        self.should_recurse = ((f_left << 1) & 0x0F) & 0x02 == 2;

        self.recursion_available = f_right >> 7 == 1;
        self.z_flag = ((f_right & 0x70) >> 4) > 0;
        self.response_code = ResponseCode::from(f_right & 0x0F); 
        
        self.question_count = buffer.read_mut_u16()?;
        self.answer_count = buffer.read_mut_u16()?;
        self.authority_count = buffer.read_mut_u16()?;
        self.additional_count = buffer.read_mut_u16()?;

        Ok(())
    }
}


pub struct QuestionHeader {

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
    for (buff, answ) in test_queries {
        buffer.set_buffer(buff);
        let mut hdr = DnsHeader::new();
        let ans = hdr.read_from(&mut buffer);

        assert_eq!(hdr, answ);
        assert_eq!(ans.is_ok(), true);

    }
}