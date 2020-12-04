use std::collections::HashSet;
use std::io::Read;

use reqwest;
use reqwest::Url;

use regex::Regex;

fn normalize_url(url: &str, origin: &str) -> String {
    let normalized_url: &str;
    if let Some(pos) = url.find('#') {
        normalized_url = &url[..pos];
    } else {
        normalized_url = url;
    }

    if normalized_url.starts_with("/")
    {
        return format!("{}{}", origin, normalized_url);
    }

    normalized_url.to_string()
}

fn fetch_url(client: &reqwest::blocking::Client, url: &str) -> String {
    let mut res = client.get(url).send().unwrap();
    println!("Status for {}: {}", url, res.status());

    let mut body = String::new();
    res.read_to_string(&mut body).unwrap();
    body
}

fn get_link_from_url(html: &str, origin_url: &str) -> HashSet<String> {
    let re = Regex::new("href *= *\"([a-zA-Z0-9\\(\\)!@:%_.~#?&=/\\+\\-]+)").unwrap();

    re.captures_iter(html)
        .map(|c| c[1].to_string())
        .filter_map(|c| check_url(&c, origin_url))
        .collect::<HashSet<String>>()
}

fn check_url(url: &str, origin: &str) -> Option<String> {
    let normalized_url = normalize_url(url, origin);

    if url.ends_with(".pdf") {
        //todo Download PDF
        return None;
    }

    let parsed_url = Url::parse(&normalized_url);
    let origin_url = Url::parse(origin).unwrap();
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
    let client = reqwest::blocking::Client::new();
    let origin_url = "http://doyko.xyz";
    // todo format with / to avoid double check
    let body = fetch_url(&client, origin_url);

    let mut visited_url = HashSet::new();
    visited_url.insert(origin_url.to_string());

    let mut new_url = get_link_from_url(&body, origin_url)
        .difference(&visited_url)
        .map(|x| x.to_string())
        .collect::<HashSet<String>>();

    while !new_url.is_empty() {
        let found_urls: HashSet<String> = new_url
            .iter()
            .map(|url| {
                let body = fetch_url(&client, url);
                get_link_from_url(&body, origin_url)
            })
            .fold(HashSet::new(), |mut acc, x| {
                acc.extend(x);
                acc
            });

        visited_url.extend(new_url);

        new_url = found_urls
            .difference(&visited_url)
            .map(|x| x.to_string())
            .collect::<HashSet<String>>();
    }

    println!("URLs: {:#?}", visited_url);
}
