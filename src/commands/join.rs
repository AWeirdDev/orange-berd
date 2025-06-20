use std::sync::Arc;

use async_trait::async_trait;

use twilight_interactions::command::{ CreateCommand, CommandModel };
use twilight_model::application::interaction::{ Interaction, InteractionChannel };
use twilight_util::builder::InteractionResponseDataBuilder;

use crate::{
    commands::traits::HandleCommand,
    interaction_context::CommandInteractionContext,
    state::State,
};

#[derive(CreateCommand, CommandModel)]
#[command(name = "join", desc = "Join (connect to) a voice channel, or yours.")]
pub(crate) struct JoinCommand {
    /// The channel to join. If not specified, Berd will join your channel.
    #[command(channel_types = "guild_voice", channel_types = "guild_stage_voice")]
    channel: Option<InteractionChannel>,
}

#[async_trait]
impl HandleCommand for JoinCommand {
    async fn handle_mut(
        &mut self,
        interaction: Interaction,
        state: Arc<State>
    ) -> anyhow::Result<()> {
        let ic = CommandInteractionContext::new(&state.http, &interaction);

        let channel_id = {
            if let Some(channel) = self.channel.take() {
                channel.id
            } else {
                let user_id = interaction.author_id().unwrap();
                let ch_id = if let Some(vs) = state.cache.get_voice_state(&user_id) {
                    vs.channel_id
                } else {
                    let vs = state.http
                        .user_voice_state(interaction.guild_id.unwrap(), user_id).await?
                        .model().await?;
                    state.cache.record_voice_state(user_id, vs);
                    state.cache.get_voice_state(&user_id).unwrap().channel_id
                };

                if ch_id.is_none() {
                    ic.respond(
                        InteractionResponseDataBuilder::new()
                            .content("You're not connected to a voice channel!")
                            .build()
                    ).await?;
                    return Ok(());
                }
                ch_id.unwrap()
            }
        };

        ic.defer(false).await?;
        if let Err(e) = state.songbird.join(interaction.guild_id.unwrap(), channel_id).await {
            if e.should_reconnect_driver() {
                ic
                    .create_followup(&ic.interaction.token)
                    .content(
                        "It seems like I cannot join the voice channel for now, but this failure can be reattempted. You can use the `/join` command again."
                    ).await?;
            } else if e.should_leave_server() {
                ic
                    .create_followup(&ic.interaction.token)
                    .content(
                        "Oh, no... it seems like Discord's voice gateway state isn't consistent right now, we should probably retry later."
                    ).await?;
            }

            return Ok(());
        }

        ic
            .create_followup(&ic.interaction.token)
            .content(&format!("Joined <#{}>!", channel_id)).await?;

        Ok(())
    }
}
