extern crate logwatcher;
extern crate regex;
extern crate reqwest;

use std::env;
use logwatcher::LogWatcher;
use regex::Regex;
use std::collections::HashMap;

fn handle_line(line: String) {
    if line.contains("ChatWindow: Character") {
        let re = Regex::new(r"ChatWindow: Character (...*)").unwrap();
        let caps = re.captures(&line).unwrap();
        let message = caps[1].to_owned();

        let mut map = HashMap::new();
        map.insert("content", message);

        let discord_webhook = &env::var("DISCORD_WEBHOOK").expect("Expected webhook");
        let client = reqwest::Client::new();
        let _res = client.post(discord_webhook)
            .json(&map)
            .send();
    }
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

    let mut log_watcher = LogWatcher::register(filename).unwrap();
    log_watcher.watch(|line: String| {
        handle_line(line);
    });
}