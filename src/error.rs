#[derive(Debug)]
pub enum Error {
    IncompleteRequestData,
    InvalidRequestData,
    ConnectionClosed,
    Msg(String), // If possible make static
    Io(std::io::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
