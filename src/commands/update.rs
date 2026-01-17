use std::{cmp::min, env};

use inquire::Confirm;
use log::debug;
use reqwest::header::USER_AGENT;

use tokio::runtime::Runtime;

use current_platform::CURRENT_PLATFORM;
use futures_util::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use serde::Deserialize;

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Clone, Deserialize, Debug)]
struct Release {
    tag_name: String,
}

pub(crate) fn update() -> Result<(), String> {
    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        let client = reqwest::Client::new();

        let resp = client
            .get("https://api.github.com/repos/UbiHome/UbiHome/releases")
            .header(USER_AGENT, format!("UbiHome {}", VERSION)) 
            .send()
            .await
            .unwrap();

        let json = resp.json::<Vec<Release>>().await.unwrap();
        let new_version = json[0].tag_name.clone();

        if new_version == format!("v{}", VERSION) {
            println!("Already on the latest version: {}", VERSION);
            return Ok(());
        }


        let ans = Confirm::new(&format!("Update to version {}?", new_version))
            .with_default(true)
            .with_help_message(format!(
                "This will overwrite the current ({}) executable.",
                VERSION
            ).as_str())
            .prompt();

        if ans.unwrap() {
            // e.g. x86_64-unknown-linux-gnu
            let target_tuple = CURRENT_PLATFORM.split("-").collect::<Vec<_>>(); 
            let target_arch = target_tuple[0];
            let target_os = target_tuple[2];
            #[cfg(not(target_os = "windows"))]
            let file_ending = "";
            #[cfg(target_os = "windows")]
            let file_ending = ".exe";

            let download_file_name = format!("ubihome-{}-{}{}", target_os, target_arch, file_ending);

            let download_url = format!("https://github.com/UbiHome/UbiHome/releases/download/{}/{}", new_version, download_file_name);
            debug!("Download URL: {}", download_url);

            println!("Downloading...");
            let resp = client.get(download_url).send().await.expect("request failed");
            if resp.status() != reqwest::StatusCode::OK {
                return Err(format!("Failed to download file: {}", resp.status()));
            }

            let total_size = resp.content_length().unwrap_or(0);

            // Setup progress bar
            let pb = if total_size > 0 {
                let pb = ProgressBar::new(total_size);
                pb.set_style(ProgressStyle::default_bar()
                    .template("{msg}\n{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})")
                    .unwrap()
                    .progress_chars("#>-"));
                pb.set_message(format!("Downloading {}", download_file_name));
                pb
            } else {
                let pb = ProgressBar::new_spinner();
                pb.set_style(ProgressStyle::default_spinner()
                    .template("{msg}\n{spinner:.green} [{elapsed_precise}] {bytes} ({bytes_per_sec})")
                    .unwrap());
                pb.set_message(format!("Downloading {}", download_file_name));
                pb
            };

            // Download chunks with progress bar
            let mut downloaded: u64 = 0;
            let mut stream = resp.bytes_stream();
            let mut body_data = Vec::new();

            while let Some(item) = stream.next().await {
                let chunk = item.map_err(|e| format!("Error while downloading file: {}", e))?;
                body_data.extend_from_slice(&chunk);
                let new = if total_size > 0 {
                    min(downloaded + (chunk.len() as u64), total_size)
                } else {
                    downloaded + (chunk.len() as u64)
                };
                downloaded = new;
                pb.set_position(new);
            }

            pb.finish_with_message(format!("Downloaded {}", download_file_name));

            match env::current_exe() {
                Ok(exe_path) => {
                    let mut new_exe_path = exe_path.clone();
                    new_exe_path.set_file_name(format!("new_{}", new_exe_path.file_name().unwrap_or_default().to_string_lossy()));
                    std::fs::write(&new_exe_path, body_data).expect("Failed to create temporary file");

                    print!("Updating {}. ", exe_path.display());
                    self_replace::self_replace(&new_exe_path).unwrap();
                    std::fs::remove_file(&new_exe_path).unwrap();
                    println!("Updated!");
                }
                Err(e) => println!("failed to get current exe path: {e}"),
            };
            return Ok(());
        } else{
            println!("Update cancelled.");
            return Ok(());
        }

    }).unwrap();
    Ok(())
}
