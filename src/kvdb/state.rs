use std::collections::HashMap;
use std::sync::RwLock;
use std::io::{Error, ErrorKind};
use bincode;

#[derive(Debug, Clone, PartialEq, bincode::Encode, bincode::Decode)]
pub enum Command {
    Set { key: String, value: String },
    Get { key: String },
}

impl Command {
    pub fn new_set(k: &str, v: &str) -> Self {
        Self::Set { key: k.to_string(), value: v.to_string() }
    }

    pub fn new_get(k: &str) -> Self {
        Self::Get { key: k.to_string() }
    }

    pub fn encode(&self) -> Result<Vec<u8>, bincode::error::EncodeError> {
        bincode::encode_to_vec(self, bincode::config::standard())
    }

    pub fn decode(bytes: &[u8]) -> Result<Command, bincode::error::DecodeError> {
        let (command, _bytes_read) = bincode::decode_from_slice(bytes, bincode::config::standard())?;
        Ok(command)
    }
}

pub struct StateMachine {
    server: RwLock<HashMap<String, String>>,
    id: u64,
}

impl StateMachine {
    pub fn new(server_id: u64) -> Self {
        StateMachine { server: RwLock::new(HashMap::new()), id: server_id }
    }

    pub fn apply(&mut self, cmd: &[u8]) -> Result<Vec<u8>, Error> {
        let decoded_cmd = Command::decode(cmd)
            .map_err(|e| Error::new(ErrorKind::Other, format!("failed to decode command: {e}")))?;
        match decoded_cmd {
            Command::Set { key, value } => {
                let mut db = self.server.write()
                    .map_err(|e| Error::new(ErrorKind::Other, format!("failed to acquire write lock: {e}")))?;
                db.insert(key, value);
                Ok(Vec::new())
            },
            Command::Get { key } => {
                let db = self.server.read().map_err(|e| Error::new(ErrorKind::Other, format!("failed to acquire read lock: {e}")))?;
                match db.get(&key) {
                    Some(item) => Ok(item.as_bytes().to_vec()),
                    None => Err(Error::new(ErrorKind::NotFound, "key not found")),
                }
            },
        }
    }
}