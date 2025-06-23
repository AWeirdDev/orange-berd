use std::sync::Arc;

use async_trait::async_trait;
use twilight_interactions::command::{ CommandModel, CommandOption, CreateCommand, CreateOption };
use twilight_model::application::interaction::Interaction;

use crate::{
    commands::traits::HandleCommand,
    interaction_context::CommandInteractionContext,
    state::{ RepeatMode, State },
};

#[derive(CreateOption, CommandOption)]
pub(crate) enum ModeOption {
    #[option(name = "Single", value = 0)]
    Single,

    #[option(name = "No repeat", value = 1)]
    No,
}

impl ModeOption {
    fn get_mode(&self) -> RepeatMode {
        match self {
            Self::Single => RepeatMode::Single,
            Self::No => RepeatMode::No,
        }
    }
}

#[derive(CreateCommand, CommandModel)]
#[command(name = "repeat", desc = "Set the repeat mode.")]
pub(crate) struct RepeatCommand {
    #[command(desc = "The mode")]
    mode: ModeOption,
}

#[async_trait]
impl HandleCommand for RepeatCommand {
    async fn handle_mut(
        &mut self,
        interaction: Interaction,
        state: Arc<State>
    ) -> anyhow::Result<()> {
        let ic = CommandInteractionContext::new(&state.http, &interaction);
        let mode = self.mode.get_mode();
        let guild = interaction.guild_id.unwrap();

        if let Some(mut rf) = state.guild_data.get_mut(&guild) {
            rf.repeat = mode;
            ic.respond("Set!").await?;
        } else {
            ic.respond("Hmm, are we in the same room?").await?;
        }

        Ok(())
    }
}
