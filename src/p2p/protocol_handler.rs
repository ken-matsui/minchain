use p2p::message::Message;

#[derive(Clone)]
pub struct ProtocolHandler;

/// Process or generate an enhanced message of your own extension.
impl ProtocolHandler {
    pub fn new() -> ProtocolHandler {
        println!("Initializing MyProtocolMessageHandler...");
        ProtocolHandler {}
    }

    pub fn handle_message(&self, msg: Message) {
        println!("ProtocolHandler received: {:?}", msg);
    }
}
