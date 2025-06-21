use std::sync::Arc;

use async_trait::async_trait;
use songbird::tracks::PlayMode;
use twilight_interactions::command::{ CommandModel, CreateCommand };
use twilight_model::application::interaction::Interaction;

use crate::{
    commands::traits::HandleCommand,
    interaction_context::CommandInteractionContext,
    state::State,
};

#[derive(CreateCommand, CommandModel)]
#[command(name = "pause", desc = "Pause the song.")]
pub(crate) struct PauseCommand;

#[async_trait]
impl HandleCommand for PauseCommand {
    async fn handle_mut(
        &mut self,
        interaction: Interaction,
        state: Arc<State>
    ) -> anyhow::Result<()> {
        handler(interaction, state).await?;
        Ok(())
    }
}

#[derive(CreateCommand, CommandModel)]
#[command(name = "resume", desc = "Resume the song.")]
pub(crate) struct ResumeCommand;

#[async_trait]
impl HandleCommand for ResumeCommand {
    async fn handle_mut(
        &mut self,
        interaction: Interaction,
        state: Arc<State>
    ) -> anyhow::Result<()> {
        handler(interaction, state).await?;
        Ok(())
    }
}

async fn handler(interaction: Interaction, state: Arc<State>) -> anyhow::Result<()> {
    let ic = CommandInteractionContext::new(&state.http, &interaction);

    let Some(guild) = interaction.guild_id else {
        ic.respond("Hmm, we're not in a server!").await?;
        return Ok(());
    };

    if let Some(rf) = state.guild_data.get(&guild) {
        let info = rf.handle.get_info().await?;
        match info.playing {
            PlayMode::End => ic.respond("Oh, seems like this song has already ended!").await?,
            PlayMode::Stop => ic.respond("Oh, seems like this song has been stopped!").await?,
            PlayMode::Errored(_) =>
                ic.respond(
                    "It seems like there was an error while playing, and can't really restart."
                ).await?,
            PlayMode::Pause => {
                rf.handle.play()?;
                ic.respond("Resumed!").await?;
            }
            PlayMode::Play => {
                rf.handle.pause()?;
                ic.respond("Paused!").await?;
            }
            _ => (),
        }
    }

    Ok(())
}
