use anyhow::Result;
use reqwest::header::HeaderMap;
use rustypipe::{ client::RustyPipe, model::MusicItem, param::StreamFilter };
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
    ) -> Result<Vec<YouTubeAudio>> {
        let mut items = self.pipe.query().music_search_main(q).await?.items;

        let mut results = vec![];
        for item in items.items.drain(..) {
            if let MusicItem::Track(track) = item {
                let player = self.pipe.query().player(track.id).await?;
                let details = &player.details;

                let stream = player.select_audio_stream(&StreamFilter::default()).unwrap();
                let s = stream.url.clone();
                let size = stream.size;
                results.push(
                    YouTubeAudio::new(s, size, AuxMetadata {
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
        }

        Ok(results)
    }
}

#[derive(Debug)]
pub(crate) struct YouTubeAudio {
    url: String,
    size: u64,
    metadata: Option<AuxMetadata>,
}

impl YouTubeAudio {
    pub(crate) fn new(url: String, size: u64, metadata: AuxMetadata) -> Self {
        Self { url, size, metadata: Some(metadata) }
    }
}

#[async_trait::async_trait]
impl Compose for YouTubeAudio {
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

impl From<YouTubeAudio> for Input {
    fn from(val: YouTubeAudio) -> Self {
        Input::Lazy(Box::new(val))
    }
}
