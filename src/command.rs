use tokio::time::Duration;

use bytes::Bytes;

use crate::{
    db::Db,
    error::{Error, Result},
    resp::RESP,
};

#[derive(Debug)]
pub enum Command {
    Ping {
        msg: Option<Bytes>,
    },
    Echo {
        msg: RESP,
    },
    Set {
        key: String,
        value: Bytes,
        ttl: Option<u64>,
    },
    Get {
        key: String,
    },
}

impl Command {
    pub async fn execute_cmd(self, db: &mut Db) -> RESP {
        use Command::*;
        match self {
            Ping { msg } => {
                if let Some(msg) = msg {
                    RESP::Bulk(msg)
                } else {
                    RESP::Simple("PONG".to_string())
                }
            }
            Echo { msg } => msg,
            Set { key, value, ttl } => {
                if let Some(previous_entry) = db.set(key, value, ttl) {
                    RESP::Bulk(previous_entry)
                } else {
                    RESP::Simple("OK".to_string())
                }
            }
            Get { key } => {
                if let Some(data) = db.get(&key) {
                    RESP::Bulk(data)
                } else {
                    RESP::Null
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
            let arg0 = extract_string(&args[0])?;

            match arg0.to_uppercase().as_str() {
                "PING" => {
                    let msg = if let Some(msg) = args.get(1) {
                        Some(extract_string_as_bytes(msg)?)
                    } else {
                        None
                    };
                    return Ok(Command::Ping { msg });
                }
                "ECHO" => {
                    if let Some(resp) = args.get(1) {
                        if resp.is_string() {
                            return Ok(Command::Echo { msg: resp.clone() });
                        } else {
                            return Err(Error::Msg("Echo command only takes strings".to_string()));
                        }
                    } else {
                        return Err(Error::Msg("Echo command takes 1 argument".to_string()));
                    }
                }
                "SET" => {
                    let key = if let Some(raw_key) = args.get(1) {
                        extract_string(&raw_key)?
                    } else {
                        return Err(Error::Msg("Set command requires a key".to_string()));
                    };

                    let value = if let Some(raw_key) = args.get(2) {
                        extract_string_as_bytes(&raw_key)?
                    } else {
                        return Err(Error::Msg("Set command requires a key".to_string()));
                    };

                    let ttl: Option<u64> = if let Some(raw_ttl) = args.get(3) {
                        let time_unit = extract_string(&raw_ttl)?;

                        let duration: u64 = if let Some(raw_ttl) = args.get(3) {
                            extract_string(&raw_ttl)?.parse::<u64>().map_err(|_| {
                                Error::Msg("Expiration duration must be a number".to_string())
                            })?
                        } else {
                            return Err(Error::Msg("Set command requires a key".to_string()));
                        };

                        match time_unit.as_str() {
                            // Time units in seconds
                            "EX" => Some(duration * 1000),
                            // Time units in milliseconds
                            "PX" => Some(duration),
                            _ => {
                                return Err(Error::Msg(format!(
                                    "Set command option '{time_unit}' is not supported"
                                )))
                            }
                        }
                    } else {
                        None
                    };

                    Ok(Command::Set { key, value, ttl })
                }
                "GET" => {
                    let key = if let Some(raw_key) = args.get(1) {
                        extract_string(&raw_key)?
                    } else {
                        return Err(Error::Msg("Set command requires a key".to_string()));
                    };

                    Ok(Command::Get { key })
                }
                _ => return Err(Error::Msg("Unknown or unsupported command".to_string())),
            }
        } else {
            Err(Error::Msg("Commands should be an array".to_string()))
        }
    }
}

fn extract_string(val: &RESP) -> Result<String> {
    match val {
        RESP::Bulk(body) => Ok(body.into_iter().map(|b| *b as char).collect()),
        RESP::Simple(body) => Ok(body.clone()),
        _ => Err(Error::Msg(
            "Expected command argument to be a string".to_string(),
        )),
    }
}
fn extract_string_as_bytes(val: &RESP) -> Result<Bytes> {
    match val {
        RESP::Bulk(body) => Ok(body.clone()),
        _ => Err(Error::Msg(
            "Expected command argument to be a bulk string".to_string(),
        )),
    }
}
