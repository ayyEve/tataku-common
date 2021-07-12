

pub struct Chat {
    messages: Vec<ChatMessage>
}

pub struct ChatMessage {
    sender: String,
    // channel or username
    channel: String, 
    sender_id: u32,
    timestamp: u64, //TODO: make this not shit
    text: String
}