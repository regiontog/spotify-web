use attohttpc::header::{AUTHORIZATION, CONTENT_LENGTH};
use derive_builder::Builder;
use serde::Serialize;
use serde_with_macros::skip_serializing_none;

use crate::scope::*;
use crate::Spotify;

type Response = attohttpc::Result<attohttpc::Response>;

/*
* Endpoints:
* Pause a User's Playback
* Seek To Position In Currently Playing Track
* Set Repeat Mode On User’s Playback
* Set Volume For User's Playback
* Skip User’s Playback To Next Track
* Skip User’s Playback To Previous Track
* Start/Resume a User's Playback
* Toggle Shuffle For User’s Playback
* Transfer a User's Playback
*/

#[derive(Serialize)]
#[serde(untagged)]
pub enum Offset {
    Position { position: u64 },
    Uri { uri: String },
}

#[skip_serializing_none]
#[derive(Serialize)]
struct ResumePlaybackBody<'a> {
    context_uri: Option<&'a str>,
    uris: Option<&'a [&'a str]>,
    offset: Option<Offset>,
    position_ms: Option<u64>,
}

#[derive(Builder, Default)]
#[builder(pattern = "owned")]
#[builder(build_fn(skip))]
#[builder(setter(strip_option))]
pub struct ResumePlayback<'a> {
    device_id: Option<&'a str>,
    context_uri: Option<&'a str>,
    uris: Option<&'a [&'a str]>,
    offset: Option<Offset>,
    position_ms: Option<u64>,
}

impl<'a> ResumePlayback<'a> {
    pub fn builder() -> ResumePlaybackBuilder<'a> {
        Default::default()
    }
}

impl<'a> From<ResumePlayback<'a>> for ResumePlaybackBody<'a> {
    fn from(pb: ResumePlayback<'a>) -> Self {
        ResumePlaybackBody {
            context_uri: pb.context_uri,
            uris: pb.uris,
            offset: pb.offset,
            position_ms: pb.position_ms,
        }
    }
}

impl<'a> ResumePlaybackBuilder<'a> {
    pub fn build(self) -> ResumePlayback<'a> {
        ResumePlayback {
            device_id: self.device_id.flatten(),
            context_uri: self.context_uri.flatten(),
            uris: self.uris.flatten(),
            offset: self.offset.flatten(),
            position_ms: self.position_ms.flatten(),
        }
    }
}

impl<Scopes> Spotify<Scopes>
where
    Scopes: Scoped<UserModifyPlaybackState>,
{
    pub fn pause_playback<'a>(&self, device_id: impl Into<Option<&'a str>>) -> Response {
        let mut req = attohttpc::put("https://api.spotify.com/v1/me/player/pause")
            .header(AUTHORIZATION, &self.authorization_header)
            .header(CONTENT_LENGTH, 0);

        if let Some(device_id) = device_id.into() {
            req = req.param("device_id", device_id);
        }

        req.send()
    }

    pub fn resume_playback<'a>(&self, params: impl Into<Option<ResumePlayback<'a>>>) -> Response {
        let params = params.into().unwrap_or_default();

        let mut req = attohttpc::put("https://api.spotify.com/v1/me/player/play")
            .header(AUTHORIZATION, &self.authorization_header);

        if let Some(device_id) = params.device_id {
            req = req.param("device_id", device_id);
        }

        req.json(&Into::<ResumePlaybackBody>::into(params))?.send()
    }
}

impl<Scopes> Spotify<Scopes>
where
    Scopes: Scoped<UserReadCurrentlyPlaying>,
{
    pub fn currently_playing<'a>(&self, market: impl Into<Option<&'a str>>) -> Response {
        let mut req = attohttpc::get("https://api.spotify.com/v1/me/player/currently-playing")
            .header(AUTHORIZATION, &self.authorization_header);

        if let Some(market) = market.into() {
            req = req.param("market", market);
        }

        req.send()
    }
}

impl<Scopes> Spotify<Scopes>
where
    Scopes: Scoped<UserReadPlaybackState>,
{
    pub fn currently_playing_state<'a>(&self, market: impl Into<Option<&'a str>>) -> Response {
        let mut req = attohttpc::get("https://api.spotify.com/v1/me/player/currently-playing")
            .header(AUTHORIZATION, &self.authorization_header);

        if let Some(market) = market.into() {
            req = req.param("market", market);
        }

        req.send()
    }
}
