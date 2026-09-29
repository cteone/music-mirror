use std::{path::PathBuf, process::Command};

use crate::structs::{FFProbe, FileType};

// pub fn compare_file()
//
pub fn compare_file(
    input_path: &PathBuf,
    output_path: &PathBuf,
    target_file_type: &FileType,
) -> Result<bool, ()> {
    let input = Command::new("ffprobe")
        .arg("-v")
        .arg("quiet")
        .arg("-print_format")
        .arg("json")
        .arg("-show_format")
        .arg("-show_streams")
        .arg(input_path)
        .output()
        .expect("failed to read audio info");
    let output = Command::new("ffprobe")
        .arg("-v")
        .arg("quiet")
        .arg("-print_format")
        .arg("json")
        .arg("-show_format")
        .arg("-show_streams")
        .arg(output_path)
        .output()
        .expect("failed to read audio info");

    let input_json = String::from_utf8(input.stdout).unwrap();
    let output_json = String::from_utf8(output.stdout).unwrap();

    let input_parsed: FFProbe = serde_json::from_str(&input_json).unwrap();
    let output_parsed: FFProbe = serde_json::from_str(&output_json).unwrap();
    // println!("{input_parsed:#?}");
    // println!("{output_parsed:#?}");

    let tags_equal =
        input_parsed.format.and_then(|f| f.tags) == output_parsed.format.and_then(|f| f.tags);
    Ok(tags_equal)
}

pub fn compare_metadata(input_path: &PathBuf, output_path: &PathBuf) -> Result<bool, ()> {
    let input = Command::new("ffprobe")
        .arg("-v")
        .arg("quiet")
        .arg("-print_format")
        .arg("json")
        .arg("-show_format")
        .arg("-show_streams")
        .arg(input_path)
        .output()
        .expect("failed to read audio info");
    let output = Command::new("ffprobe")
        .arg("-v")
        .arg("quiet")
        .arg("-print_format")
        .arg("json")
        .arg("-show_format")
        .arg("-show_streams")
        .arg(output_path)
        .output()
        .expect("failed to read audio info");

    let input_json = String::from_utf8(input.stdout).unwrap();
    let output_json = String::from_utf8(output.stdout).unwrap();

    let input_parsed: FFProbe = serde_json::from_str(&input_json).unwrap();
    let output_parsed: FFProbe = serde_json::from_str(&output_json).unwrap();
    // println!("{input_parsed:#?}");
    // println!("{output_parsed:#?}");

    let input_tags = input_parsed.format.unwrap().tags.unwrap();
    let output_tags = output_parsed.format.unwrap().tags.unwrap();

    // let input_streams = input_parsed.streams.unwrap();
    // let output_streams = output_parsed.streams.unwrap();
    //
    // let streams_equal = input_streams
    //     .iter()
    //     .zip(&output_streams)
    //     .any(|(input, output)| input != output);
    let streams_equal = true;

    let tags_equal = input_tags == output_tags;

    let res = streams_equal && tags_equal;

    if !res {
        // println!("{input_streams:#?}");
        // println!("{output_streams:#?}");
        println!("{input_tags:#?}");
        println!("{output_tags:#?}");
    }

    Ok(res)
}
