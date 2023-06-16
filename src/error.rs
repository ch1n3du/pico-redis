#[derive(Debug)]
pub enum Error {
    IncompleteRequestData,
    InvalidRequestData,
    ConnectionClosed,
    Msg(String), // If possible make static
    Io(std::io::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Error::*;
        match self {
            IncompleteRequestData => write!(f, "Parse Error: Incomplete request data"),
            InvalidRequestData => write!(f, "Parse Error: Invalid request data"),
            ConnectionClosed => write!(f, "Network Error: Peer closed connection"),
            Msg(err) => write!(f, "General Error: {err}"),
            Io(err) => write!(f, "IO Error: {err}"),
        }
    }
}
