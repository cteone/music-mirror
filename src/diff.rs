use std::{
    cmp::Ordering,
    path::{Path, PathBuf},
};

use rayon::prelude::*;
use walkdir::{DirEntry, WalkDir};

use crate::compare_file::{compare_file, compare_metadata};
use crate::structs::{Codec, Diff, Diffs, FFProbe, FileType, Source, Target};
fn is_hidden(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.starts_with("."))
        .unwrap_or(false)
}

fn is_ignored(
    excludes: &Vec<glob::Pattern>,
    includes: &Vec<glob::Pattern>,
    entry: &DirEntry,
) -> bool {
    let entry = entry.path().to_str();

    if includes.iter().any(|g| g.matches(entry.unwrap())) {
        //TODO: get rid of
        //unwraps
        return false;
    }

    entry
        .map(|s| excludes.iter().any(|g| g.matches(s)))
        .unwrap_or(false)
}

pub fn diff(source: &Source, target: &Target) -> Diffs {
    let diff_files: Vec<Diff> = vec![];
    let diff_directories: Vec<Diff> = vec![];

    let mut files_to_compare: Vec<(PathBuf, PathBuf, FileType)> = vec![];

    let mut diffs = Diffs {
        directories: diff_directories,
        files: diff_files,
    };

    let input_walker = WalkDir::new(&source.library.path)
        .sort_by_file_name()
        .into_iter()
        .filter_entry(|e| !is_hidden(e) && !is_ignored(&target.excludes, &target.includes, e));

    let mut output_walker = WalkDir::new(&target.library.path)
        .sort_by_file_name()
        .into_iter();

    let mut target_path_current = next_path(&mut output_walker);

    for entry in input_walker {
        let entry = entry.unwrap(); //or error
        let source_path = entry.into_path();

        let relative_path = source_path.strip_prefix(&source.library.path).unwrap();

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

        loop {
            if let Some((path_current, _file_type_current)) = &target_path_current {
                match target_path_expected.cmp(path_current) {
                    Ordering::Less => {
                        add_to_diff(
                            Diff::Create {
                                source_path: source_path.clone(),
                                target_path: target_path_expected.clone(),
                                file_type: target_file_type.clone(),
                            },
                            &mut diffs,
                        );
                        break;
                    }
                    Ordering::Equal => {
                        //both exist: compare metadata and bitrate
                        //should be this file_type or the other one?
                        if let FileType::Audio(_, _) = file_type {
                            files_to_compare.push((
                                source_path,
                                path_current.clone(),
                                target_file_type,
                            ));
                        }
                        target_path_current = next_path(&mut output_walker);
                        break;
                    }
                    Ordering::Greater => {
                        //none is unreachable, maybe rewrite
                        if let Some((target_path_prev, target_file_type_prev)) = target_path_current
                        {
                            target_path_current = next_path(&mut output_walker);

                            add_to_diff(
                                Diff::Remove {
                                    target_path: target_path_prev.clone(),
                                    file_type: target_file_type_prev.clone(),
                                },
                                &mut diffs,
                            );
                        } //none is unreachable
                        continue;
                    }
                }
            } else {
                add_to_diff(
                    Diff::Create {
                        source_path: source_path.clone(),
                        target_path: target_path_expected.clone(),
                        file_type: target_file_type.clone(),
                    },
                    &mut diffs,
                );
                break;
            }
        }
    }

    while let Some((target_path, target_file_type)) = next_path(&mut output_walker) {
        add_to_diff(
            Diff::Remove {
                target_path: target_path,
                file_type: target_file_type,
            },
            &mut diffs,
        )
    }

    let files: Vec<Diff> = files_to_compare
        .par_iter()
        .filter_map(|(source, target, target_file_type)| {
            match compare_file(source, target, target_file_type) {
                Ok(false) => Some(Diff::Modify {
                    source_path: source.clone(),
                    target_path: target.clone(),
                    file_type: *target_file_type,
                }),
                _ => None,
            }
        })
        .collect();

    for diff in files {
        add_to_diff(diff, &mut diffs);
    }

    let (not_removes_files, removes): (Vec<Diff>, Vec<Diff>) = diffs
        .files
        .into_iter()
        .partition(|diff| !matches!(diff, Diff::Remove { .. }));

    let files = not_removes_files
        .into_iter()
        .chain(removes.into_iter().rev())
        .collect();

    let (not_removes_directories, removes): (Vec<Diff>, Vec<Diff>) = diffs
        .directories
        .into_iter()
        .partition(|diff| !matches!(diff, Diff::Remove { .. }));

    let directories = not_removes_directories
        .into_iter()
        .chain(removes.into_iter().rev())
        .collect();

    Diffs {
        files: files,
        directories: directories,
    }
}

