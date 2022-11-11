use std::{fmt, time::SystemTime};

use crate::schema::videos;
use chrono::{Date, DateTime, NaiveDate, NaiveDateTime, TimeZone, Utc};
use diesel::{prelude::*, sql_types::Timestamp};
use serde::{de, Deserialize, Serialize};
use serde_with::{serde_as, TimestampSeconds};

#[derive(Insertable, Queryable, Serialize, Deserialize, Debug)]
#[diesel(table_name=videos)]
pub struct VideoInfo {
    id: String,
    webpage_url: String,
    is_live: Option<bool>,
    age_limit: Option<i16>,
    uploader_id: Option<String>,
    channel: String,
    channel_follower_count: Option<i64>,
    playlist_id: Option<String>,
    playlist_title: Option<String>,
    playlist_index: Option<i32>,
    display_id: Option<String>,
    view_count: Option<i64>,
    acodec: Option<String>,
    fulltitle: Option<String>,
    title: String,
    description: String,
    format: Option<String>,
    fps: Option<f64>,
    #[serde(deserialize_with = "vec_to_string")]
    tags: String,
    thumbnail: Option<String>,
    #[serde(deserialize_with = "yt_date_to_date")]
    upload_date: NaiveDate,
    ext: Option<String>,
    duration: Option<i32>,
}

pub fn yt_date_to_date<'de, D>(deserializer: D) -> Result<NaiveDate, D::Error>
where
    D: de::Deserializer<'de>,
{
    let buf = String::deserialize(deserializer)?;

    let format = "%Y%m%d";

    let date = NaiveDate::parse_from_str(&buf, format).expect("Failed to parse YT date");

    Ok(date)
}
pub fn vec_to_string<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: de::Deserializer<'de>,
{
    let buf: Vec<String> = Vec::deserialize(deserializer)?;
    Ok(buf.join(" "))
}
