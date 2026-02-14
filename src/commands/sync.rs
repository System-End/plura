use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
enum SyncStep {
    Direction,
    Entities,
    PkToken,
    SpToken,
    Done,
}

#[derive(Debug, Clone)]
pub struct SyncConversation {
    step: SyncStep,
    direction: Option<String>,
    entities: Option<Vec<String>>,
    pk_token: Option<String>,
    sp_token: Option<String>,
}

pub type SyncConversations = Arc<Mutex<HashMap<String, SyncConversation>>>; // user_id -> state

const ALL_ENTITIES: &[&str] = &["members", "switches", "systems", "groups", "messages"];
const ALL_DIRECTIONS: &[&str] = &["pk_to_plura", "plura_to_pk", "sp_to_plura", "plura_to_sp"];

/// Handles the conversational sync command flow.
/// Only allows running in DMs (channel_id starts with 'D').
/// Returns a message to send back to the user.
pub async fn handle_sync_command(
    user_id: &str,
    channel_id: &str,
    text: &str,
    conversations: SyncConversations,
) -> String {
    // Only allow in DMs (channel_id starts with 'D')
    if !channel_id.starts_with('D') {
        return "Please run `/sync` in a DM with me for your privacy.".to_string();
    }

    let mut conversations = conversations.lock().unwrap();
    let convo = conversations
        .entry(user_id.to_string())
        .or_insert(SyncConversation {
            step: SyncStep::Direction,
            direction: None,
            entities: None,
            pk_token: None,
            sp_token: None,
        });

    match convo.step {
        SyncStep::Direction => {
            let trimmed = text.trim();
            if trimmed.is_empty() || !ALL_DIRECTIONS.contains(&trimmed) {
                return format!(
                    "Which direction do you want to sync?\nReply with one of: {}",
                    ALL_DIRECTIONS.join(", ")
                );
            }
            convo.direction = Some(trimmed.to_string());
            convo.step = SyncStep::Entities;
            format!(
                "Which entities? Reply with a comma-separated list from the following:\n{}\nOr reply with `all` to sync everything.",
                ALL_ENTITIES.join(", ")
            )
        }
        SyncStep::Entities => {
            let trimmed = text.trim().to_lowercase();
            let entities: Vec<String> = if trimmed == "all" {
                ALL_ENTITIES.iter().map(|s| s.to_string()).collect()
            } else {
                trimmed
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| ALL_ENTITIES.contains(&s.as_str()))
                    .collect()
            };
            if entities.is_empty() {
                return format!(
                    "Please reply with at least one valid entity from: {}\nOr reply with `all`.",
                    ALL_ENTITIES.join(", ")
                );
            }
            convo.entities = Some(entities);
            convo.step = SyncStep::PkToken;
            match convo.direction.as_deref() {
                Some("pk_to_plura") | Some("plura_to_pk") => {
                    "Please provide your PluralKit token.".to_string()
                }
                Some("sp_to_plura") | Some("plura_to_sp") => {
                    "Please provide your SimplyPlural token.".to_string()
                }
                _ => "Please provide the relevant token.".to_string(),
            }
        }
        SyncStep::PkToken => {
            if text.trim().is_empty() {
                return match convo.direction.as_deref() {
                    Some("pk_to_plura") | Some("plura_to_pk") => {
                        "Please provide your PluralKit token.".to_string()
                    }
                    Some("sp_to_plura") | Some("plura_to_sp") => {
                        "Please provide your SimplyPlural token.".to_string()
                    }
                    _ => "Please provide the relevant token.".to_string(),
                };
            }
            match convo.direction.as_deref() {
                Some("pk_to_plura") | Some("plura_to_pk") => {
                    convo.pk_token = Some(text.trim().to_string());
                    convo.step = SyncStep::SpToken;
                    "Please provide your SimplyPlural token.".to_string()
                }
                Some("sp_to_plura") | Some("plura_to_sp") => {
                    convo.sp_token = Some(text.trim().to_string());
                    convo.step = SyncStep::SpToken;
                    "Please provide your PluralKit token.".to_string()
                }
                _ => {
                    convo.pk_token = Some(text.trim().to_string());
                    convo.step = SyncStep::SpToken;
                    "Please provide the other token.".to_string()
                }
            }
        }
        SyncStep::SpToken => {
            if text.trim().is_empty() {
                return match convo.direction.as_deref() {
                    Some("pk_to_plura") | Some("plura_to_pk") => {
                        "Please provide your SimplyPlural token.".to_string()
                    }
                    Some("sp_to_plura") | Some("plura_to_sp") => {
                        "Please provide your PluralKit token.".to_string()
                    }
                    _ => "Please provide the other token.".to_string(),
                };
            }
            match convo.direction.as_deref() {
                Some("pk_to_plura") | Some("plura_to_pk") => {
                    convo.sp_token = Some(text.trim().to_string());
                }
                Some("sp_to_plura") | Some("plura_to_sp") => {
                    convo.pk_token = Some(text.trim().to_string());
                }
                _ => {}
            }
            convo.sp_token = Some(text.trim().to_string());
            convo.step = SyncStep::Done;
            let summary = format!(
                "Syncing!\nDirection: {:?}\nEntities: {:?}\nPK Token: [hidden]\nSP Token: [hidden]",
                convo.direction, convo.entities
            );
            // Optionally, remove the conversation state now:
            conversations.remove(user_id);
            summary
        }
        SyncStep::Done => {
            "Sync already completed. Run `/sync` again to start a new sync.".to_string()
        }
    }
}
