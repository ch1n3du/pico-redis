use bytes::Bytes;

use crate::{
    cache::Cache,
    error::{Error, Result},
    resp::RESP,
};

#[derive(Debug)]
pub enum Command {
    Ping { msg: Option<Bytes> },
}

impl Command {
    pub async fn execute_cmd(self, _cache: &mut Cache) -> RESP {
        use Command::*;
        match self {
            Ping { msg } => {
                if let Some(msg) = msg {
                    RESP::Bulk(msg)
                } else {
                    RESP::Simple("PONG".to_string())
                }
            }
        }
    }
}

// impl C {}
impl TryFrom<RESP> for Command {
    type Error = Error;
    fn try_from(value: RESP) -> Result<Self> {
        if let RESP::Array(args) = value {
            if args.is_empty() {
                return Err(Error::Msg("Command array is empty".to_string()));
            }
            let arg0 = extract_simplestring(&args[0])?;

            match arg0.to_uppercase().as_str() {
                "PING" => {
                    let msg = if let Some(msg) = args.get(1) {
                        Some(extract_bulkstring(msg)?)
                    } else {
                        None
                    };
                    return Ok(Command::Ping { msg });
                }
                _ => return Err(Error::Msg("Unknown or unsupported command".to_string())),
            }
        } else {
            Err(Error::Msg("Commands should be an array".to_string()))
        }
    }
}

fn extract_simplestring(val: &RESP) -> Result<String> {
    match val {
        RESP::Bulk(body) => Ok(body.into_iter().map(|b| *b as char).collect()),
        _ => Err(Error::Msg(
            "Expected command argument to be a simple string".to_string(),
        )),
    }
}
fn extract_bulkstring(val: &RESP) -> Result<Bytes> {
    match val {
        RESP::Bulk(body) => Ok(body.clone()),
        _ => Err(Error::Msg(
            "Expected command argument to be a bulk string".to_string(),
        )),
    }
}
