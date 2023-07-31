use crate::app::state::Item;
use reqwest::Result;
use log::{error, info};
pub async fn get_item_list(url:&str) -> Result<Vec<Item>> {
    info!("🛜 sending request to {}", url);
    let res = reqwest::get(url).await?;
    let body = res.text().await?;
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
