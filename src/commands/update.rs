use std::{cmp::min, env};

use inquire::Confirm;
use reqwest::header::{AUTHORIZATION, USER_AGENT};

use tokio::runtime::Runtime;

use current_platform::CURRENT_PLATFORM;
use futures_util::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use serde::Deserialize;

const VERSION_GIT_TAG: &str = env!("GIT_TAG");
const GIT_HASH: &str = env!("GIT_HASH");
const USER_AGENT_VALUE: &str = concat!("UbiHome ", env!("GIT_TAG"));

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
    current_version_tag: &str,
) -> String {
    releases
        .iter()
        .filter(|release| include_pre_release || is_normal_release_tag(&release.tag_name))
        .take_while(|release| release.tag_name != current_version_tag)
        .map(|release| release.body.clone())
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect::<Vec<_>>()
        .join("\n\n")
}

fn current_download_file_name() -> String {
    // e.g. x86_64-unknown-linux-gnu
    let target_tuple = CURRENT_PLATFORM.split("-").collect::<Vec<_>>();
    let target_arch = target_tuple[0];
    let target_os = target_tuple[2];
    #[cfg(not(target_os = "windows"))]
    let file_ending = "";
    #[cfg(target_os = "windows")]
    let file_ending = ".exe";

    format!("ubihome-{}-{}{}", target_os, target_arch, file_ending)
}

async fn download_bytes(
    request: reqwest::RequestBuilder,
    download_file_name: &str,
) -> Result<Vec<u8>, String> {
    println!("Downloading...");
    let resp = request.send().await.expect("request failed");
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
        pb.set_style(
            ProgressStyle::default_spinner()
                .template("{msg}\n{spinner:.green} [{elapsed_precise}] {bytes} ({bytes_per_sec})")
                .unwrap(),
        );
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
    Ok(body_data)
}

fn apply_update(body_data: Vec<u8>) -> Result<(), String> {
    match env::current_exe() {
        Ok(exe_path) => {
            let mut new_exe_path = exe_path.clone();
            new_exe_path.set_file_name(format!(
                "new_{}",
                new_exe_path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
            ));
            std::fs::write(&new_exe_path, body_data).expect("Failed to create temporary file");

            print!("Updating {}. ", exe_path.display());
            self_replace::self_replace(&new_exe_path).unwrap();
            std::fs::remove_file(&new_exe_path).unwrap();
            println!("Updated!");
        }
        Err(e) => println!("failed to get current exe path: {e}"),
    };
    Ok(())
}

fn confirm_update(prompt: &str, help_message: &str) -> Result<bool, String> {
    let ans = Confirm::new(prompt)
        .with_default(true)
        .with_help_message(help_message)
        .prompt();

    match ans {
        Ok(true) => {
            println!("Updating...");
            Ok(true)
        }
        Ok(false) => {
            println!("Update cancelled.");
            Ok(false)
        }
        Err(e) => {
            match e {
                inquire::error::InquireError::OperationCanceled
                | inquire::error::InquireError::OperationInterrupted => {
                    println!("Update cancelled.");
                    return Ok(false);
                }
                _ => (),
            }
            println!("Failed to read user input: {}", e);
            Err("Failed to read user input.".to_string())
        }
    }
}

pub(crate) fn update(include_pre_release: bool) -> Result<(), String> {
    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        println!("Current version: {} ({})", VERSION_GIT_TAG, GIT_HASH);

        let client = reqwest::Client::new();

        let resp = client
            .get("https://api.github.com/repos/UbiHome/UbiHome/releases")
            .header(USER_AGENT, USER_AGENT_VALUE)
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

        if new_version == format!("v{}", VERSION_GIT_TAG) {
            println!("Already on the latest version.");
            return Ok(());
        }

        let update_changelog =
            build_update_changelog(&releases, include_pre_release, VERSION_GIT_TAG);
        println!("Changes since:\n\n{}\n", update_changelog);

        if !confirm_update(
            &format!("Update to version {}?", new_version),
            &format!(
                "This will overwrite the current ({}) executable.",
                VERSION_GIT_TAG
            ),
        )? {
            return Ok(());
        }

        let download_file_name = current_download_file_name();
        let download_url = format!(
            "https://github.com/UbiHome/UbiHome/releases/download/{}/{}",
            new_version, download_file_name
        );

        let body_data = download_bytes(client.get(download_url), &download_file_name).await?;
        apply_update(body_data)
    })
}

const CI_WORKFLOW_PATH: &str = ".github/workflows/ci_rust.yml";

#[derive(Deserialize)]
struct PullRequest {
    head: PullRequestHead,
}

#[derive(Deserialize)]
struct PullRequestHead {
    sha: String,
}

#[derive(Deserialize)]
struct WorkflowRunsResponse {
    workflow_runs: Vec<WorkflowRun>,
}

