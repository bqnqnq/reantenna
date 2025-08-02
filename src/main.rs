// SPDX-License-Identifier: Apache-2.0
use clap::{Parser, Subcommand};
use colored::Colorize;
use reqwest::blocking::Client;
use reqwest::header::AUTHORIZATION;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs;

#[derive(Parser)]
#[command(about = "Reantenna is CLI tools for Misskey.")]
struct Cli {
    #[command(subcommand)]
    subcommand: SubCommands,
}

#[derive(Subcommand)]
enum SubCommands {
    /// 電波をキャッチ
    Catch {
        #[arg(short, long)]
        id: Option<String>,

        #[arg(short, long, default_value_t = 10)]
        limit: u32,

        #[arg(short, long, default_value_t = true)]
        show: bool,
    },
    /// 結果を出力
    Show,
}

#[derive(Serialize, Deserialize, Debug)]
struct User {
    id: String,
    host: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct NoteData {
    user: User,
    text: String,
}

#[derive(Serialize)]
struct PostData {
    #[serde(rename = "antennaId")]
    antenna_id: String,
    limit: u32,
    #[serde(rename = "sinceDate")]
    since_date: u32,
    #[serde(rename = "untilDate")]
    until_date: u32,
}

#[derive(Deserialize)]
struct ConfigToml {
    antenna_id: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Cli::parse();
    let client = Client::new();

    let token = load_env("./.reantenna/.env")?;
    let config = load_config("./.reantenna/config.toml")?;

    match args.subcommand {
        SubCommands::Catch { id, limit, show } => {
            let antenna_id = id.unwrap_or(config.antenna_id);
            catch(antenna_id, limit, show, &client, &token)?;
        }
        SubCommands::Show => {
            let json = fs::read_to_string("./.reantenna/latest.json")?;
            let data: Vec<NoteData> = serde_json::from_str(&json)?;
            show(&data)?;
        }
    }

    Ok(())
}

fn load_env(path: &str) -> Result<String, Box<dyn Error>> {
    let line = fs::read_to_string(path)?.trim().to_string();
    if let Some(token) = line.strip_prefix("TOKEN=") {
        if !token.is_empty() {
            return Ok(token.to_string());
        }
    }
    Err("Invalid .env format: expected 'TOKEN=...'" .into())
}

fn load_config(path: &str) -> Result<ConfigToml, Box<dyn Error>> {
    let toml_data = fs::read_to_string(path)?;
    let config: ConfigToml = toml::from_str(&toml_data)?;
    Ok(config)
}

fn catch(id: String, limit: u32, show_flag: bool, client: &Client, token: &str) -> Result<(), Box<dyn Error>> {
    let bearer_token = format!("Bearer {}", token);

    let post_data = PostData {
        antenna_id: id,
        limit,
        since_date: 0,
        until_date: 0,
    };

    let response = client
        .post("https://misskey.io/api/antennas/notes")
        .header(AUTHORIZATION, bearer_token)
        .json(&post_data)
        .send()?;

    println!("{}", response.status());

    let response_body = response.text()?;
    let data: Vec<NoteData> = serde_json::from_str(&response_body)?;
    fs::write("./.reantenna/latest.json", serde_json::to_string_pretty(&data)?)?;

    if show_flag {
        show(&data)?;
    }

    Ok(())
}

fn show(data: &[NoteData]) -> Result<(), Box<dyn Error>> {
    for note in data {
        println!(
            "{}@{}",
            note.user.id.cyan().bold(),
            note.user.host.as_deref().unwrap_or("misskey.io")
        );
        println!("{}", note.text);
        println!("----");
    }

    Ok(())
}
