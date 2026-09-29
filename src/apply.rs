use rayon::prelude::*;
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use crate::structs::{Diff, Diffs, FileType};

pub fn apply(diffs: &Diffs) {
    let _ = diffs.directories.iter().try_for_each(|diff| match diff {
        Diff::Create {
            source_path,
            target_path,
            file_type,
        } => create(source_path, target_path, file_type),
        Diff::Modify {
            source_path,
            target_path,
            file_type,
        } => modify(source_path, target_path, file_type),
        Diff::Remove {
            target_path,
            file_type,
        } => remove(target_path, file_type),
    });

    let _ = diffs.files.par_iter().try_for_each(|diff| match diff {
        Diff::Create {
            source_path,
            target_path,
            file_type,
        } => create(source_path, target_path, file_type),

        Diff::Modify {
            source_path,
            target_path,
            file_type,
        } => modify(source_path, target_path, file_type),

        Diff::Remove {
            target_path,
            file_type,
        } => remove(target_path, file_type),
    });
}

fn modify(input_path: &PathBuf, output_path: &PathBuf, file_type: &FileType) -> Result<(), ()> {
    remove(output_path, file_type)?;
    create(input_path, output_path, file_type)
}

fn remove(output_path: &PathBuf, file_type: &FileType) -> Result<(), ()> {
    match file_type {
        FileType::Directory => fs::remove_dir(output_path).map_err(|_| ()),
        FileType::Audio(_, _) => fs::remove_file(output_path).map_err(|_| ()),
        FileType::Other => fs::remove_file(output_path).map_err(|_| ()),
    }
}

fn create(input_path: &Path, output_path: &Path, file_type: &FileType) -> Result<(), ()> {
    match file_type {
        FileType::Audio(codec, bitrate) => {
            let bitrate = bitrate.unwrap();
            let status = Command::new("ffmpeg")
                .arg("-i")
                .arg(input_path)
                .arg("-c:a")
                .arg(codec.encoder())
                .arg("-b:a")
                .arg(format!("{bitrate}k"))
                .arg("-c:v")
                .arg("copy")
                .arg(output_path)
                .status()
                .expect("Failed to execute process");

            if status.success() { Ok(()) } else { Err(()) }
        }
        FileType::Directory => fs::create_dir(output_path).map_err(|_| ()),
        FileType::Other => fs::copy(input_path, output_path)
            .map(|_| ())
            .map_err(|_| ()),
    }
}