#[derive(Deserialize)]
struct WorkflowRun {
    id: u64,
    path: String,
    status: String,
    conclusion: Option<String>,
}

#[derive(Deserialize)]
struct ArtifactsResponse {
    artifacts: Vec<Artifact>,
}

#[derive(Deserialize)]
struct Artifact {
    archive_download_url: String,
    expired: bool,
}

/// Updates to the latest development build published for an open pull request.
/// CI publishes each platform's binary as an unzipped, single-file workflow artifact, which
/// this fetches directly from the pull request's most recent successful CI run.
///
/// Downloading workflow artifacts always requires an authenticated GitHub API request (even for
/// public repositories), so a token with at least `repo` (or `actions:read` + `pull-requests:read`
/// for fine-grained tokens) scope must be supplied.
pub(crate) fn update_pr(pr_number: u32, github_token: String) -> Result<(), String> {
    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        println!("Current version: {} ({})", VERSION_GIT_TAG, GIT_HASH);

        let client = reqwest::Client::new();
        let auth_header = format!("Bearer {}", github_token);

        let pr = client
            .get(format!(
                "https://api.github.com/repos/UbiHome/UbiHome/pulls/{}",
                pr_number
            ))
            .header(USER_AGENT, USER_AGENT_VALUE)
            .send()
            .await
            .map_err(|e| format!("Failed to fetch PR #{}: {}", pr_number, e))?
            .error_for_status()
            .map_err(|e| format!("Failed to fetch PR #{}: {}", pr_number, e))?
            .json::<PullRequest>()
            .await
            .map_err(|e| format!("Failed to parse PR info: {}", e))?;

        let runs = client
            .get(format!(
                "https://api.github.com/repos/UbiHome/UbiHome/actions/runs?head_sha={}",
                pr.head.sha
            ))
            .header(USER_AGENT, USER_AGENT_VALUE)
            .header(AUTHORIZATION, &auth_header)
            .send()
            .await
            .map_err(|e| format!("Failed to list workflow runs: {}", e))?
            .error_for_status()
            .map_err(|e| format!("Failed to list workflow runs (check your GitHub token): {}", e))?
            .json::<WorkflowRunsResponse>()
            .await
            .map_err(|e| format!("Failed to parse workflow runs: {}", e))?;

        // `head_sha` already scopes this to the PR's current head commit, but that commit's
        // workflow can still have multiple runs (e.g. manual re-runs) - always pick the newest
        // one (highest run id) rather than trusting the API's response order.
        let Some(run) = runs
            .workflow_runs
            .iter()
            .filter(|run| {
                run.path == CI_WORKFLOW_PATH
                    && run.status == "completed"
                    && run.conclusion.as_deref() == Some("success")
            })
            .max_by_key(|run| run.id)
        else {
            return Err(format!(
                "No successful CI build found for PR #{}.",
                pr_number
            ));
        };

        let download_file_name = current_download_file_name();
        let artifacts = client
            .get(format!(
                "https://api.github.com/repos/UbiHome/UbiHome/actions/runs/{}/artifacts?name={}",
                run.id, download_file_name
            ))
            .header(USER_AGENT, USER_AGENT_VALUE)
            .header(AUTHORIZATION, &auth_header)
            .send()
            .await
            .map_err(|e| format!("Failed to look up the build artifact: {}", e))?
            .error_for_status()
            .map_err(|e| format!("Failed to look up the build artifact: {}", e))?
            .json::<ArtifactsResponse>()
            .await
            .map_err(|e| format!("Failed to parse build artifact info: {}", e))?;

        let Some(artifact) = artifacts.artifacts.iter().find(|artifact| !artifact.expired) else {
            return Err(format!(
                "No build artifact for your platform ({}) found for PR #{}.",
                download_file_name, pr_number
            ));
        };

        if !confirm_update(
            &format!("Update to the latest development build of PR #{} ({})?", pr_number, pr.head.sha),
            &format!(
                "This will overwrite the current executable ({}) with an unreleased (and potentially unsecure) development build.",
                VERSION_GIT_TAG
            ),
        )? {
            return Ok(());
        }

        let request = client
            .get(&artifact.archive_download_url)
            .header(USER_AGENT, USER_AGENT_VALUE)
            .header(AUTHORIZATION, &auth_header);

        let body_data = download_bytes(request, &download_file_name).await?;
        apply_update(body_data)
    })
}

#[cfg(test)]
mod tests {
    use super::is_normal_release_tag;
    use super::*;

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

        let changelog = build_update_changelog(&releases, false, "v1.0.0");

        assert_eq!(changelog, "older\n\nmiddle\n\nlatest");
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

        let changelog = build_update_changelog(&releases, false, "v1.0.0");

        assert_eq!(changelog, "stable");
    }
}
