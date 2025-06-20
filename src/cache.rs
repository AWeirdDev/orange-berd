use dashmap::{ DashMap, mapref::one::Ref };
use twilight_model::{ id::{ marker::UserMarker, Id }, voice::VoiceState };

#[derive(Debug, Default)]
pub(crate) struct Cache {
    voice_states: DashMap<Id<UserMarker>, VoiceState>,
}

impl Cache {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn record_voice_state(&self, id: Id<UserMarker>, state: VoiceState) {
        self.voice_states.insert(id, state);
    }

    pub(crate) fn get_voice_state(
        &self,
        id: &Id<UserMarker>
    ) -> Option<Ref<'_, Id<UserMarker>, VoiceState>> {
        self.voice_states.get(id)
    }
}
