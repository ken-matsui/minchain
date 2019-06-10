extern crate serde;
extern crate semver;

use std::string::String;
use std::net::SocketAddr;
use self::serde::{Serialize, Deserialize};
use self::semver::Version;

static PROTOCOL_NAME: &'static str = "mincoin_protocol";
static PROTOCOL_VERSION: &'static str = "0.1.0";

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub enum MsgType {
    Add,
    Remove,
    CoreList,
    RequestCoreList,
    Ping,
    AddAsEdge,
    RemoveEdge,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Message {
    pub protocol: String,
    pub version: String,
    pub msg_type: MsgType,
    pub my_port: u16,

    #[serde(default)]
    pub payload: Option<Vec<SocketAddr>>,
}

#[derive(Clone)]
pub struct MessageManager;

impl MessageManager {
    pub fn new() -> MessageManager {
        println!("Initializing MessageManager ...");
        MessageManager{}
    }

    pub fn build(&self, msg_type: MsgType, my_port: u16, payload: Option<Vec<SocketAddr>>) -> String {
        let msg = Message {
            protocol: PROTOCOL_NAME.to_string(),
            version: PROTOCOL_VERSION.to_string(),
            msg_type,
            my_port,
            payload,
        };
        serde_json::to_string(&msg).unwrap()
    }

    pub fn parse(&self, msg_str: &String) -> Result<(MsgType, Option<Vec<SocketAddr>>), &'static str> {
        let msg: Message = serde_json::from_str(&msg_str).unwrap();

        if msg.protocol != PROTOCOL_NAME.to_string() {
            Err("Error: Protocol name is not matched")
        } else if Version::parse(&msg.version) > Version::parse(PROTOCOL_VERSION) {
            Err("Error: Protocol version is not matched")
        } else if msg.msg_type == MsgType::CoreList {
            Ok((msg.msg_type, msg.payload))
        } else {
            Ok((msg.msg_type, msg.payload))
        }
    }
}

impl Drop for MessageManager {
    /// Close socket.
    fn drop(&mut self) -> () { // connection_close
        println!("Shutdown MessageManager ...");
    }
}
