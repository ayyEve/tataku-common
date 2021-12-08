#![allow(dead_code, unused, non_snake_case)]
use crate::prelude::*;

const CHANNEL_LIST_WIDTH:f64 = 100.0;
// const CHAT_SIZE:Vector2 = Vector2::new(window_size().x - CHANNEL_LIST_WIDTH, 300.0);
// const CHAT_POS:Vector2 = Vector2::new(0.0, window_size().y - CHAT_SIZE.y);
// const CHANNEL_LIST_SIZE:Vector2 = Vector2::new(CHANNEL_LIST_WIDTH, CHAT_SIZE.y);

pub struct Chat {
    // messages
    messages: HashMap<ChatChannel, Vec<ChatMessage>>,
    // if the chat is visible or not
    pub visible: bool,

    // scrollables
    channel_scroll: ScrollableArea,
    messages_scroll: ScrollableArea,

    // positions/sizes
    chat_size: Vector2,
    chat_pos: Vector2,
    channel_list_size: Vector2,
}
impl Chat {
    pub fn new() -> Self {
        let window_size = Settings::window_size();

        let chat_size = Vector2::new(window_size.x - CHANNEL_LIST_WIDTH, 300.0);
        let chat_pos = Vector2::new(0.0, window_size.y - chat_size.y);
        let channel_list_size = Vector2::new(CHANNEL_LIST_WIDTH, chat_size.y);
        
        Self {
            // [channels][messages]
            messages: HashMap::new(),
            visible: false,


            channel_scroll: ScrollableArea::new(chat_pos, channel_list_size, true),
            messages_scroll: ScrollableArea::new(chat_size + Vector2::new(CHANNEL_LIST_WIDTH, 0.0), chat_size, true),

            // positions/sizes
            chat_size,
            chat_pos,
            channel_list_size
        }
    }

    pub fn add_message(&mut self, m:ChatMessage) {
        if !self.messages.contains_key(&m.channel) {
            self.messages.insert(m.channel.clone(), Vec::new());
        }

        self.messages.get_mut(&m.channel).unwrap().push(m);
    }
}

impl Dialog<Game> for Chat {
    fn get_bounds(&self) -> Rectangle {
        if !self.visible { // not showing
            Rectangle::bounds_only(Vector2::one() * -1.0, Vector2::zero())
        // } else if self.expanded { // showing users as well
        //     Rectangle::bounds_only(Vector2::zero(), Vector2::zero())
        } else {
            Rectangle::bounds_only(self.chat_pos, Vector2::new(
                // probably faster than getting the window size.x
                self.chat_size.x + self.channel_list_size.x,
                self.chat_size.y
            ))
        }
    }

    fn draw(&mut self, args:&piston::RenderArgs, depth: &f64, list: &mut Vec<Box<dyn Renderable>>) {
        if !self.visible {return}
        let args = *args;
        let depth = *depth;
        list.extend(self.channel_scroll.draw(args, Vector2::zero(), depth));
        list.extend(self.messages_scroll.draw(args, Vector2::zero(), depth));
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