use bytes::{buf, Buf, BytesMut};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt, BufWriter},
    net::TcpStream,
};

use crate::{
    error::{Error, Result},
    resp::RESP,
};

pub struct Connection {
    // ! Add BufWriter
    stream: BufWriter<TcpStream>,
    buffer: BytesMut,
}

impl Connection {
    pub fn new(stream: TcpStream) -> Connection {
        Connection {
            stream: BufWriter::new(stream),
            buffer: BytesMut::with_capacity(4096),
        }
    }

    pub async fn read_frame(&mut self) -> Result<Option<RESP>> {
        loop {
            if let Some(frame) = self.parse_frame()? {
                return Ok(Some(frame));
            }

            let message_length = self
                .stream
                .read_buf(&mut self.buffer)
                .await
                .map_err(Error::Io)?;
            if 0 == message_length {
                // If `0` then the peer has closed the connection
                if self.buffer.is_empty() {
                    return Ok(None);
                } else {
                    return Err(Error::ConnectionClosed);
                }
            }
        }
    }
    fn parse_frame(&mut self) -> Result<Option<RESP>> {
        match RESP::parse(&self.buffer) {
            Ok((resp, offset)) => {
                self.buffer.advance(offset);
                return Ok(Some(resp));
            }
                ,
            // If there's not enough data to parse a frame return None.
            Err(Error::IncompleteRequestData) => return Ok(None),
            Err(e) => return Err(e),
        }
    }

    pub async fn write_frame(&mut self, frame: &RESP) -> Result<()> {
        match frame {
            RESP::Array(elements) => {
                self.stream.write_u8(b'*').await.map_err(Error::Io)?;
                self.write_decimal(elements.len() as i64)
                    .await
                    .map_err(Error::Io)?;
                self.write_crlf().await.map_err(Error::Io)?;

                for element in elements {
                    self.write_value(element).await.map_err(Error::Io)?;
                }
            }
            _ => self.write_value(frame).await.map_err(Error::Io)?,
        }
        self.stream.flush().await.map_err(Error::Io)?;

        Ok(())
    }

    async fn write_value(&mut self, frame: &RESP) -> std::io::Result<()> {
        match frame {
            RESP::Simple(body) => {
                self.stream.write_u8(b'+').await?;
                self.stream.write_all(body.as_bytes()).await?;
                self.write_crlf().await?;
            }
            RESP::Error(body) => {
                self.stream.write_u8(b'-').await?;
                self.stream.write_all(body.as_bytes()).await?;
                self.write_crlf().await?;
            }
            RESP::Integer(val) => {
                self.stream.write_u8(b':').await?;
                self.write_decimal(*val).await?;
                self.write_crlf().await?
            }
            RESP::Bulk(body) => {
                self.stream.write_u8(b'$').await?;
                self.write_decimal(body.len() as i64).await?;
                self.write_crlf().await?;
                self.stream.write_all(&body).await?;
                self.write_crlf().await?;
            }
            RESP::Null => self.stream.write_all(b"$-1\r\n").await?,
            RESP::Array(_) => unreachable!(),
        }

        Ok(())
    }

    async fn write_decimal(&mut self, val: i64) -> std::io::Result<()> {
        let formatted = format!("{val}").as_bytes().to_owned();
        self.stream.write_all(&formatted).await?;

        Ok(())
    }

    async fn write_crlf(&mut self) -> std::io::Result<()> {
        self.stream.write_all(&RESP::CRLF).await?;
        Ok(())
    }
}
