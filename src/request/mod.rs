use crate::app::state::Item;
use crate::crypto::mac::Mac;
use crypto::hmac::Hmac;
use crypto::sha1::Sha1;
use lazy_static::lazy_static;
use log::{error, info, warn};
use reqwest::header::{HeaderMap, HeaderName, HeaderValue, CONTENT_TYPE};
use reqwest::Result;
use std::collections::HashMap;
use std::time::SystemTime;
mod verify;
lazy_static! {
    //try to load the user name and password from the access.toml

    //TODO: cusomize the path of the access.toml
    static ref USER_CONFIG:verify::AccessKey = verify::AccessKey::new("access.toml");
}

fn extract_path_and_host_from_url(url: &str) -> (String, String) {
    let mut url = url;
    if url.starts_with("http://") {
        url = &url[7..];
    } else if url.starts_with("https://") {
        url = &url[8..];
    }
    let mut iter = url.split("/");
    let host = iter.next().unwrap_or("");
    let path = iter.collect::<Vec<&str>>().join("/");
    return (host.to_string(), "/".to_owned() + &path);
}

fn extract_filename_from_url(url: &str) -> String {
    let slash_index = url.rfind('/').expect("unknown state");
    return url[slash_index..url.len()].to_owned();
}

fn prepare(method: &str, path: &str, account: &str, passwd: &str) -> HeaderMap {
    let mut res = HeaderMap::new();
    let time_stamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("failed to get time stamp")
        .as_secs();
    let token = format!("{},{},{},{}", time_stamp, account, method, path);
    info!("token:{}", token);
    info!("passwd:{}", passwd);
    let mut mac = Hmac::new(Sha1::new(), passwd.as_bytes());
    mac.input(token.as_bytes());
    let result = mac.result();
    let hmac = result.code();
    info!("hmac:{:?}", hmac);
    let sign = hex::encode(hmac);
    res.insert(
        HeaderName::from_static("auth-token"),
        HeaderValue::from_str(&format!("{},{},{}", time_stamp, account, sign)).unwrap(),
    );
    info!("header value:{},{},{}", time_stamp, account, sign);
    return res;
}

pub async fn get_item_list(url: &str) -> Result<Vec<Item>> {
    info!("🛜 sending request to {}", url);
    let (_, path) = extract_path_and_host_from_url(url);
    let account = USER_CONFIG.account.as_str();
    let passwd = USER_CONFIG.key.as_str();
    let header = prepare("GET", &path, account, passwd);
    let client = reqwest::Client::new();
    let res = client.get(url).headers(header).send().await?;
    let status = res.status();
    info!("status:{:?}", status);
    let body = res.text().await?;
    info!("body:{:?}", body);
    let items: Vec<Item> = serde_json::from_str(&body).unwrap();
    return Ok(items);
}

pub async fn get_file_chunk(url: &str) -> Result<String> {
    info!("🛜 sending request to {}", url);
    let (_, path) = extract_path_and_host_from_url(url);
    let account = USER_CONFIG.account.as_str();
    let passwd = USER_CONFIG.key.as_str();
    let header = prepare("GET", &path, account, passwd);
    let client = reqwest::Client::new();
    let res = client.get(url).headers(header).send().await?;
    let status = res.status();
    info!("status:{:?}", status);
    let body = res.text().await?;
    return Ok(body);
}

pub async fn dump_file(url: &str, file_chunk: &Option<String>) -> Result<()> {
    info!("🦁️download file from {}", url);
    if url.ends_with('/') {
        warn!("trying to dump a directory, ignored.");
        return Ok(());
    }

    info!("target url:{}", url);
    let file_name = extract_filename_from_url(url);
    if let Some(file_chunk_str) = file_chunk {
        std::fs::write(&file_name, file_chunk_str)
            .expect(&format!("failed to write to {} from {}", &file_name, url));
        return Ok(());
    }

    let chunk = get_file_chunk(url).await?;
    std::fs::write(&file_name, chunk)
        .expect(&format!("failed to write to {} from {}", &file_name, url));
    return Ok(());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn success_get_item_list() -> Result<()> {
        let list = get_item_list("http:localhost:7878/Users/march1917/").await;
        println!("{:?}", list.unwrap());
        Ok(())
    }
}
