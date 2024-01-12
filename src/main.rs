use reqwest;
use dotenv::dotenv;
use clap::{App, Arg};
use serde::Deserialize;
use std::ffi::OsString;
use std::process::Command;
use lazy_static::lazy_static;

lazy_static! {
    static ref API_KEY: String = std::env::var("YOUTUBE_API_KEY").expect("YOUTUBE_API_KEY must be set");
}

#[derive(Deserialize, Debug)]
struct VideoInfo{
    #[serde(default)]
    id: String,
    #[serde(default)]
    snippet: Snippet,
}

#[derive(Deserialize, Debug, Default)]
struct Snippet{
    title: String
}

#[derive(Deserialize, Debug)]
struct PlaylistItems{
    items: Vec<VideoInfo>
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>>{
    dotenv().ok();

    let matches = App::new("Rust Youtube Downloader")
                    .version("0.1.0")
                    .author("Raman Sharma")
                    .about("Download youtube videos and playlists from the command line")
                    .arg(Arg::with_name("url")
                    .help("Youtube video or playlist url")
                    .required(true)
                    .index(1))
                    .get_matches_safe();

    match matches {
        Ok(matches) => {
            let video_url = matches.value_of("url").unwrap();

            if video_url.contains("playlist") {
                download_playlist(video_url).await?;
            } else {
                download_video(video_url).await?;
            }
        }
        Err(err) => {
            eprintln!("{}", err);
            std::process::exit(1);
        }
    }

    Ok(())
}

async fn download_video(video_url: &str) -> Result<(), Box<dyn std::error::Error>>{
    let video_id = extract_video_id(video_url)?;
    let video_info = get_video_info(&video_id).await?;

    println!("Downloading video: {}", video_info.snippet.title);

    let video_url_arg : OsString = video_url.into();

    // log output of video_url_arg
    println!("video_url_arg: {:?}", video_url_arg);
    

    let output = Command::new("youtube-dl")
                    .arg("-f")
                    .arg("bestvideo[ext=mp4]+bestaudio[ext=m4a]/best[ext=mp4]/best")
                    .arg("-o")
                    .arg("%(title)s.%(ext)s")
                    .arg(&video_url_arg)
                    .output()?;

    if output.status.success(){
        println!("Downloaded video: {:?}", video_info.snippet.title);
    }else{
        println!("Failed to download video: {:?}", video_info.snippet.title);
    }

    Ok(())
}

async fn download_playlist(playlist_url: &str) -> Result<(), Box<dyn std::error::Error>>{
    let playlist_id = extract_playlist_id(playlist_url)?;
    let playlist_info = get_playlist_items(&playlist_id).await?;

    println!("Downloading playlist: {}", playlist_info.items[0].snippet.title);

    for video in playlist_info.items{
        let video_url = format!("https://www.youtube.com/watch?v={}", video.id);
        let _ = download_video(&video_url).await?;
    }

    Ok(())
}

fn extract_video_id(video_url: &str) -> Result<String, &'static str>{
    let video_id: Vec<&str> = video_url.split("=").collect::<Vec<&str>>();

    if let Some(id) = video_id.last(){
        Ok(id.to_string())
    } else{
        Err("Could not extract video id".into())
    }
}

fn extract_playlist_id(playlist_url: &str) -> Result<String, &'static str>{
    let playlist_id: Vec<&str> = playlist_url.split("=").collect::<Vec<&str>>();

    if let Some(id) = playlist_id.last(){
        Ok(id.to_string())
    } else{
        Err("Could not extract playlist id".into())
    }
}

async fn get_video_info(video_id: &str) -> Result<VideoInfo, reqwest::Error>{
    let url = format!("https://www.googleapis.com/youtube/v3/videos?part=snippet&id={}&key={}", video_id, *API_KEY);
    let response: VideoInfo = reqwest::get(&url).await?.json::<VideoInfo>().await?;

    Ok(response)
}

async fn get_playlist_items(playlist_id: &str) -> Result<PlaylistItems, reqwest::Error>{
    let url = format!("https://www.googleapis.com/youtube/v3/playlistItems?playlistId={}&key={}&part=snippet&maxResults=50", playlist_id, *API_KEY);
    let response: PlaylistItems = reqwest::get(&url).await?.json::<PlaylistItems>().await?;

    Ok(response)
}

