use std::io::{self, Read};

use flate2::read::GzDecoder;

use reqwest::header::USER_AGENT;
use tar::Archive;

use tokio::runtime::Runtime;
use futures_lite::StreamExt;

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

        // TODO: scrape that and release files directly to release.

        let full_url = "https://github.com/UbiHome/UbiHome/releases/download/v0.5.2/ubihome-Linux-musl-x86_64.tar.gz";

        let response;
        match client.get(full_url).send().await {
            Ok(res) => response = res,
            Err(error) => {
                return Err(io::Error::new(io::ErrorKind::InvalidData, error));
            }
        };
    
        let (tx, rx) = flume::bounded(0);
    
        let decoder_thread = std::thread::spawn(move || {
            let input = ChannelRead::new(rx);
            let gz = GzDecoder::new(input);
            let mut archive = Archive::new(gz);
            archive.unpack("./").unwrap();
        });
    
        if response.status() == reqwest::StatusCode::OK {
            let mut stream = response.bytes_stream();
    
            while let Some(item) = stream.next().await {
                let chunk = item
                    .or(Err(format!("Error while downloading file")))
                    .unwrap();
                tx.send_async(chunk.to_vec()).await.unwrap();
            }
            drop(tx); // close the channel to signal EOF
        }
    
        tokio::task::spawn_blocking(|| decoder_thread.join())
            .await
            .unwrap()
            .unwrap();

        Ok(())

    }).unwrap();

    // TODO: 
    Ok(())
}


// Wrap a channel into something that impls `io::Read`
struct ChannelRead {
    rx: flume::Receiver<Vec<u8>>,
    current: io::Cursor<Vec<u8>>,
}

impl ChannelRead {
    fn new(rx: flume::Receiver<Vec<u8>>) -> ChannelRead {
        ChannelRead {
            rx,
            current: io::Cursor::new(vec![]),
        }
    }
}

impl Read for ChannelRead {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if self.current.position() == self.current.get_ref().len() as u64 {
            // We've exhausted the previous chunk, get a new one.
            if let Ok(vec) = self.rx.recv() {
                self.current = io::Cursor::new(vec);
            }
            // If recv() "fails", it means the sender closed its part of
            // the channel, which means EOF. Propagate EOF by allowing
            // a read from the exhausted cursor.
        }
        self.current.read(buf)
    }
}