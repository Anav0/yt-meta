use chrono::{Date, NaiveDate, Offset, TimeZone, Utc};
use clap::{arg, command, Parser, Subcommand};
use csv::WriterBuilder;
use diesel::{Connection, PgConnection, RunQueryDsl};
use dotenvy::dotenv;
use std::{
    fs::{self, DirEntry},
    io::Error,
    str::FromStr,
};
use ytdlp::YtDlp;

pub mod models;
pub mod schema;
pub mod ytdlp;

use models::VideoInfo;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    #[command()]
    CSV {
        output: String,
        #[arg(short = 'f')]
        channels_file_path: String,
    },

    #[command()]
    Monitor {
        channels_file_path: String,
        from: Option<String>,
    },
}
pub fn establish_connection() -> PgConnection {
    dotenv().ok();
    dotenvy::from_filename(".env").expect("Failed to read .env file");

    let database_url = dotenvy::var("DATABASE_URL").expect("DATABASE_URL must be set");

    PgConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}

fn main() {
    let args = Args::parse();
    use self::schema::videos::dsl::*;

    Utc.timestamp(20200703, 0);

    match args.command {
        Commands::CSV {
            output,
            channels_file_path,
        } => {
            let ytdlp = YtDlp::new(&output);
            let mut wtr = WriterBuilder::new().delimiter(b';').from_writer(vec![]);

            let content = fs::read_to_string(&channels_file_path).expect(&format!(
                "Failed to read channels file at: '{channels_file_path}'"
            ));

            // We go one by one not to upset YT servers
            for line in content.lines() {
                let trimed_line = line.trim();
                if trimed_line.starts_with("#") || trimed_line == "" {
                    continue;
                }
                println!("Downloading videos metadata for channel: '{line}'");
                ytdlp.download_meta(line);
            }
            let downloaded_file_paths: Vec<Result<DirEntry, Error>> = fs::read_dir(&output)
                .expect("Failed to read output directory")
                .collect();

            let mut i = 0;
            let total = downloaded_file_paths.len();
            for path in downloaded_file_paths {
                let path_u = path.unwrap();

                if !path_u.metadata().unwrap().is_file() {
                    continue;
                }

                let file_content =
                    fs::read_to_string(path_u.path()).expect("Failed to read metafile");

                let video: VideoInfo = serde_json::from_str(&file_content).unwrap();

                wtr.serialize(video)
                    .expect("Failed to serialize video from metafile: ''");

                if i % 10 == 0 {
                    println!("Processed {i} out of {total}");
                }
                i += 1;
            }
            let text = String::from_utf8(wtr.into_inner().unwrap()).unwrap();
            fs::write(format!("{output}/info.csv"), text)
                .expect("Failed to save results into csv file. Check output location");
            println!("Done!")
        }
        Commands::Monitor {
            channels_file_path,
            from,
        } => {
            let output = "./data";
            let downloaded_file_paths: Vec<Result<DirEntry, Error>> = match fs::read_dir(output) {
                Ok(dir) => dir.collect(),
                Err(err) => {
                    let ytdlp = YtDlp::new(&output);
                    let mut wtr = WriterBuilder::new().delimiter(b';').from_writer(vec![]);

                    let content = fs::read_to_string(&channels_file_path).expect(&format!(
                        "Failed to read channels file at: '{channels_file_path}'"
                    ));

                    // We go one by one not to upset YT servers
                    for line in content.lines() {
                        let trimed_line = line.trim();
                        if trimed_line.starts_with("#") || trimed_line == "" {
                            continue;
                        }
                        println!("Downloading videos metadata for channel: '{line}'");
                        ytdlp.download_meta(line);
                    }
                    fs::read_dir(&output)
                        .expect("Failed to read output directory")
                        .collect()
                }
            };

            let mut i = 0;
            let total = downloaded_file_paths.len();

            let db_conn = &mut establish_connection();

            let mut fail_count = 0;
            let mut videos_list: Vec<VideoInfo> = vec![];
            for path in downloaded_file_paths {
                let path_u = path.unwrap();
                let metadata = path_u.metadata().unwrap();
                let modified = metadata.modified().unwrap();
                //TODO: use from param here
                if !metadata.is_file() {
                    continue;
                }

                let file_content =
                    fs::read_to_string(path_u.path()).expect("Failed to read metafile");

                match serde_json::from_str::<VideoInfo>(&file_content) {
                    Ok(video) => {
                        videos_list.push(video);
                        if i % 10 == 0 {
                            println!("Processed {i} out of {total}");
                        }
                        i += 1;
                    }
                    Err(err) => {
                        dbg!(err);
                        fail_count += 1;
                        let file_name = path_u.file_name();
                        let l = file_name.to_str().unwrap();
                        println!("Failed to parse metafile: '{l}'");
                    }
                }
            }

            diesel::insert_into(videos)
                .values(videos_list)
                .execute(db_conn)
                .expect("Failed to insert videos into db");

            println!("Failed to parse: {fail_count} out of {total} meta files");
        }
    }
}
