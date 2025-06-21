//! Notes:
//! **DO NOT** give out `Ref` to an item in [`GuildData`], if not dropped, a **dead lock** may be present.

use std::collections::VecDeque;

use dashmap::DashMap;
use songbird::{ tracks::TrackHandle, Songbird };

use twilight_http::Client as HttpClient;
use twilight_model::id::{ marker::GuildMarker, Id };

use crate::{ cache::Cache, innertube::YouTubeAudio };

#[derive(Debug)]
pub struct GuildData {
    pub(crate) queue: VecDeque<YouTubeAudio>,
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

    pub(crate) fn has_guild_data(&self, guild: &Id<GuildMarker>) -> bool {
        self.guild_data.contains_key(guild)
    }

    pub(crate) fn remove_guild_data(
        &self,
        guild: &Id<GuildMarker>
    ) -> Option<(Id<GuildMarker>, GuildData)> {
        self.guild_data.remove(guild)
    }

    /// You must use [`State::put_handle`] first, if there's nothing playing.
    pub(crate) fn add_track(&self, guild: &Id<GuildMarker>, track: YouTubeAudio) {
        if let Some(mut rf) = self.guild_data.get_mut(guild) {
            rf.queue.push_back(track);
        }
    }

    pub(crate) fn pop_track(&self, guild: &Id<GuildMarker>) -> Option<YouTubeAudio> {
        if let Some(mut rf) = self.guild_data.get_mut(guild) { rf.queue.pop_front() } else { None }
    }
}
