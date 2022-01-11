use crate::prelude::*;
use crate::create_packet;

pub fn create_server_send_message_packet(id: u32, message: String, channel: String) -> Vec<u8> {
    create_packet!(
        PacketId::Server_SendMessage,
        id,
        message,
        channel
    )
}

pub async fn create_server_score_update_packet(user_id: u32, mode: PlayMode) -> Vec<u8> {
    // user_ranked_score, user_total_score, user_accuracy, play_count, rank
    let res = Database::get_user_score_info(user_id, mode).await;

    create_packet!(
        PacketId::Server_ScoreUpdate,
        user_id,
        res.0, // user_total_score,
        res.1, // user_ranked_score,
        res.2, // user_accuracy,
        res.3, // play_count,
        res.4 // rank
    )
}

pub fn create_server_status_update_packet (user: &UserConnection) -> Vec<u8> {
    create_packet!(
        PacketId::Server_UserStatusUpdate,
        user.user_id,
        user.action,
        user.action_text.clone(),
        user.mode
    )
}

