use std::collections::HashSet;
use std::env;
use std::error;
use std::fs::{self, File};
use std::io::{self, Read};
use std::path::Path;

use once_cell::sync::Lazy;

use log::{self, LevelFilter};
use log4rs::append::file::FileAppender;
use log4rs::config::{Appender, Config, Root};

use reqwest::{self, Url};

use regex::Regex;

use serde_json::Value;

static RE: Lazy<Regex> =
    Lazy::new(|| Regex::new("href *= *\"([a-zA-Z0-9\\(\\)!@:%_.~#?&=/\\+\\-]+)").unwrap());

struct Target {
    name: String,
    url: String,
}

fn init_log() -> Result<(), Box<dyn error::Error>> {
    let logfile = FileAppender::builder().build("output.log")?;

    let config = Config::builder()
        .appender(Appender::builder().build("logfile", Box::new(logfile)))
        .build(Root::builder().appender("logfile").build(LevelFilter::Info))?;

    log4rs::init_config(config)?;

    Ok(())
}

fn read_targets(file_name: &str) -> Result<Vec<Target>, Box<dyn error::Error>> {
    let json = fs::read_to_string(file_name)?;
    let json: Value = serde_json::from_str(&json)?;

    let targets = json
        .as_object()
        .unwrap()
        .into_iter()
        .map(move |(key, value)| Target {
            name: key.to_string(),
            url: value.as_str().unwrap().to_string(),
        })
        .collect::<Vec<Target>>();

    targets.iter().for_each(|target| {
        let path = format!("pdf/{}", target.name);
        if !Path::new(&path).exists() {
            fs::create_dir(&path).unwrap();
        }
    });

    Ok(targets)
}

fn download_pdf(url: &str, folder: &str) {
    log::info!("PDF found : {}", url);
    let mut res = reqwest::blocking::get(url).unwrap();
    let filename = url.split('/').next_back().unwrap();
    let mut out = File::create(format!("pdf/{}/{}", folder, filename)).unwrap();

    io::copy(&mut res, &mut out).unwrap();
}

fn normalize_url(url: &str, origin: &str) -> String {
    let normalized_url = if let Some(pos) = url.find('#') {
        &url[..pos]
    } else {
        url
    };

    if normalized_url.starts_with("/") {
        return format!("{}{}", origin, normalized_url);
    }

    normalized_url.to_string()
}

fn fetch_url(client: &reqwest::blocking::Client, url: &str) -> Option<String> {
    let mut res = client.get(url).send().ok()?;

    log::info!("Status for {}: {}", url, res.status());

    let mut body = String::new();
    match res.read_to_string(&mut body) {
        Ok(_) => Some(body),
        Err(_) => None,
    }
}

fn get_link_from_url(html: &str, target: &Target, pdf_left: &mut u32) -> HashSet<String> {
    RE.captures_iter(html)
        .map(|c| c[1].to_string())
        .filter_map(|c| check_url(&c, target, pdf_left))
        .collect::<HashSet<String>>()
}

fn check_url(url: &str, target: &Target, pdf_left: &mut u32) -> Option<String> {
    if *pdf_left == 0 {
        return None;
    }

    let normalized_url = normalize_url(url, &target.url);

    if url.ends_with(".pdf") {
        download_pdf(&normalized_url, &target.name);
        *pdf_left -= 1;
        return Some(normalized_url);
    }

    let parsed_url = Url::parse(&normalized_url);
    let origin_url = Url::parse(&target.url).unwrap();
    match parsed_url {
        Ok(parsed_url) => {
            if parsed_url.has_host()
                && parsed_url.host_str().unwrap() == origin_url.host_str().unwrap()
            {
                Some(normalized_url)
            } else {
                None
            }
        }
        Err(_e) => None,
    }
}

fn main() {
    init_log().unwrap();

    if !Path::new("pdf").exists() {
        fs::create_dir("pdf").unwrap();
    }

    let args = env::args().collect::<Vec<String>>();
    let limit = args
        .get(1)
        .unwrap_or(&"50".to_string())
        .parse::<u32>()
        .unwrap();

    let targets = read_targets("target.json");
    if targets.is_err() {
        log::error!("Can't read json target file !");
        return;
    }

    let targets = targets.unwrap();

    let client = reqwest::blocking::Client::new();

    for target in targets {
        let origin_url = format!("{}/", &target.url);

        let body = fetch_url(&client, &origin_url);
        if body.is_none() {
            log::warn!("Can't find url {} for {}", &target.url, target.name);
            continue;
        }

        let body = body.unwrap();

        let mut visited_url = HashSet::new();
        visited_url.insert(origin_url.to_string());

        let mut pdf_left: u32 = limit;

        let mut new_url = get_link_from_url(&body, &target, &mut pdf_left)
            .difference(&visited_url)
            .map(|x| x.to_string())
            .collect::<HashSet<String>>();

        while !new_url.is_empty() {
            let found_urls: HashSet<String> = new_url
                .iter()
                .filter_map(|url| fetch_url(&client, url))
                .map(|html| get_link_from_url(&html, &target, &mut pdf_left))
                .fold(HashSet::new(), |mut acc, x| {
                    acc.extend(x);
                    acc
                });

            if pdf_left == 0 {
                log::info!("PDF limit of {} reached for {}", limit, target.name);
                break;
            }

            visited_url.extend(new_url);

            new_url = found_urls
                .difference(&visited_url)
                .map(|x| x.to_string())
                .collect::<HashSet<String>>();
        }
    }
}
