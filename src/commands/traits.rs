use std::sync::Arc;

use async_trait::async_trait;
use twilight_interactions::command::CommandModel;
use twilight_model::application::interaction::Interaction;

use crate::state::State;

#[async_trait]
pub(crate) trait HandleCommand: CommandModel {
    async fn handle_mut(
        &mut self,
        interaction: Interaction,
        state: Arc<State>
    ) -> anyhow::Result<()>;
}
