use std::sync::Arc;

use anyhow::{ Context, Result };
use twilight_interactions::command::{ CommandModel, CreateCommand };
use twilight_model::application::interaction::{ application_command::CommandData, Interaction };

use crate::{
    commands::{
        join::JoinCommand,
        pause_resume::{ PauseCommand, ResumeCommand },
        play::PlayCommand,
        stop::StopCommand,
        traits::HandleCommand,
    },
    state::State,
};

mod traits;
mod join;
mod play;
mod stop;
mod pause_resume;

#[derive(CreateCommand, CommandModel)]
#[command(name = "berd", desc = "The Berd music bot.")]
pub(crate) enum BerdCommands {
    #[command(name = "join")] Join(JoinCommand),
    #[command(name = "play")] Play(PlayCommand),
    #[command(name = "stop")] Stop(StopCommand),
    #[command(name = "pause")] Pause(PauseCommand),
    #[command(name = "resume")] Resume(ResumeCommand),
}

impl BerdCommands {
    pub(crate) async fn run(
        interaction: Interaction,
        data: CommandData,
        state: Arc<State>
    ) -> Result<()> {
        let command = Self::from_interaction(data.into()).context("Parsing command data")?;
        match command {
            Self::Join(mut join) => join.handle_mut(interaction, state).await,
            Self::Play(mut play) => play.handle_mut(interaction, state).await,
            Self::Stop(mut stop) => stop.handle_mut(interaction, state).await,
            Self::Pause(mut pause) => pause.handle_mut(interaction, state).await,
            Self::Resume(mut resume) => resume.handle_mut(interaction, state).await,
        }
    }
}