fn add_to_diff(diff: Diff, diffs: &mut Diffs) {
    match diff {
        Diff::Create { file_type, .. }
        | Diff::Modify { file_type, .. }
        | Diff::Remove { file_type, .. } => match file_type {
            FileType::Directory => diffs.directories.push(diff),
            _ => diffs.files.push(diff),
        },
    }
}

fn next_path(walker: &mut walkdir::IntoIter) -> Option<(PathBuf, FileType)> {
    let entry = walker.next()?.ok()?;

    let path = entry.into_path();

    let file_type: FileType = if path.is_dir() {
        FileType::Directory
    } else {
        let input_extension = path.extension().and_then(|ext| ext.to_str());
        match input_extension {
            Some(ext) if ext == Codec::Aac.extension() => FileType::Audio(Codec::Aac, None),
            Some(ext) if ext == Codec::Flac.extension() => FileType::Audio(Codec::Flac, None),
            Some(ext) if ext == Codec::Alac.extension() => FileType::Audio(Codec::Alac, None),
            Some(ext) if ext == Codec::Mp3.extension() => FileType::Audio(Codec::Mp3, None),
            Some(ext) if ext == Codec::Opus.extension() => FileType::Audio(Codec::Opus, None),
            Some(_) => FileType::Other,
            None => FileType::Other,
        }
    };
    Some((path, file_type))
}

// fn compare_metadata(input_path: &PathBuf, output_path: &PathBuf) -> Result<bool, ()> {
//     let input = Command::new("ffprobe")
//         .arg("-v")
//         .arg("quiet")
//         .arg("-print_format")
//         .arg("json")
//         .arg("-show_format")
//         .arg("-show_streams")
//         .arg(input_path)
//         .output()
//         .expect("failed to read audio info");
//     let output = Command::new("ffprobe")
//         .arg("-v")
//         .arg("quiet")
//         .arg("-print_format")
//         .arg("json")
//         .arg("-show_format")
//         .arg("-show_streams")
//         .arg(output_path)
//         .output()
//         .expect("failed to read audio info");
//
//     let input_json = String::from_utf8(input.stdout).unwrap();
//     let output_json = String::from_utf8(output.stdout).unwrap();
//
//     let input_parsed: FFProbe = serde_json::from_str(&input_json).unwrap();
//     let output_parsed: FFProbe = serde_json::from_str(&output_json).unwrap();
//     // println!("{input_parsed:#?}");
//     // println!("{output_parsed:#?}");
//
//     let input_tags = input_parsed.format.unwrap().tags.unwrap();
//     let output_tags = output_parsed.format.unwrap().tags.unwrap();
//
//     // let input_streams = input_parsed.streams.unwrap();
//     // let output_streams = output_parsed.streams.unwrap();
//     //
//     // let streams_equal = input_streams
//     //     .iter()
//     //     .zip(&output_streams)
//     //     .any(|(input, output)| input != output);
//     let streams_equal = true;
//
//     let tags_equal = input_tags == output_tags;
//
//     let res = streams_equal && tags_equal;
//
//     if !res {
//         // println!("{input_streams:#?}");
//         // println!("{output_streams:#?}");
//         println!("{input_tags:#?}");
//         println!("{output_tags:#?}");
//     }
//
//     Ok(res)
// }
