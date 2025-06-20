use std::collections::VecDeque;

use dashmap::DashMap;
use songbird::{ tracks::TrackHandle, Songbird };

use twilight_http::Client as HttpClient;
use twilight_model::id::{ marker::GuildMarker, Id };

use crate::{ cache::Cache, innertube::PlayableAudio };

#[derive(Debug)]
pub struct GuildData {
    pub(crate) queue: VecDeque<PlayableAudio>,
    pub(crate) handle: TrackHandle,
}

#[derive(Debug)]
pub(crate) struct State {
    pub(crate) http: HttpClient,
    pub(crate) guild_data: DashMap<Id<GuildMarker>, GuildData>,
    pub(crate) songbird: Songbird,
    pub(crate) cache: Cache,
}

impl State {
    pub(crate) fn new(http: HttpClient, songbird: Songbird) -> Self {
        Self {
            http,
            guild_data: DashMap::new(),
            songbird,
            cache: Cache::new(),
        }
    }

    pub(crate) fn put_handle(&self, guild: Id<GuildMarker>, handle: TrackHandle) {
        if let Some(mut data) = self.guild_data.get_mut(&guild) {
            data.handle = handle;
        } else {
            self.guild_data.insert(guild, GuildData { queue: VecDeque::new(), handle });
        }
    }

    /// You must use [`State::put_handle`] first.
    pub(crate) fn add_track(&self, guild: &Id<GuildMarker>, track: PlayableAudio) {
        if let Some(mut rf) = self.guild_data.get_mut(guild) {
            rf.queue.push_back(track);
        }
    }
}
