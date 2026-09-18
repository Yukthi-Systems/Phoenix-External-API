pub mod auth;
mod rmq;


/// Sends a message to RabbitMQ for assigning a new server to a mailbox
pub async fn assign_new_mailbox_server(email_id: String) {
    rmq::send_mailbox_message_to_rmq(
        "new_email",
        &serde_json::json!({
            "email": email_id
        })
    ).await.unwrap_or_else(|err| {
        log::error!("Failed to send new mailbox server message: {}", err);
    });
}
