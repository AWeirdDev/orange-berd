use std::sync::Arc;

use async_trait::async_trait;
use twilight_interactions::command::{ CommandModel, CreateCommand };
use twilight_model::application::interaction::Interaction;

use crate::{
    commands::traits::HandleCommand,
    interaction_context::CommandInteractionContext,
    state::State,
};

#[derive(CreateCommand, CommandModel)]
#[command(name = "stop", desc = "Stop the current music player")]
pub(crate) struct StopCommand;

#[async_trait]
impl HandleCommand for StopCommand {
    async fn handle_mut(
        &mut self,
        interaction: Interaction,
        state: Arc<State>
    ) -> anyhow::Result<()> {
        let ic = CommandInteractionContext::new(&state.http, &interaction);

        let guild = interaction.guild_id.unwrap();
        if let Some(rf) = state.guild_data.get(&guild) {
            rf.handle.stop()?;
        } else {
            ic.respond("Hmm... berd is not available for now. Are we in the same room?").await?;
            return Ok(());
        }
        ic.respond("Stopped this song!").await?;

        Ok(())
    }
}
