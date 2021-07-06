

pub struct Chat {
    messages: Vec<ChatMessage>
}



pub struct ChatMessage {
    sender: String,
    sender_id: u32,
    timestamp: u64, //TODO: make this not shit
    text: String
}