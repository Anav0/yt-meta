use csv::WriterBuilder;
use rayon::prelude::*;
use serde::{de, Deserialize, Serialize};
use std::{
    env,
    fs::{self, DirEntry},
    io::Error,
};

pub fn vec_to_string<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: de::Deserializer<'de>,
{
    let buf: Vec<String> = Vec::deserialize(deserializer)?;
    Ok(buf.join(", "))
}

#[derive(Serialize, Deserialize, Debug)]
struct Thumbnail {
    id: String,
    width: Option<usize>,
    height: Option<usize>,
    resolution: Option<String>,
    url: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct VideoInfo {
    id: String,
    webpage_url: String,
    is_live: Option<bool>,
    age_limit: Option<usize>,
    uploader_id: Option<String>,
    channel: String,
    channel_follower_count: Option<usize>,
    playlist_id: Option<String>,
    playlist_title: Option<String>,
    playlist_index: Option<usize>,
    display_id: Option<String>,
    view_count: Option<usize>,
    acodec: Option<String>,
    fulltitle: Option<String>,
    title: String,
    description: String,
    format: Option<String>,
    fps: Option<f64>,
    #[serde(deserialize_with = "vec_to_string")]
    tags: String,
    thumbnail: Option<String>,
    upload_date: Option<String>,
    ext: Option<String>,
    duration: Option<usize>,
}
fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        print!("Program requires two arguments. First is a folder containing YT data and second is save location");
        return;
    }

    let yt_metadata_folder = &args[1];
    let output = &args[2];

    fs::create_dir_all(output).expect("Failed to create output folder.");

    let paths: Vec<Result<DirEntry, Error>> = fs::read_dir(yt_metadata_folder)
        .expect("Invalid YT metadata folder")
        .collect();

    if paths.len() == 0 {
        print!("No meta files found in specified folder");
        return;
    }

    let mut i = 0;
    let total = paths.len();
    println!("Found {total} files");
    let mut wtr = WriterBuilder::new().from_writer(vec![]);
    for path in paths {
        let x = path.unwrap();
        let file_content = fs::read_to_string(x.path()).unwrap();
        let video: VideoInfo = serde_json::from_str(&file_content).unwrap();
        wtr.serialize(video)
            .expect("Failed to serialize video into csv row");

        if i % 200 == 0 {
            println!("{i}/{total}");
        }
        i += 1
    }

    let text = String::from_utf8(wtr.into_inner().unwrap()).unwrap();
    fs::write("{output}/data.csv", text)
        .expect("Failed to save results into csv file. Check output location");
}
