use anyhow::{Context, Result};
use clap::Parser;
use reqwest::Client;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::time::Duration;
use tokio::time::sleep;

const WAYBACK_API: &str = "https://web.archive.org/cdx/search/cdx";

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Name of the person to greet
    #[arg(short, long)]
    target_url: String,

    /// Number of times to greet
    #[arg(short, long, default_value_t = String::from("waybackmachine_data"))]
    output_dir: String,
}

#[tokio::main]
async fn main() -> Result<()> {

    let args = Args::parse();
    
    // 保存ディレクトリを作成
    std::fs::create_dir_all(&args.output_dir)?;

    let client = Client::new();
    let snapshots = get_archived_snapshots(&client, &args.target_url).await?;

    if snapshots.is_empty() {
        println!("No snapshots found.");
    } else {
        println!("Found {} snapshots.", snapshots.len());
        for (index, snapshot) in snapshots.iter().enumerate() {
            sleep(Duration::from_secs(10)).await;

            let timestamp = &snapshot[2];
            let original_url = &snapshot[3];
            let snapshot_url = format!("https://web.archive.org/web/{}/{}", timestamp, original_url);

            let file_name = format!("{}.html", timestamp);
            let save_path = format!("{}/{}", &args.output_dir, file_name);

            println!("Saving {} {}/{}", save_path, index + 1, snapshots.len());
            download_snapshot(&client, &snapshot_url, &save_path).await?;
            println!("Saved {} {}/{}", save_path, index + 1, snapshots.len());
        }
    }

    Ok(())
}

async fn get_archived_snapshots(client: &Client, url: &str) -> Result<Vec<Vec<String>>> {
    let request_url = format!("{}?url={}&output=json&collapse=digest&matchType=prefix", WAYBACK_API, url);
    let response = client.get(&request_url).timeout(Duration::from_secs(180)).send().await?;
    let body = response.text().await?;

    let lines: Vec<&str> = body.lines().collect();
    if lines.len() < 2 {
        return Ok(vec![]);
    }

    let snapshots = lines.iter().skip(1).map(|line| {
        line.split(',').map(|s| s.replace('"', "")).collect()
    }).collect();

    Ok(snapshots)
}

async fn download_snapshot(client: &Client, snapshot_url: &str, save_path: &str) -> Result<()> {
    let response = client.get(snapshot_url).timeout(Duration::from_secs(180)).send().await?;
    let body = response.text().await?;

    let path = Path::new(save_path);
    let mut file = File::create(path).with_context(|| format!("Failed to create file: {}", save_path))?;
    file.write_all(body.as_bytes()).with_context(|| format!("Failed to write to file: {}", save_path))?;

    Ok(())
}
