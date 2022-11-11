// @generated automatically by Diesel CLI.

diesel::table! {
    videos (id) {
        id -> Text,
        webpage_url -> Nullable<Text>,
        is_live -> Nullable<Bool>,
        age_limit -> Nullable<Int2>,
        uploader_id -> Nullable<Text>,
        channel -> Nullable<Text>,
        channel_follower_count -> Nullable<Int8>,
        playlist_id -> Nullable<Text>,
        playlist_title -> Nullable<Text>,
        playlist_index -> Nullable<Int4>,
        display_id -> Nullable<Text>,
        view_count -> Nullable<Int8>,
        acodec -> Nullable<Text>,
        fulltitle -> Nullable<Text>,
        title -> Nullable<Text>,
        description -> Nullable<Text>,
        format -> Nullable<Text>,
        fps -> Nullable<Float8>,
        tags -> Nullable<Text>,
        thumbnail -> Nullable<Text>,
        upload_date -> Nullable<Date>,
        ext -> Nullable<Text>,
        duration -> Nullable<Int4>,
    }
}
