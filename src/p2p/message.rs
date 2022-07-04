extern crate semver;
extern crate serde;

use anyhow::anyhow;
use std::collections::HashSet;
use std::net::SocketAddr;
use std::string::String;

use transaction::pool::Transaction;

use self::semver::Version;
use self::serde::{Deserialize, Serialize};

const PROTOCOL_NAME: &str = "mincoin_protocol";
const PROTOCOL_VERSION: &str = "0.1.0";

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub enum MsgType {
    Add,
    Remove,
    CoreList,
    RequestCoreList,
    Ping,
    AddAsEdge,
    RemoveEdge,
    NewTransaction,
    NewBlock,
    RequestFullChain,
    RspFullChain,
    Enhanced,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Message {
    pub protocol: String,
    pub version: String,
    pub msg_type: MsgType,
    pub my_addr: SocketAddr,
    pub new_core_set: Option<HashSet<SocketAddr>>,
    pub new_transaction: Option<Transaction>,
}

impl Message {
    pub fn new(
        msg_type: MsgType,
        my_addr: SocketAddr,
        new_core_set: Option<HashSet<SocketAddr>>,
        new_transaction: Option<Transaction>,
    ) -> Message {
        Message {
            protocol: PROTOCOL_NAME.to_string(),
            version: PROTOCOL_VERSION.to_string(),
            msg_type,
            my_addr,
            new_core_set,
            new_transaction,
        }
    }
}

pub fn build(
    msg_type: MsgType,
    my_addr: SocketAddr,
    new_core_set: Option<HashSet<SocketAddr>>,
    new_transaction: Option<Transaction>,
) -> String {
    let msg = Message::new(msg_type, my_addr, new_core_set, new_transaction);
    serde_json::to_string(&msg).unwrap()
}

pub fn parse(msg_str: &str) -> anyhow::Result<Message> {
    let msg: Message = serde_json::from_str(msg_str).unwrap();

    if msg.protocol != PROTOCOL_NAME {
        Err(anyhow!("Protocol name is not matched"))
    } else if Version::parse(&msg.version)? > Version::parse(PROTOCOL_VERSION)? {
        Err(anyhow!("Protocol version is not matched"))
    } else {
        Ok(msg)
    }
}
