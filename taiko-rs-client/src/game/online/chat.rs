#![allow(dead_code, unused, non_snake_case)]
use crate::prelude::*;

const CHANNEL_LIST_WIDTH:f64 = 100.0;
// const CHAT_SIZE:Vector2 = Vector2::new(window_size().x - CHANNEL_LIST_WIDTH, 300.0);
// const CHAT_POS:Vector2 = Vector2::new(0.0, window_size().y - CHAT_SIZE.y);
// const CHANNEL_LIST_SIZE:Vector2 = Vector2::new(CHANNEL_LIST_WIDTH, CHAT_SIZE.y);

pub struct Chat {
    messages: HashMap<ChatChannel, Vec<ChatMessage>>,
    pub visible: bool,

    channel_scroll: ScrollableArea,
    messages_scroll: ScrollableArea,
}
impl Chat {
    pub fn new() -> Self {
        let window_size = Settings::window_size();

        let CHAT_SIZE:Vector2 = Vector2::new(window_size.x - CHANNEL_LIST_WIDTH, 300.0);
        let CHAT_POS:Vector2 = Vector2::new(0.0, window_size.y - CHAT_SIZE.y);
        let CHANNEL_LIST_SIZE:Vector2 = Vector2::new(CHANNEL_LIST_WIDTH, CHAT_SIZE.y);
                
        Self {
            messages: HashMap::new(),
            visible: false,

            // [channels][messages]

            channel_scroll: ScrollableArea::new(CHAT_POS, CHANNEL_LIST_SIZE, true),
            messages_scroll: ScrollableArea::new(CHAT_SIZE + Vector2::new(CHANNEL_LIST_WIDTH, 0.0), CHAT_SIZE, true)
        }
    }

    pub fn add_message(&mut self, m:ChatMessage) {
        if !self.messages.contains_key(&m.channel) {
            self.messages.insert(m.channel.clone(), Vec::new());
        }

        self.messages.get_mut(&m.channel).unwrap().push(m);
    }
}

impl Dialog for Chat {}
impl Menu<Game> for Chat {
    fn draw(&mut self, args:piston::RenderArgs) -> Vec<Box<dyn Renderable>> {
        if !self.visible {return Vec::new()}

        let mut list: Vec<Box<dyn Renderable>> = Vec::new();
        list.extend(self.channel_scroll.draw(args, Vector2::zero(), 0.0));
        list.extend(self.messages_scroll.draw(args, Vector2::zero(), 0.0));
        list
    }
}




pub struct ChatMessage {
    sender: String,
    // channel or username
    channel: ChatChannel, 
    sender_id: u32,
    timestamp: u64, //TODO: make this not shit
    text: String
}


// some kind of identifier
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum ChatChannel {
    Channel{name:String},
    User{user_id:u32}
}