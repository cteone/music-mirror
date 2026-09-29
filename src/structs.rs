use std::collections::HashMap;
use std::path::PathBuf;

use serde::Deserialize;

// pub enum Container {
//     M4a,
//     Flac,
//     Mp3,
// }

#[derive(Eq, PartialEq, Hash, Debug, Copy, Clone, Deserialize)]
pub enum Codec {
    Aac,
    Alac,
    Flac,
    Opus,
    Mp3,
}

impl Codec {
    pub fn extension(&self) -> &str {
        match self {
            Codec::Aac => "m4a",
            Codec::Alac => "m4a",
            Codec::Flac => "flac",
            Codec::Opus => "opus",
            Codec::Mp3 => "mp3",
        }
    }

    pub fn encoder(&self) -> String {
        match self {
            Codec::Aac => "aac".to_string(),
            Codec::Alac => "alac_at".to_string(),
            Codec::Flac => "flac".to_string(),
            Codec::Opus => "libopus".to_string(),
            Codec::Mp3 => "libmp3lame".to_string(),
        }
    }
}

pub struct Config {
    pub source: Source,
    pub targets: Vec<Target>,
}

pub struct Library {
    pub name: String,
    pub path: PathBuf,
}

pub struct Source {
    pub library: Library,
}

pub struct Target {
    pub library: Library,
    pub transcodes: Transcodes,
    pub includes: Vec<glob::Pattern>,
    pub excludes: Vec<glob::Pattern>,
}

pub struct Transcodes {
    pub rules: HashMap<Codec, (Codec, u32)>,
    pub default: (Codec, u32),
}

// pub struct Config {
//     pub input_library: PathBuf,
//     pub output_library: PathBuf,
//     pub codec: Codec,
//     pub bitrate: u32,
//     pub ignore: Option<Vec<glob::Pattern>>,
//     pub do_not_ignore: Option<Vec<glob::Pattern>>,
// }

#[derive(Debug, Clone, Copy)]
pub enum FileType {
    Audio(Codec, Option<u32>),
    Directory,
    Other,
}

#[derive(Debug)]
pub struct Diffs {
    pub directories: Vec<Diff>,
    pub files: Vec<Diff>,
}

#[derive(Debug)]
pub enum Diff {
    Create {
        source_path: PathBuf,
        target_path: PathBuf,
        file_type: FileType,
    },
    Modify {
        source_path: PathBuf,
        target_path: PathBuf,
        file_type: FileType,
    },
    Remove {
        target_path: PathBuf,
        file_type: FileType,
    },
}

#[derive(Debug, Deserialize)]
pub struct FFProbe {
    pub format: Option<Format>,
    pub streams: Option<Vec<Stream>>,
}

#[derive(Debug, Deserialize)]
pub struct Format {
    pub tags: Option<Tags>,
}

#[derive(Debug, Deserialize)]
pub struct Tags {
    #[serde(alias = "TITLE")]
    pub title: Option<String>,
    #[serde(alias = "ARTIST")]
    pub artist: Option<String>,
    #[serde(alias = "ALBUM_ARTIST")]
    pub album_artist: Option<String>,
    #[serde(alias = "ALBUM")]
    pub album: Option<String>,
    #[serde(alias = "DATE")]
    pub date: Option<String>,
    #[serde(alias = "GENRE")]
    pub genre: Option<String>,
    #[serde(alias = "TRACK")]
    pub track: Option<String>,
    #[serde(alias = "DISC")]
    pub disc: Option<String>,
}

impl PartialEq for Tags {
    fn eq(&self, other: &Self) -> bool {
        self.title == other.title
            && self.artist == other.artist
            && self.album_artist == other.album_artist
            && self.album == other.album
            && self.date == other.date
            && self.genre == other.genre
        // && self.track == other.track
        // && self.disc == other.disc
    }
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct Stream {
    pub codec_name: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct FileConfig {
    pub source: FileSource,
    pub targets: Vec<FileTarget>,
}

#[derive(Debug, Deserialize)]
pub struct FileSource {
    pub name: String,
    pub path: PathBuf,
}

#[derive(Debug, Deserialize)]
pub struct FileTarget {
    pub name: String,
    pub path: PathBuf,

    #[serde(default)]
    pub includes: Vec<String>,

    #[serde(default)]
    pub excludes: Vec<String>,

    pub transcodes: FileTranscodes,
}

#[derive(Debug, Deserialize)]
pub struct FileTranscodes {
    pub default: (Codec, u32),

    #[serde(default)]
    pub rules: HashMap<Codec, (Codec, u32)>,
}

impl TryFrom<FileConfig> for Config {
    type Error = String;

    fn try_from(config: FileConfig) -> Result<Self, Self::Error> {
        let source = Source {
            library: Library {
                name: config.source.name,
                path: config.source.path,
            },
        };

        let targets = config
            .targets
            .into_iter()
            .map(|target| {
                let includes = target
                    .includes
                    .into_iter()
                    .map(|pattern| {
                        glob::Pattern::new(&pattern)
                            .map_err(|e| format!("invalid include pattern '{pattern}': {e}"))
                    })
                    .collect::<Result<Vec<_>, _>>()?;

                let excludes = target
                    .excludes
                    .into_iter()
                    .map(|pattern| {
                        glob::Pattern::new(&pattern)
                            .map_err(|e| format!("invalid exclude pattern '{pattern}': {e}"))
                    })
                    .collect::<Result<Vec<_>, _>>()?;

                Ok(Target {
                    library: Library {
                        name: target.name,
                        path: target.path,
                    },
                    transcodes: Transcodes {
                        rules: target.transcodes.rules,
                        default: target.transcodes.default,
                    },
                    includes,
                    excludes,
                })
            })
            .collect::<Result<Vec<_>, String>>()?;

        Ok(Config { source, targets })
    }
}
