use std::sync::Arc;

use async_trait::async_trait;
use songbird::input::Compose;
use twilight_interactions::command::{ CommandModel, CreateCommand };
use twilight_model::application::interaction::Interaction;

use crate::{
    commands::traits::HandleCommand,
    innertube::InnerTube,
    interaction_context::CommandInteractionContext,
    state::State,
};

#[derive(CreateCommand, CommandModel)]
#[command(name = "play", desc = "Play some music with Berd!")]
pub(crate) struct PlayCommand {
    /// The music to look for.
    query: String,
}

#[async_trait]
impl HandleCommand for PlayCommand {
    async fn handle_mut(
        &mut self,
        interaction: Interaction,
        state: Arc<State>
    ) -> anyhow::Result<()> {
        let ic = CommandInteractionContext::new(&state.http, &interaction);
        ic.defer(false).await?;

        let tube = InnerTube::new();
        let mut results = tube.search(&self.query).await?;
        if results.is_empty() {
            ic.create_followup(&ic.interaction.token).content("No results found :(").await?;
            return Ok(());
        }

        let mut result = results
            .drain(..)
            .next()
            .unwrap();

        if let Ok(metadata) = result.aux_metadata().await {
            ic
                .create_followup(&ic.interaction.token)
                .content(
                    &format!(
                        "Playing **{}** - **{}**",
                        metadata.artist.unwrap_or("Unknown".to_string()),
                        metadata.track.unwrap_or("Unknown".to_string())
                    )
                ).await?;
        }

        let guild = interaction.guild_id.unwrap();
        if let Some(birdx) = state.songbird.get(guild) {
            let mut call = birdx.lock().await;
            let handle = call.play_input(result.into());
            state.put_handle(guild, handle);
        } else {
            ic
                .create_followup(&ic.interaction.token)
                .content("Hmm... you sure we're in the same room?").await?;
        }

        Ok(())
    }
}
