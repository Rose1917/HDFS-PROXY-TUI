use std::sync::Arc;

use eyre::Result;
use log::LevelFilter;
use hdfs_proxy_tui::app::App;
use hdfs_proxy_tui::io::handler::IoAsyncHandler;
use hdfs_proxy_tui::io::IoEvent;
use hdfs_proxy_tui::start_ui;
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    let (sync_io_tx, mut sync_io_rx) = tokio::sync::mpsc::channel::<IoEvent>(100);
    let args = env::args().collect::<Vec<String>>();
    if args.len() < 2 {
        println!("Usage: {} <base-url>", args[0]);
        println!(r#"
         _         _  __                                   _         _
        | |__   __| |/ _|___   _ __  _ __ _____  ___   _  | |_ _   _(_)
        | '_ \ / _` | |_/ __| | '_ \| '__/ _ \ \/ / | | | | __| | | | |
        | | | | (_| |  _\__ \ | |_) | | | (_) >  <| |_| | | |_| |_| | |
        |_| |_|\__,_|_| |___/ | .__/|_|  \___/_/\_\\__, |  \__|\__,_|_|
                              |_|                  |___/
                              "#);
        std::process::exit(0);
    }
    let base_url = args
        .get(1)
        .expect(&"can not found the base url".to_string())
        .to_string();

    // We need to share the App between thread
    let app = Arc::new(tokio::sync::Mutex::new(App::new(sync_io_tx.clone())));
    let app_ui = Arc::clone(&app);

    // Configure log
    tui_logger::init_logger(LevelFilter::Debug).unwrap();
    tui_logger::set_default_level(log::LevelFilter::Debug);

    // Handle IO in a specifc thread
    tokio::spawn(async move {
        let mut handler = IoAsyncHandler::new(app);
        while let Some(io_event) = sync_io_rx.recv().await {
            handler.handle_io_event(io_event).await;
        }
    });

    start_ui(&app_ui, base_url).await?;
    Ok(())
}
