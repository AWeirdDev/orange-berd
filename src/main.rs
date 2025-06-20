use std::sync::Arc;

use songbird::{ shards::TwilightMap, Songbird };
use tracing::instrument;
use twilight_gateway::{ Event, EventTypeFlags, Intents, Shard, ShardId, StreamExt };
use twilight_http::Client as HttpClient;
use twilight_interactions::command::CreateCommand;
use twilight_model::application::{ command::Command, interaction::InteractionData };

use crate::{ commands::BerdCommands, state::State };

mod innertube;
mod interaction_context;
mod commands;
mod cache;
mod state;

#[tokio::main]
#[instrument]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    dotenvy::dotenv_override().ok();

    let token = dotenvy::var("BERD_DISCORD_TOKEN")?;

    let http = HttpClient::new(token.clone());
    let user_id = http.current_user().await?.model().await?.id;

    {
        let commands = [Command::from(BerdCommands::create_command())];
        let ic = http.interaction(user_id.cast());
        ic.set_global_commands(&commands).await?;
    }

    let intents = Intents::GUILD_MESSAGES | Intents::GUILD_VOICE_STATES | Intents::MESSAGE_CONTENT;
    let config = twilight_gateway::Config::new(token, intents);
    let shards: Vec<Shard> = twilight_gateway
        ::create_recommended(&http, config, |_, builder| builder.build()).await?
        .collect();

    let senders = TwilightMap::new(
        shards
            .iter()
            .map(|item| (item.id().number(), item.sender()))
            .collect()
    );
    let songbird = Songbird::twilight(Arc::new(senders), user_id);

    let state = Arc::new(State::new(http, songbird));

    let mut set = tokio::task::JoinSet::new();
    for shard in shards {
        set.spawn(tokio::spawn(runner(shard, state.clone())));
    }
    set.join_all().await;

    Ok(())
}

#[instrument]
async fn runner(mut shard: Shard, state: Arc<State>) {
    let shard_id = shard.id();
    while let Some(item) = shard.next_event(EventTypeFlags::all()).await {
        let Ok(event) = item else {
            let err = item.unwrap_err();
            tracing::error!(?err, "Error receiving event");
            continue;
        };
        tokio::spawn({
            let state = state.clone();
            async move {
                handle_event(&shard_id, event, state).await;
            }
        });
    }
}

async fn handle_event(shard: &ShardId, event: Event, state: Arc<State>) {
    state.songbird.process(&event).await;

    match event {
        Event::Ready(_) => {
            tracing::info!("Shard {} is ready", shard);
        }
        Event::VoiceStateUpdate(vsu) => {
            let vsu = *vsu;
            let vs = vsu.0;
            let user_id = vs.user_id;
            state.cache.record_voice_state(user_id, vs);
        }
        Event::InteractionCreate(icre) => {
            let mut interaction = (*icre).0;
            let data = interaction.data.take().unwrap();

            if let InteractionData::ApplicationCommand(cmd) = data {
                if let Err(e) = BerdCommands::run(interaction, *cmd, state).await {
                    tracing::error!(?e);
                }
            }
        }
        _ => {
            return;
        }
    }
}
