extern crate logwatcher;
extern crate regex;
extern crate reqwest;

use std::env;
use logwatcher::LogWatcher;
use regex::Regex;
use std::collections::HashMap;
use std::process::Command;

fn handle_line(line: String) {
    if line.contains("ChatWindow: Character") {
        post_chat(line);
    } else if line.contains("]ConanSandbox: Purge Started at V(") {
        post_purge(line);
    }
}

fn post_chat(line: String) {
    let re = Regex::new(r"ChatWindow: Character (...*)").unwrap();
    let caps = re.captures(&line).unwrap();
    let message = caps[1].to_owned();

    post_discord(discord_simple_message(message));
}

fn post_purge(line: String) {
    let re = Regex::new(r"for Clan (...*), Using Wave (...*)").unwrap();
    let caps = re.captures(&line).unwrap();
    let guild_id = caps[1].to_owned();
    let attackers = caps[2].to_owned();

    let rcon_response = rcon(format!("sql SELECT name FROM guilds WHERE guildId={}", &guild_id).as_ref());

    let re = Regex::new(r"#0 (...*) ").unwrap();
    let guild_name: String;
    match re.captures(&rcon_response) {
        Some(_) => {
            let caps = re.captures(&rcon_response).unwrap();
            guild_name = caps[1].to_owned();
        }
        None => {
            // Not a valid guild id. Is it a character id?
            let rcon_response = rcon(format!("sql SELECT char_name FROM characters WHERE id={}", &guild_id).as_ref());
            let re = Regex::new(r"#0 (...*) ").unwrap();
            match re.captures(&rcon_response) {
                Some(_) => {
                    let caps = re.captures(&rcon_response).unwrap();
                    guild_name = caps[1].to_owned();
                }
                None => {
                    // Give up and just post id
                    guild_name = guild_id
                }
            }
        }
    }

    let message = format!("{} is being purged by {}", guild_name, attackers);

    post_discord(discord_simple_message(message));
}

fn discord_simple_message(message: String) -> HashMap<String, String> {
    let mut map: HashMap<String, String> = HashMap::new();
    map.insert("content".to_string(), message);
    return map;
}

fn post_discord(map: HashMap<String, String>) {
    let discord_webhook = &env::var("DISCORD_WEBHOOK").expect("Expected DISCORD_WEBHOOK environment variable");
    let client = reqwest::Client::new();
    let _res = client.post(discord_webhook)
        .json(&map)
        .send();
}

fn rcon(command: &str) -> String {
    let output = Command::new("mcrcon")
        .arg("-c")
        .arg("-H")
        .arg("127.0.0.1")
        .arg("-P")
        .arg("25575")
        .arg("-p")
        .arg(&env::var("RCON_PASSWORD").expect("Expected RCON_PASSWORD environment variable"))
        .arg(command)
        .output()
        .expect("failed to execute process");

    return String::from_utf8_lossy(&output.stdout).to_string();
}

fn main() {
    let args: Vec<_> = env::args().collect();

    let filename: String;

    if args.len() > 1 {
        filename = args[1].to_owned();
    } else {
        println!("{}", "Need a filename");
        return;
    }

    let _webhook = &env::var("DISCORD_WEBHOOK").expect("Expected DISCORD_WEBHOOK environment variable");
    let _rcon_password = &env::var("RCON_PASSWORD").expect("Expected RCON_PASSWORD environment variable");

    let mut log_watcher = LogWatcher::register(filename).unwrap();
    log_watcher.watch(|line: String| {
        handle_line(line);
    });
}