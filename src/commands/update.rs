use std::{cmp::min, env};

use inquire::Confirm;
use log::debug;
use reqwest::header::USER_AGENT;

use tokio::runtime::Runtime;

use current_platform::CURRENT_PLATFORM;
use futures_util::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use serde::Deserialize;

const VERSION: &str = env!("GIT_TAG");

#[derive(Clone, Deserialize, Debug)]
struct Release {
    tag_name: String,
    #[serde(default)]
    body: String,
}

fn is_normal_release_tag(tag: &str) -> bool {
    if !tag.starts_with('v') {
        return false;
    }

    let mut parts = tag[1..].split('.');
    let Some(major) = parts.next() else {
        return false;
    };
    let Some(minor) = parts.next() else {
        return false;
    };
    let Some(patch) = parts.next() else {
        return false;
    };

    parts.next().is_none()
        && !major.is_empty()
        && !minor.is_empty()
        && !patch.is_empty()
        && major.chars().all(|c| c.is_ascii_digit())
        && minor.chars().all(|c| c.is_ascii_digit())
        && patch.chars().all(|c| c.is_ascii_digit())
}

fn build_update_changelog(
    releases: &[Release],
    include_pre_release: bool,
    current_version: &str,
) -> String {
    let current_tag = format!("v{}", current_version);

    releases
        .iter()
        .filter(|release| include_pre_release || is_normal_release_tag(&release.tag_name))
        .take_while(|release| release.tag_name != current_tag)
        .map(|release| format!("## {}\n\n{}", release.tag_name, release.body.trim()))
        .collect::<Vec<_>>()
        .join("\n\n")
}

#[cfg(test)]
mod tests {
    use super::{build_update_changelog, is_normal_release_tag, Release};

    #[test]
    fn accepts_normal_release_tag() {
        assert!(is_normal_release_tag("v1.2.3"));
        assert!(is_normal_release_tag("v0.0.1"));
        assert!(is_normal_release_tag("v12.34.56"));
    }

    #[test]
    fn rejects_pre_release_and_invalid_tags() {
        assert!(!is_normal_release_tag("v0.14.1-next.2"));
        assert!(!is_normal_release_tag("v1.2.3-beta"));
        assert!(!is_normal_release_tag("1.2.3"));
        assert!(!is_normal_release_tag("v1.2"));
        assert!(!is_normal_release_tag("v1.2.3.4"));
        assert!(!is_normal_release_tag("v1.2.x"));
    }

    #[test]
    fn builds_changelog_for_all_skipped_releases() {
        let releases = vec![
            Release {
                tag_name: "v1.3.0".to_string(),
                body: "latest".to_string(),
            },
            Release {
                tag_name: "v1.2.0".to_string(),
                body: "middle".to_string(),
            },
            Release {
                tag_name: "v1.1.0".to_string(),
                body: "older".to_string(),
            },
            Release {
                tag_name: "v1.0.0".to_string(),
                body: "current".to_string(),
            },
        ];

        let changelog = build_update_changelog(&releases, false, "1.0.0");

        assert_eq!(
            changelog,
            "## v1.3.0\n\nlatest\n\n## v1.2.0\n\nmiddle\n\n## v1.1.0\n\nolder"
        );
    }

    #[test]
    fn ignores_pre_releases_when_disabled() {
        let releases = vec![
            Release {
                tag_name: "v1.2.0-next.1".to_string(),
                body: "pre".to_string(),
            },
            Release {
                tag_name: "v1.1.0".to_string(),
                body: "stable".to_string(),
            },
            Release {
                tag_name: "v1.0.0".to_string(),
                body: "current".to_string(),
            },
        ];

        let changelog = build_update_changelog(&releases, false, "1.0.0");

        assert_eq!(changelog, "## v1.1.0\n\nstable");
    }
}

pub(crate) fn update(include_pre_release: bool) -> Result<(), String> {
    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        let client = reqwest::Client::new();

        let resp = client
            .get("https://api.github.com/repos/UbiHome/UbiHome/releases")
            .header(USER_AGENT, format!("UbiHome {}", VERSION)) 
            .send()
            .await
            .unwrap();

        let releases = resp.json::<Vec<Release>>().await.unwrap();
        let Some(new_version) = releases
            .iter()
            .find(|release| include_pre_release || is_normal_release_tag(&release.tag_name))
            .map(|release| release.tag_name.clone())
        else {
            return Err("No matching release found.".to_string());
        };

        if new_version == format!("v{}", VERSION) {
            println!("Already on the latest version: {}", VERSION);
            return Ok(());
        }


        let update_changelog = build_update_changelog(&releases, include_pre_release, VERSION);
        let help_message = if update_changelog.is_empty() {
            format!("This will overwrite the current ({}) executable.", VERSION)
        } else {
            format!(
                "This will overwrite the current ({}) executable.\n\n{}",
                VERSION, update_changelog
            )
        };

        let ans = Confirm::new(&format!("Update to version {}?", new_version))
            .with_default(true)
            .with_help_message(help_message.as_str())
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
            Ok(())
        } else{
            println!("Update cancelled.");
            Ok(())
        }

    }).unwrap();
    Ok(())
}
