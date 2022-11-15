use std::{path::Path, process::Command};

use chrono::{DateTime, Local, NaiveDate};
use std::path::PathBuf;

pub struct YtDlp<'a> {
    output: &'a str,
}

#[cfg(target_os = "windows")]

fn get_ytdlp_location() -> PathBuf {
    let mut exec_path = std::env::current_exe().unwrap();
    exec_path.pop();
    exec_path.push("yt-dlp\\yt-dlp.exe");
    exec_path
}

#[cfg(target_os = "linux")]
fn get_ytdlp_location() -> PathBuf {
    let mut exec_path = std::env::current_exe().unwrap();
    exec_path.pop();
    exec_path.push("yt-dlp/yt-dlp");
    exec_path
}

impl<'a> YtDlp<'a> {
    pub fn new(output: &'a str) -> Self {
        Self { output }
    }

    pub fn download_meta(&self, url: &str, date_to_download_from: &NaiveDate) {
        let ytdlp_location = get_ytdlp_location();
        let date_str = date_to_download_from.format("%Y%m%d").to_string();

        let status = Command::new(ytdlp_location)
            .args([
                "--ignore-errors",
                "--write-info-json",
                "--no-write-playlist-metafiles",
                "--compat-options",
                "no-playlist-metafiles",
                "--skip-download",
                "--progress",
                "--quiet",
                "--dateafter",
                &date_str,
                "--paths",
                self.output,
                url,
            ])
            .status()
            .expect(&format!("Failed to download metadata for url: '{url}'"));

        if !status.success() {
            println!("Failed to download metadata for url: '{url}'")
        }
    }
}
