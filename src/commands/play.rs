use std::sync::Arc;

use anyhow::Context;
use async_trait::async_trait;
use songbird::{
    input::Compose,
    tracks::TrackHandle,
    Event as SongbirdEvent,
    EventContext,
    EventHandler as SongbirdEventHandler,
    TrackEvent,
};
use twilight_interactions::command::{ CommandModel, CreateCommand };
use twilight_model::{ application::interaction::Interaction, id::{ marker::GuildMarker, Id } };

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
            ic.create_followup(&interaction.token).content("No results found :(").await?;
            return Ok(());
        }

        let mut result = results
            .drain(..)
            .next()
            .unwrap();

        let Some(guild) = interaction.guild_id else {
            ic.respond("Hmm, we're not in a server!").await?;
            return Ok(());
        };

        if let Some(birdx) = state.songbird.get(guild) {
            let mut call = birdx.lock().await;
            if state.has_guild_data(&guild) {
                // when there's guild data, there's 100% a handle.
                // guranteed because there is no Option<T> block

                if let Ok(metadata) = result.aux_metadata().await {
                    ic
                        .create_followup(&interaction.token)
                        .content(
                            &format!(
                                "Added **{}** - **{}** to queue!",
                                metadata.artist.unwrap_or("Unknown".to_string()),
                                metadata.track.unwrap_or("Unknown".to_string())
                            )
                        ).await?;
                }

                state.add_track(&guild, result);
            } else {
                if let Ok(metadata) = result.aux_metadata().await {
                    ic
                        .create_followup(&interaction.token)
                        .content(
                            &format!(
                                "Playing **{}** - **{}**",
                                metadata.artist.unwrap_or("Unknown".to_string()),
                                metadata.track.unwrap_or("Unknown".to_string())
                            )
                        ).await?;
                }

                let handle = call.play_input(result.into());
                add_track_handle_events(&handle, guild, state.clone())?;
                state.put_handle(guild, handle);
            }
        } else {
            ic
                .create_followup(&interaction.token)
                .content("Hmm... you sure we're in the same room?").await?;
        }

        Ok(())
    }
}

struct TrackHandleEvents {
    state: Arc<State>,
    guild: Id<GuildMarker>,
}

#[async_trait]
impl SongbirdEventHandler for TrackHandleEvents {
    async fn act(&self, _: &EventContext<'_>) -> Option<SongbirdEvent> {
        tracing::info!("Song finished");

        if let Some(track) = self.state.pop_track(&self.guild) {
            let birdx = self.state.songbird.get_or_insert(self.guild);
            tracing::info!("Next song: {:?}", track);

            let mut call = birdx.lock().await;
            let handle = call.play_input(track.into());

            if let Err(e) = add_track_handle_events(&handle, self.guild, self.state.clone()) {
                tracing::error!(?e, "Failed to register event handler for track");
                return None;
            }

            self.state.put_handle(self.guild, handle);
        } else {
            tracing::info!("Removed guild data: {}", &self.guild);
            self.state.remove_guild_data(&self.guild);
        }

        None
    }
}

fn add_track_handle_events(
    handle: &TrackHandle,
    guild: Id<GuildMarker>,
    state: Arc<State>
) -> anyhow::Result<()> {
    let event_handler = TrackHandleEvents { state, guild };
    Ok(
        handle
            .add_event(SongbirdEvent::Track(TrackEvent::End), event_handler)
            .context("Failed to register event handler for track")?
    )
}
