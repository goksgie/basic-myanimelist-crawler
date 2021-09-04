use std::fmt;


pub enum ErrorType {
    /// An out of bounds error may occur if a given position is
    /// greater than 512 or less than 0.
    OutOfBounds,

    /// End of file error may occur if the user tries to read
    /// from the buffer after it reached at the end of it.
    EndOfFile,

    /// While reading query name, it is possible to observe
    /// infinite loops. To avoid that, we set a maximum number
    /// of jumps.
    MaxJumpsReached,
}

impl fmt::Display for ErrorType {
    fn fmt(&self, f_out: &mut fmt::Formatter) -> fmt::Result {
        write!(f_out, "An error with type: {} occured", self)
    }
}

impl fmt::Debug for ErrorType {
    fn fmt(&self, f_out: &mut fmt::Formatter) -> fmt::Result {
        write!(f_out, "Error {} occured in file: {}, line: {}", self, file!(), line!())
    }
}

const BUFFER_SIZE: usize = 512;


#[derive(Debug)]
pub struct ByteBuffer {
    /// A typical DNS query's length is 512 bytes.
    buf: [u8; BUFFER_SIZE],

    /// within a buffer, we need to keep track of our current index.
    index: usize,
}


impl ByteBuffer {
    /// Constructs a new buffer. In the future, it might make sense to
    /// accept a an already existing buffer or some type that supports
    /// into trait as well. But for now, we opt against doing so for
    /// the sake of development speed.
    pub fn new() -> Self {
        ByteBuffer { buf: [0; BUFFER_SIZE], index: 0 } 
    }

    pub fn set_buffer(&mut self, buf: &Vec<u8>) {
        for index in 0..buf.len() {
            self.buf[index] = buf[index];
        }
        self.index = 0;
    }

    /// Returns the current index (where we are pointing at on the buffer)
    pub fn get_index(&self) -> usize {
        self.index
    }

    /// Tries to increase the index by steps. If the boundary (512) is exceeded,
    /// returns OutOfBounds error. Otherwise, Ok().
    pub fn step(&mut self, steps: usize) -> Result<(), ErrorType> {
        if self.index + steps >= BUFFER_SIZE {
            return Err(ErrorType::OutOfBounds);
        }
        self.index += steps;
        Ok(())
    }

    /// Tries to set the current index to the position value. If the position
    /// is greater than 512, returns an OutOfBounds error. Otherwise, Ok().
    pub fn seek(&mut self, pos: usize) -> Result<(), ErrorType> {
        if pos >= BUFFER_SIZE {
            return Err(ErrorType::OutOfBounds);
        }
        self.index = pos;
        Ok(())
    }

    /// Tries to read the buffer by one and increases the current index.
    /// May throw EndOfBuffer error if the current index is at 512.
    /// Otherwise, returns the byte.
    pub fn read_mut(&mut self) -> Result<u8, ErrorType> {
        if self.index == BUFFER_SIZE {
            return Err(ErrorType::EndOfFile);
        }
        let byte = self.buf[self.index];
        self.index += 1;
        Ok(byte)
    }

    /// Tries to read two bytes from the buffer. May throw EndOfBuffer error.
    pub fn read_mut_u16(&mut self) -> Result<u16, ErrorType> {
        let word: u16 = ((self.read_mut()? as u16) << 8) | self.read_mut()? as u16; 
        Ok(word)
    }
    
    /// Tries to read four bytes from the buffer. May throw EndOfBuffer error.
    pub fn read_mut_u32(&mut self) -> Result<u32, ErrorType> {
        let dword: u32 = ((self.read_mut_u16()? as u32) << 16) | self.read_mut_u16()? as u32;
        Ok(dword)
    }

    /// Tries to read the buffer by one. At the end of the operation, the
    /// current index does not change. 
    /// May throw EndOfBuffer error if the current index is at 512.
    /// Otherwise, returns the requested byte.
    pub fn read(&self) -> Result<u8, ErrorType> {
        if self.index == BUFFER_SIZE {
            Err(ErrorType::EndOfFile)
        } else {
            Ok(self.buf[self.index])
        }
    }

    /// Tries to read the value at the position. May throuw OutOfBounds error.
    pub fn get_at(&self, index: usize) -> Result<u8, ErrorType> {
        if index >= BUFFER_SIZE {
            Err(ErrorType::OutOfBounds)
        } else {
            Ok(self.buf[index])
        }
    }

