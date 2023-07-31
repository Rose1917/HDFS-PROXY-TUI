use toml;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct AccessKey{
    pub account:String,
    pub key:String,
}

impl AccessKey{
    pub fn new(path:&str) -> AccessKey{
       let path_str = std::fs::read_to_string(path).expect("Unable to read config file");
       let access_key = toml::from_str::<AccessKey>(&path_str).unwrap(); 
       return access_key;
    }
}
