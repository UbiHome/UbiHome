use reqwest::header::USER_AGENT;
use std::collections::HashMap;

use tokio::runtime::Runtime;

use serde::Deserialize;

#[derive(Clone, Deserialize, Debug)]
struct Release {
    tag_name: String,
}

pub(crate) fn update() -> Result<(), reqwest::Error> {
    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        let client = reqwest::Client::new();

        let resp = client
            .get("https://api.github.com/repos/UbiHome/UbiHome/releases")
            .header(USER_AGENT, "UbiHome") // TODO: Add version
            .send()
            .await
            .unwrap();

        let json = resp.json::<Vec<Release>>().await.unwrap();
        let new_version = json[0].tag_name.clone();

        println!("{new_version:#?}");
    });
    Ok(())
}
