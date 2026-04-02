use std::sync::Arc;

use error_stack::{Result, ResultExt};
use reqwest::Client;
use serde::Deserialize;
use slack_morphism::prelude::*;

use crate::{
    fetch_system,
    models::{member, user},
};

#[derive(thiserror::Error, displaydoc::Display, Debug)]
pub enum CommandError {
    /// Error calling the database
    Sqlx,
    /// Error calling the PluralKit API
    PluralKit,
}

#[derive(clap::Subcommand, Debug)]
pub enum Sync {
    /// Import members from PluralKit. Run in a DM to keep your token private.
    FromPk {
        /// Your PluralKit token (from pluralkit.me/settings)
        token: String,
    },
}

#[derive(Deserialize, Debug)]
struct PkMember {
    name: String,
    display_name: Option<String>,
    avatar_url: Option<String>,
    pronouns: Option<String>,
}

impl Sync {
    #[tracing::instrument(skip_all)]
    pub async fn run(
        self,
        event: SlackCommandEvent,
        _client: Arc<SlackHyperClient>,
        state: SlackClientEventsUserState,
    ) -> Result<SlackCommandEventResponse, CommandError> {
        let states = state.read().await;
        let user_state = states.get_user_state::<user::State>().unwrap();

        match self {
            Self::FromPk { token } => {
                fetch_system!(event, user_state => system_id);

                let http = Client::new();
                let pk_members = http
                    .get("https://api.pluralkit.me/v2/systems/@me/members")
                    .header("User-Agent", "Plura/0.1 (https://github.com/Suya1671/plura)")
                    .header("Authorization", token.trim())
                    .send()
                    .await
                    .change_context(CommandError::PluralKit)
                    .attach_printable("Failed to reach PluralKit API")?
                    .error_for_status()
                    .change_context(CommandError::PluralKit)
                    .attach_printable("PluralKit API returned an error — is your token correct?")?
                    .json::<Vec<PkMember>>()
                    .await
                    .change_context(CommandError::PluralKit)
                    .attach_printable("Failed to parse PluralKit API response")?;

                let count = pk_members.len();

                for pk_member in pk_members {
                    let display_name = pk_member
                        .display_name
                        .unwrap_or_else(|| pk_member.name.clone());
                    member::View {
                        full_name: pk_member.name,
                        display_name,
                        profile_picture_url: pk_member.avatar_url,
                        pronouns: pk_member.pronouns,
                        title: None,
                        name_pronunciation: None,
                        name_recording_url: None,
                    }
                    .add(system_id, &user_state.db)
                    .await
                    .change_context(CommandError::Sqlx)?;
                }

                Ok(SlackCommandEventResponse::new(
                    SlackMessageContent::new().with_text(
                        format!("Imported {count} member(s) from PluralKit!").into(),
                    ),
                ))
            }
        }
    }
}
