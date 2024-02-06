use std::{collections::HashMap, env, error::Error, fs::{create_dir, metadata, File}, process::exit};
use serde::Deserialize;
use cliclack::{intro, select, spinner, outro};
use colored::*;

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct Response {
    homepage: String,
    promos: HashMap<String, String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let url: &str = "https://files.minecraftforge.net/net/minecraftforge/forge/promotions_slim.json";
    let response = reqwest::get(url).await?.json::<Response>().await?;
    let mut ord: Vec<&String> = Vec::new();
    let dir: String = "/opt/minecraft".to_string();
    let mut version: &str = "";
    let mut d_url: String = "".to_string();
    let mut spinner = spinner();

    match  env::var("USER") {
        Ok(user) => {
            match user.as_str() {
                "root" => {},
                _ => {
                    eprintln!("[{}] Execute the program as root", "ERROR".red());
                    exit(1);
                }
            }
        }
        Err(_) => {
            eprintln!("[{}] Failed to obtain USER env", "ERROR".red());
            exit(1);
        }
    }

    let _ = intro("Forge server install");

    let tipo = select("Choose a version type")
        .item("recommended", "Recommended", "")
        .item("latest", "Latest", "")
        .interact()?;

    for (version, _id) in &response.promos {
        if version.contains(&tipo) {
            ord.push(version);
        }
    }

    ord.sort_by_cached_key(|s| {
        let trimmed = s.trim_end_matches("-recommended").trim_end_matches("-latest");
        let nums: Vec<u16> = trimmed.split(|c| c == '.')
            .map(|part| part.parse::<u16>())
            .collect::<Result<Vec<u16>, _>>()
            .unwrap_or_else(|_| Vec::new());
        let comb_nums: u16 = nums.iter().fold(0, |acc, &num| (acc * 100) + num as u16);

        comb_nums
    });

    let mut items = Vec::new();
    for v in ord {
        items.push((v.as_str(), v, ""));
    }

    match tipo.trim().to_lowercase().as_str() {
        "recommended" | "latest" => {
            version = select("Choose a version")
                .items(&items)
                .interact()?;
        },
        _ => {}
    }

    match metadata(&dir) {
        Ok(_) => {},
        Err(_) => {
            match create_dir(&dir) {
                Ok(_) => {},
                Err(_) => {
                    eprintln!("[{}] Failed to create directory", "ERROR".red());
                    exit(1);
                }
            }
        }
    }

    let v_num = version.trim_end_matches("-recommended").trim_end_matches("-latest");

    for (v, id) in &response.promos {
        if v == version {
            d_url = format!(
                "https://maven.minecraftforge.net/net/minecraftforge/forge/{}-{}/forge-{}-{}-installer.jar",
                v_num, id, v_num, id
            )
        }
    }

    spinner.start("Downloading...");

    let path: String = format!("{}/forge-{}-installer.jar", dir, v_num);
    let mut dld = reqwest::blocking::get(d_url.clone()).expect(format!("[{}] Request failed", "ERROR".red()).as_str());
    let mut out = File::create(path).expect(format!("[{}] Failed to create file", "ERROR".red()).as_str());
    let _ = dld.copy_to(&mut out);

    spinner.stop("Downloaded successfully");

    let _ = outro("Finalished successfully");
    
    Ok(())
}
