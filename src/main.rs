// SPDX-License-Identifier: Apache-2.0
use reqwest::blocking::Client;
use reqwest::header::AUTHORIZATION;
use serde::{Serialize, Deserialize};
use std::error::Error;
use std::fs;
use colored::Colorize;


#[derive(Deserialize, Debug)]
struct User {
    id: String,
    host: Option<String>,
}

#[derive(Deserialize, Debug)]
struct NoteData {  // レスポンスから抽出するデータ
  user: User,
  text: String,
}

#[derive(Serialize)]
struct PostData {  // リクエストするデータ
  antennaId: String,
  limit: u32,
  sinceDate: u32,
  untilDate: u32,
}

fn main() -> Result<(), Box<dyn Error>> {
    let client = Client::new();
    let mut token = String::new();
    let mut antenna_id = String::new();
    
    // パッケージ直下の.envの読み込み
    let env_path = ".env";
    let env_data = fs::read_to_string(env_path)?;
    
    for line in env_data.lines() {
        if line.starts_with("TOKEN=") {
            token = line.trim_start_matches("TOKEN=").trim().to_string();
        }
        if line.starts_with("ANTENNA_ID=") {
            antenna_id = line.trim_start_matches("ANTENNA_ID=").trim().to_string();
        }
    }

    let bearer_token = format!("Bearer {}", token);  // Header用のbearer token
    let url = "https://misskey.io/api/antennas/notes";

    let post_data = PostData {
      antennaId: antenna_id,
      limit: 10,
      sinceDate: 0,
      untilDate: 0,
    };

    let response = client
      .post(url)
      .header(AUTHORIZATION, bearer_token)
      .json(&post_data)
      .send()?;

    println!("{}", response.status());
    let response_body = response.text()?;
    
    let data: Vec<NoteData> = serde_json::from_str(&response_body)?;

    // 出力
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