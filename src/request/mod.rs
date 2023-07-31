use crate::app::state::Item;
use reqwest::Result;
use std::time::SystemTime;
use log::{error, info};
use hmac_sha1_compact::HMAC;
use std::collections::HashMap;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue, CONTENT_TYPE};
use lazy_static::lazy_static;
mod verify;
lazy_static!{
    //try to load the user name and password from the access.toml

    //TODO: cusomize the path of the access.toml
    static ref USER_CONFIG:verify::AccessKey = verify::AccessKey::new("access.toml");
}


fn extract_path_and_host_from_url(url:&str) -> (String, String) {
    let mut url = url;
    if url.starts_with("http://") {
        url = &url[7..];
    } else if url.starts_with("https://") {
        url = &url[8..];
    }
    let mut iter = url.split("/");
    let host = iter.next().unwrap_or("");
    let path = iter.collect::<Vec<&str>>().join("/");
    return (host.to_string(), path);
}

fn prepare(method:&str, path:&str, account:&str, passwd:&str) ->HeaderMap{
    let mut res = HeaderMap::new();
    let time_stamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("failed to get time stamp")
        .as_secs();
    let token = format!("{},{},{},{}", 
                    time_stamp, account, method, path);
    let hmac = HMAC::mac(token.as_bytes(), passwd.as_bytes());
    let sign = hex::encode(hmac);
    res.insert(HeaderName::from_static("auth-token"),
       HeaderValue::from_str( &format!("{},{},{}",time_stamp, account, sign)).unwrap());
    return res;
}

pub async fn get_item_list(url:&str) -> Result<Vec<Item>> {
    info!("🛜 sending request to {}", url);
    let (_,path) = extract_path_and_host_from_url(url);
    let account = USER_CONFIG.account.as_str();
    let passwd = USER_CONFIG.key.as_str();
    let header = prepare("GET", &path,account, passwd);
    let client = reqwest::Client::new();
    let res = client.get(url)
        .headers(header)
        .send()
        .await?;
    let body = res.text().await?;
    info!("body:{:?}", body);
    let items: Vec<Item> = serde_json::from_str(&body).unwrap();
    return Ok(items);
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