    /// Tries to read a slice [p_start, p_start + len) from the buffer. May
    /// throw OutOfBounds error.
    pub fn get_slice(&self, p_start: usize, len: usize) -> Result<&[u8], ErrorType> {
        if p_start + len >= BUFFER_SIZE {
            Err(ErrorType::OutOfBounds)
        } else {
            Ok(&self.buf[p_start..p_start + len])
        }
    }

    /// Reads the domain name presented in the query. Since DNS is designed
    /// to contain jumps in order to recude footprint, it is possible to have
    /// never ending loops. Hence, if the number of performed jumps exceed
    /// the maximum allowed jumps, this function will generate an Error.
    pub fn read_qname(&mut self, outstr: &mut String) -> Result<(), ErrorType> {
        // the domain name contains the following syntax:
        // [len:6]google[len:3]com
        // If two of the most significant bits are set in the length value,
        // then there comes an additional byte representing the jump position.

        let mut index = self.index;

        let max_jumps = 5;
        let mut curr_jumps = 0;
        let mut jumped = false;

        // whenever we read number of chars equal to len field,
        // we will insert out delimiter.
        let mut delim = "";

        loop {
            if curr_jumps >= max_jumps {
                return Err(ErrorType::MaxJumpsReached);
            }

            let len = self.get_at(index)?;

            // if the most two significant bits are set,
            // the next byte will be the jump position.
            if (len & 0xC0) == 0xC0 {
                // this comparision is safe because hex codes of ascii characters
                // do nat start with C.
                let jump_byte = self.get_at(index + 1)? as u16;
                let jump_offset = (((len as u16) ^ 0xC0) << 8) | jump_byte; 
                index = jump_offset as usize;
                jumped = true;
                curr_jumps += 1;
                continue;
            } else if len == 0 {
                break;
            }

            index += 1;

            outstr.push_str(&delim);
            // now we have our length and we can initiate an inner loop,
            // to read our characters.
            outstr.push_str(&String::from_utf8_lossy(self.get_slice(index, len as usize)?).to_lowercase()); 
            index += len as usize;
            delim = ".";
        }

        if !jumped {
            self.seek(index + 1)?;
        } else {
            // if we haven't jumped, we need to increment our current index by two,
            // as we read one u16 
            self.seek(self.index + 2)?;
        }

        Ok(())
    }

}

#[test]
fn test_qname() {
    let vec_test_queries = vec![
        (vec![
            0x06, 0x67, 0x6f, 0x6f, 0x67, 0x6c, 0x65,
            0x03, 0x63, 0x6f, 0x6d, 0x00
        ], "google.com"),
        (vec![
            0x0b, 0x6d, 0x79, 0x61, 0x6e, 0x69, 0x6d, 
            0x65, 0x6c, 0x69, 0x73, 0x74, 0x03, 0x6e, 
            0x65, 0x74, 0x00  
        ], "myanimelist.net"),
        // the following tests jumping/looping representation
        // of domain name.
        (vec![
            0x0b, 0x6d, 0x79, 0x61, 0x6e, 0x69, 0x6d, 
            0x65, 0x6c, 0x69, 0x73, 0x74, 0x03, 0x6e, 
            0x65, 0x74, 0x00, 0xc0, 0x00  
        ], "myanimelist.net"),
    ];

    let mut byte_buffer = ByteBuffer::new();
    for (query_vec, query_out) in vec_test_queries.iter() {
        let mut out_str = String::new();
        byte_buffer.set_buffer(query_vec);
        let res = byte_buffer.read_qname(&mut out_str);
        assert_eq!(res.is_ok(), true);
        assert_eq!(query_out, &out_str);
        // we compare with length + 2 because we have to perfrom a
        // dummy read at the end of read_qname since after domain
        // name comes 0x00 to indicate termination.
        assert_eq!(byte_buffer.get_index(), query_out.len() + 2);
    }

    // the last test case is special. If we continue to read, we should
    // obtain the same domain name again. 
    let mut out_str = String::new();
    let res = byte_buffer.read_qname(&mut out_str);
    assert_eq!(res.is_ok(), true);
    assert_eq!(vec_test_queries.last().unwrap().1, out_str);
    assert_eq!(byte_buffer.get_index(), out_str.len() + 4);
}

#[test]
fn test_byte_reads() {
    let vec_test_queries = vec![
        (
            vec![
                0x00, 0x00, 0x01, 0x2b
            ],
            0x0000012b as u32
        )
    ];

    let mut byte_buffer = ByteBuffer::new();
    for (query_vec, query_out) in vec_test_queries.iter() {
        byte_buffer.set_buffer(query_vec);
        assert_eq!(byte_buffer.read_mut_u32().unwrap(), *query_out);
    }
}