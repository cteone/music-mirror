# music-mirror
A CLI written in Rust to mirror a music library to multiple targets with many configuration options. Requires FFmpeg.
## Usage
Build using `cargo build --release`.
After building run
```
./music-mirror --config [path/to/config] --target [target library name]
```
If omitted, --config defaults to config.toml

## Configuration
Below is a sample configuration file:
```toml
[source]
name = "Drive"
path = "/Volumes/SSD/Music/"

[[targets]]
name = "Laptop"
path = "/Users/name/Music/"

includes = ["*/Band/Live", "*/Band/Live/*"]

excludes = [
  "*/Live",
  "*/Live/*",
  "*/Compilation",
  "*/Compilation/*",
  "*.lrc",
  "*.jpg",
  "*.png",
  "*.pdf",
]

[targets.transcodes]
default = ["Aac", 256]

[targets.transcodes.rules]
Alac = ["Aac", 256]
Flac = ["Aac", 256]
Aac = ["Aac", 256]
Opus = ["Aac", 256]
Mp3 = ["Aac", 256]

[[targets]]
name = "iPod"
path = "/Volumes/iPod/Music"

includes = []

excludes = ["*.lrc", "*.jpg", "*.png", "*.pdf"]

[targets.transcodes]
default = ["Aac", 128]

[targets.transcodes.rules]
Alac = ["Aac", 128]
Flac = ["Aac", 128]
Aac = ["Aac", 128]
Opus = ["Aac", 128]
Mp3 = ["Aac", 128]
```
## Note
Early in development




