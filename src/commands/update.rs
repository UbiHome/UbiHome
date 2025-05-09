use std::{env, fs::File, io::{self, Read}};

use inquire::Confirm;
use log::debug;
use reqwest::header::USER_AGENT;

use tokio::runtime::Runtime;

use serde::Deserialize;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const DOWNLOAD_FILE_NAME: &str = env!("DOWNLOAD_FILE_NAME");

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


        let ans = Confirm::new(&format!("Update to version {}?", new_version))
            .with_default(true)
            .with_help_message("This will overwrite the current executable.")
            .prompt();

        if ans.unwrap() {
            let download_url = format!("https://github.com/UbiHome/UbiHome/releases/download/{}/{}", new_version, DOWNLOAD_FILE_NAME);
            debug!("Download URL: {}", download_url);
            
            println!("Downloading...");
            let resp = client.get(download_url).send().await.expect("request failed");
            if resp.status() != reqwest::StatusCode::OK {
                return Err(format!("Failed to download file: {}", resp.status()));
            }
            let body = resp.bytes().await.expect("body invalid");
            match env::current_exe() {
                Ok(exe_path) => {
                    let mut new_exe_path = exe_path.clone();
                    new_exe_path.set_file_name(format!("new_{}", new_exe_path.file_name().unwrap_or_default().to_string_lossy()));
                    std::fs::write(&new_exe_path, body).expect("Failed to create temporary file");

                    println!("Updating executable...");
                    self_replace::self_replace(&new_exe_path).unwrap();
                    std::fs::remove_file(&new_exe_path).unwrap();
                    println!("Updated: {}", exe_path.display());
                }
                Err(e) => println!("failed to get current exe path: {e}"),
            };
            return Ok(());
        } else{
            println!("Update cancelled.");
            return Ok(());
        }

    }).unwrap();

    // TODO: 
    Ok(())
}
