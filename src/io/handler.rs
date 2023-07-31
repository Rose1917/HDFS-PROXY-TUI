use std::sync::Arc;
use std::time::Duration;

use eyre::Result;
use log::{error, info};

use super::IoEvent;
use crate::app::App;

/// In the IO thread, we handle IO event without blocking the UI thread
pub struct IoAsyncHandler {
    app: Arc<tokio::sync::Mutex<App>>,
}

impl IoAsyncHandler {
    pub fn new(app: Arc<tokio::sync::Mutex<App>>) -> Self {
        Self { app }
    }

    /// We could be async here
    pub async fn handle_io_event(&mut self, io_event: IoEvent) {
        let result = match io_event {
            IoEvent::Initialize(init_url) => self.do_initialize(init_url).await,
            IoEvent::Sleep(duration) => self.do_sleep(duration).await,
            IoEvent::StepIn => self.do_step_in().await,
            IoEvent::StepOut => self.do_step_out().await,
            IoEvent::MoveUp => self.do_move_up().await,
            IoEvent::MoveDown => self.do_move_down().await,
        };

        if let Err(err) = result {
            error!("Oops, something wrong happen: {:?}", err);
        }

        let mut app = self.app.lock().await;
        app.loaded();
    }

    /// We use dummy implementation here, just wait 1s
    async fn do_initialize(&mut self, base_url:String) -> Result<()> {
        info!("🚀 Initialize the application");
        let mut app = self.app.lock().await;
        tokio::time::sleep(Duration::from_secs(1)).await;
        app.initialized(base_url).await; // we could update the app state
        info!("👍 Application initialized");
        Ok(())
    }

    /// Just take a little break
    async fn do_sleep(&mut self, duration: Duration) -> Result<()> {
        info!("😴 Go sleeping for {:?}...", duration);
        tokio::time::sleep(duration).await;
        info!("⏰ Wake up !");
        // Notify the app for having slept
        let mut app = self.app.lock().await;
        app.slept();
        Ok(())
    }

    async fn do_step_in(&mut self) -> Result<()>{
        info!("👉 Step into", );
        Ok(())
    }

    async fn do_step_out(&mut self) -> Result<()>{
        info!("👈 back to previous directory");
        Ok(())
    }

    async fn do_move_up(&mut self) -> Result<()>{
        info!("👆 move up");
        Ok(())
    }

    async fn do_move_down(&mut self) -> Result<()>{
        info!("👇 move down");
        Ok(())
    }
}
