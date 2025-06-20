use anyhow::Result;
use reqwest::header::HeaderMap;
use rustypipe::{ client::RustyPipe, model::VideoItem, param::StreamFilter };
use songbird::input::{
    core::io::MediaSource,
    AudioStream,
    AudioStreamError,
    AuxMetadata,
    Compose,
    HttpRequest,
    Input,
};

pub(crate) struct InnerTube {
    pub(crate) pipe: RustyPipe,
}

impl InnerTube {
    #[must_use]
    pub(crate) fn new() -> Self {
        Self {
            pipe: RustyPipe::new(),
        }
    }

    pub(crate) async fn search<K: AsRef<str> + std::fmt::Debug>(
        &self,
        q: K
    ) -> Result<Vec<PlayableAudio>> {
        let mut items = self.pipe.query().search::<VideoItem, _>(q).await?.items;

        let mut results = vec![];
        for item in items.items.drain(..) {
            let player = self.pipe.query().player(item.id).await?;
            let details = &player.details;

            let stream = player.select_audio_stream(&StreamFilter::default()).unwrap();
            let s = stream.url.clone();
            let size = stream.size;
            results.push(
                PlayableAudio::new(s, size, AuxMetadata {
                    track: details.name.clone(),
                    artist: details.channel_name.clone(),
                    album: None,
                    date: None,
                    channels: None,
                    channel: None,
                    start_time: None,
                    duration: None,
                    sample_rate: None,
                    source_url: None,
                    title: None,
                    thumbnail: None,
                })
            );
        }

        Ok(results)
    }
}

#[derive(Debug)]
pub(crate) struct PlayableAudio {
    url: String,
    size: u64,
    metadata: Option<AuxMetadata>,
}

impl PlayableAudio {
    pub(crate) fn new(url: String, size: u64, metadata: AuxMetadata) -> Self {
        Self { url, size, metadata: Some(metadata) }
    }
}

#[async_trait::async_trait]
impl Compose for PlayableAudio {
    fn create(
        &mut self
    ) -> std::result::Result<
        songbird::input::AudioStream<Box<dyn songbird::input::core::io::MediaSource>>,
        songbird::input::AudioStreamError
    > {
        Err(AudioStreamError::Unsupported)
    }

    async fn create_async(
        &mut self
    ) -> std::result::Result<AudioStream<Box<dyn MediaSource>>, AudioStreamError> {
        let client = reqwest::Client::new();
        let mut req = HttpRequest {
            client,
            request: self.url.drain(..).collect::<String>(),
            headers: HeaderMap::new(),
            content_length: Some(self.size),
        };

        req.create_async().await
    }

    fn should_create_async(&self) -> bool {
        true
    }

    async fn aux_metadata(&mut self) -> std::result::Result<AuxMetadata, AudioStreamError> {
        Ok(self.metadata.take().unwrap())
    }
}

impl From<PlayableAudio> for Input {
    fn from(val: PlayableAudio) -> Self {
        Input::Lazy(Box::new(val))
    }
}
