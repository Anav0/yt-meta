use chrono::prelude::*;
use chrono::{Date, DateTime, Local, NaiveDate, Offset, TimeZone, Utc};
use clap::{arg, command, Parser, Subcommand};
use csv::WriterBuilder;
use diesel::dsl::max;
use diesel::prelude::*;
use dotenvy::dotenv;
use std::{
    fs::{self, DirEntry},
    io::Error,
    path::Path,
};
use ytdlp::YtDlp;

pub mod models;
pub mod schema;
pub mod ytdlp;

use models::VideoInfo;

use crate::models::MostRecentForChannel;
use crate::schema::videos::{dsl::*, star};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    #[command()]
    Monitor { channels_file_path: String },
}
pub fn establish_connection() -> PgConnection {
    dotenv().ok();

    let database_url = dotenvy::var("DATABASE_URL").expect("DATABASE_URL must be set");

    PgConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}

fn main() {
    let args = Args::parse();

    let mut db_conn = establish_connection();

    match args.command {
        Commands::Monitor { channels_file_path } => {
            let most_recent_by_channel: Vec<MostRecentForChannel> = videos
                .group_by(channel_url)
                .select((channel_url, max(upload_date)))
                .load::<MostRecentForChannel>(&mut db_conn)
                .expect("Failed to fetch information about most recent video uploded by channel");

            let output_path = "./data";

            let ytdlp = YtDlp::new(&output_path);
            let channel_file_contents = fs::read_to_string(&channels_file_path).expect(&format!(
                "Failed to read channels file at: '{channels_file_path}'"
            ));

            // We go one by one not to upset YT servers
            for line in channel_file_contents.lines() {
                let trimed_line = line.trim();
                if trimed_line.starts_with("#") || trimed_line == "" {
                    continue;
                }

                if Path::new(output_path).exists() {
                    fs::remove_dir_all(output_path).unwrap();
                }

                let mut start_download_from = NaiveDate::from_ymd_opt(1970, 1, 1).unwrap();
                for ch in &most_recent_by_channel {
                    if ch.channel_url == line {
                        start_download_from = ch.max.unwrap();
                        break;
                    }
                }

                println!("Downloading videos metadata for channel: '{line}' starting from {start_download_from}");
                ytdlp.download_meta(line, &start_download_from);

                let downloaded_file_paths: Vec<Result<DirEntry, Error>> =
                    fs::read_dir(&output_path)
                        .expect("Failed to read output directory")
                        .collect();

                let total = downloaded_file_paths.len();

                let db_conn = &mut establish_connection();

                let mut fail_count = 0;
                let mut videos_list: Vec<VideoInfo> = vec![];
                for path in downloaded_file_paths {
                    let path_u = path.unwrap();

                    let file_content =
                        fs::read_to_string(path_u.path()).expect("Failed to read metafile");

                    match serde_json::from_str::<VideoInfo>(&file_content) {
                        Ok(video) => {
                            videos_list.push(video);
                        }
                        Err(err) => {
                            dbg!(err);
                            fail_count += 1;
                            let file_name = path_u.file_name();
                            let l = file_name.to_str().unwrap();
                            println!("Failed to parse meta file: '{l}'");
                        }
                    }
                }
                let videos_len = videos_list.len();
                println!("Parsed {videos_len} meta files");
                if fail_count > 0 {
                    println!("Failed to parse: {fail_count} out of {total} meta files");
                }

                if videos_len > 1000 {
                    for chunk in videos_list.chunks(1000) {
                        diesel::insert_into(videos)
                            .values(chunk)
                            .on_conflict(id)
                            .do_nothing()
                            .execute(db_conn)
                            .expect("Failed to insert videos into db");
                    }
                } else {
                    diesel::insert_into(videos)
                        .values(videos_list)
                        .on_conflict(id)
                        .do_nothing()
                        .execute(db_conn)
                        .expect("Failed to insert videos into db");
                }
            }
        }
    }
}
