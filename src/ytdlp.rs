use std::process::Command;

pub struct YtDlp<'a> {
    output: &'a str,
}

#[cfg(target_os = "windows")]

fn get_ytdlp_location() -> String {
    String::from("./yt-dlp/yt-dlp.exe")
}

#[cfg(target_os = "linux")]
fn get_ytdlp_location() -> String {
    String::from("./yt-dlp/yt-dlp")
}

impl<'a> YtDlp<'a> {
    pub fn new(output: &'a str) -> Self {
        Self { output }
    }

    pub fn download_meta(&self, url: &str) {
        let ytdlp_location = get_ytdlp_location();

        let status = Command::new(ytdlp_location)
            .args([
                "--ignore-errors",
                "--write-info-json",
                "--no-write-playlist-metafiles",
                "--compat-options",
                "no-playlist-metafiles",
                "--skip-download",
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
