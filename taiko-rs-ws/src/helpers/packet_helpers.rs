use crate::prelude::*;
use crate::create_packet;

pub fn create_server_send_message_packet(id: u32, message: String, channel: String) -> Vec<u8> {
    create_packet!(Server_SendMessage {
        sender_id: id,
        message,
        channel
    })
}

pub async fn create_server_score_update_packet(user_id: u32, mode: PlayMode) -> Vec<u8> {
    // user_ranked_score, user_total_score, user_accuracy, play_count, rank
    let res = Database::get_user_score_info(user_id, mode).await;

    create_packet!(Server_ScoreUpdate {
        user_id,
        total_score: res.0,
        ranked_score: res.1,
        accuracy: res.2,
        play_count: res.3,
        rank:res.4
    })
}

pub fn create_server_status_update_packet (user: &UserConnection) -> Vec<u8> {
    create_packet!(Server_UserStatusUpdate {
        user_id: user.user_id,
        action: user.action,
        action_text: user.action_text.clone(),
        mode: user.mode
    })
}

