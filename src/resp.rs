use bytes::Bytes;
// use tokio::io::AsyncWriteExt;

use crate::error::{Error, Result};
// use std::io::Write;
// use bytes::Buf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RESP {
    Simple(String),
    Integer(i64),
    Error(String),
    Null,
    Bulk(Bytes),
    Array(Vec<RESP>),
}

impl RESP {
    pub const CRLF: [u8; 2] = *b"\r\n";
    pub fn parse(src: &[u8]) -> Result<(RESP, usize)> {
        if src.is_empty() {
            return Err(Error::IncompleteRequestData);
        }
        // println!("SRC: {:?}", bytes_to_string(src));
        match src[0] {
            b'+' => {
                let crlf_start_index = find_crlf(&src).ok_or(Error::IncompleteRequestData)?;
                let body: String = bytes_to_string(&src[1..crlf_start_index]);
                let resp = RESP::Simple(body);

                Ok((resp, crlf_start_index + 2))
            }
            b':' => {
                let crlf_start_index = find_crlf(&src).ok_or(Error::IncompleteRequestData)?;
                let int: i64 = bytes_to_string(&src[1..crlf_start_index])
                    .parse::<i64>()
                    .map_err(|_| Error::Msg("Invalid integer literal".to_string()))?;
                let resp = RESP::Integer(int);

                Ok((resp, crlf_start_index + 2))
            }
            b'-' => {
                let crlf_start_index = find_crlf(&src).ok_or(Error::IncompleteRequestData)?;
                let body: String = bytes_to_string(&src[1..crlf_start_index]);
                let resp = RESP::Error(body);

                return Ok((resp, crlf_start_index + 2));
            }
            b'$' => {
                let length_bytes_crlf = find_crlf(&src).ok_or(Error::IncompleteRequestData)?;

                let length: i64 = bytes_to_string(&src[1..length_bytes_crlf])
                    .parse()
                    .map_err(|_| Error::Msg("Invalid bulk string length.".to_string()))?;

                if length == -1 && src.len() < 6 && src[length_bytes_crlf + 1] != b'\r' {
                    return Ok((RESP::Null, 5));
                }

                let body_start = length_bytes_crlf + 2;
                let body_end_crlf = body_start
                    + find_crlf(&src[body_start..]).ok_or(Error::IncompleteRequestData)?;
                let string_body: Bytes = Bytes::copy_from_slice(&src[body_start..body_end_crlf]);

                if length != -1 && string_body.len() != length as usize {
                    return Err(Error::Msg(
                        "Bulk string length doesn't match body length".to_string(),
                    ));
                }
                return Ok((RESP::Bulk(string_body), body_end_crlf + 2));
            }
            b'*' => {
                let length_bytes_crlf = find_crlf(&src).ok_or(Error::IncompleteRequestData)?;

                let length: i64 = bytes_to_string(&src[1..length_bytes_crlf])
                    .parse()
                    .map_err(|_| Error::Msg("Invalid bulk string length.".to_string()))?;

                if length == -1 && src.len() < 6 && src[length_bytes_crlf + 1] != b'\r' {
                    return Ok((RESP::Null, 5));
                }
                let length = length as usize;

                let mut elements: Vec<RESP> = Vec::new();
                let mut cursor = length_bytes_crlf + 2;
                for _ in 0..length {
                    let (element, offset) = RESP::parse(&src[cursor..])?;
                    elements.push(element);
                    cursor += offset;
                }

                return Ok((RESP::Array(elements), cursor));
            }
            _ => {
                return Err(Error::InvalidRequestData);
            }
        }
    }

    pub fn is_string(&self) -> bool {
        use RESP::*;
        match self {
            Simple(_) | Bulk(_) => true,
            _ => false,
        }
    }
}

fn find_crlf(bytes: &[u8]) -> Option<usize> {
    for i in 0..bytes.len() - 1 {
        if bytes[i] == b'\r' && bytes[i + 1] == b'\n' {
            return Some(i);
        }
    }
    None
}

fn bytes_to_string(bytes: &[u8]) -> String {
    bytes.into_iter().map(|byte| *byte as char).collect()
}

#[cfg(test)]
mod tests {
    use super::RESP;
    #[test]
    fn null_strings_works() {
        let src = b"$-1\r\n";
        let res = RESP::parse(src);
        assert_eq!((RESP::Null, 5), res.unwrap())
    }

    #[test]
    fn empty_strings_works() {
        let src = b"$-1\r\n\r\n";
        let res = RESP::parse(src);
        assert_eq!((RESP::Bulk("".into()), 7), res.unwrap())
    }

    #[test]
    fn null_arrays_works() {
        let src = b"*-1\r\n";
        let res = RESP::parse(src);
        assert_eq!((RESP::Null, 5), res.unwrap())
    }

    #[test]
    fn empty_arrays_works() {
        let src = b"*-1\r\n\r\n";
        let res = RESP::parse(src);
        println!("{res:?}");
        assert_eq!((RESP::Array(Vec::new()), 7), res.unwrap());
    }

    #[test]
    fn can_parse() {
        let src =
            b"*5\r\n+simple string\r\n-simple error\r\n:-121\r\n:121\r\n$11\r\nbulk string\r\n";
        let parse_res = RESP::parse(src);
        let expected = RESP::Array(vec![
            RESP::Simple("simple string".to_string()),
            RESP::Error("simple error".to_string()),
            RESP::Integer(-121),
            RESP::Integer(121),
            RESP::Bulk("bulk string".into()),
        ]);
        assert_eq!((expected, 66), parse_res.unwrap());
    }
}
