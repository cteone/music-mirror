//TODO: WIP

use std::{path::Path, time::Duration};

use notify_debouncer_mini::{DebounceEventResult, DebouncedEvent, new_debouncer, notify::*};

use crate::{
    compare_file::{compare_file, compare_metadata},
    structs::{Codec, Config, FileType},
};

pub fn watcher(config: &Config) {
    let mut debouncer = new_debouncer(
        Duration::from_secs(10),
        |res: DebounceEventResult| match res {
            Ok(events) => events
                .iter()
                .for_each(|e| println!("Event {:?} for {:?}", e.kind, e.path)),
            Err(e) => println!("Error {:?}", e),
        },
    )
    .unwrap();

    debouncer
        .watcher()
        .watch(&config.source.library.path, RecursiveMode::Recursive)
        .unwrap();

    loop {}
}

fn handle_file_update(event: &DebouncedEvent, config: &Config) {
    let source_path = &event.path;

    let relative_path = source_path
        .strip_prefix(&config.source.library.path)
        .unwrap();

    let file_type: FileType = if source_path.is_dir() {
        FileType::Directory
    } else {
        let source_extension = relative_path.extension().and_then(|ext| ext.to_str());
        match source_extension {
            //need to revise since different ones can have the same extension
            Some(ext) if ext == Codec::Aac.extension() => FileType::Audio(Codec::Aac, None),
            Some(ext) if ext == Codec::Flac.extension() => FileType::Audio(Codec::Flac, None),
            Some(ext) if ext == Codec::Alac.extension() => FileType::Audio(Codec::Alac, None),
            Some(ext) if ext == Codec::Mp3.extension() => FileType::Audio(Codec::Mp3, None),
            Some(ext) if ext == Codec::Opus.extension() => FileType::Audio(Codec::Opus, None),
            Some(_) => FileType::Other,
            None => FileType::Other,
        }
    };

    for target in &config.targets {
        let target_file_type = match file_type {
            FileType::Directory => FileType::Directory,
            FileType::Audio(codec, bitrate) => {
                let (codec, bitrate) = target
                    .transcodes
                    .rules
                    .get(&codec)
                    .unwrap_or(&target.transcodes.default);
                FileType::Audio(codec.clone(), Some(bitrate.clone()))
            }
            FileType::Other => FileType::Other,
        };

        let target_path_expected = match target_file_type {
            FileType::Directory => Path::new(&target.library.path).join(relative_path),
            FileType::Audio(codec, bitrate) => {
                let input_stem = source_path.file_stem().unwrap().to_str().unwrap();
                let mut path = Path::new(&target.library.path).to_path_buf();

                if let Some(parent) = relative_path.parent() {
                    path = path.join(parent);
                }

                path.join(input_stem.to_owned() + "." + codec.extension())
            }
            FileType::Other => Path::new(&target.library.path).join(relative_path),
        };

        //     match target_path_expected.try_exists() {
        //         Ok(true) => {
        //             if let FileType::Audio(_, _) = file_type {
        //                 match compare_file(source_path, &target_path_expected, target_file_type) {
        //                     Ok(false) => Some(Diff::Modify {
        //                         source_path: source.clone(),
        //                         target_path: target.clone(),
        //                         file_type: *target_file_type,
        //                     }),
        //                     _ => None,
        //                 }
        //             }
        //         }
        //         Ok(false) => {}
        //         Err(error) => error,
        //     }
        // }
    }
}
