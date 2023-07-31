use std::time::Duration;
use serde::Serialize;
use serde::Deserialize;
use crate::request::get_item_list;
use log::error;
use log::info;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Item{
    pub name: String,
    pub size: i64,
}

#[derive(Clone, Debug)]
pub enum AppState {
    Init,
    Initialized {
        duration: Duration,
        counter_sleep: u32,
        counter_tick: u64,
        current_url: String,
        current_index:i32,
        current_items: Vec<Item>,
        last_index:i32,
        show_file:bool,
        frame_start:usize,
        frame_end:usize,
    },
}

impl AppState {
    pub async fn initialized(url:String) -> Self {
        let duration = Duration::from_secs(1);
        let counter_sleep = 0;
        let counter_tick = 0;
        let current_items = get_item_list(&url).await;

        match current_items{
            Ok(items) => {
                let current_index = 0;
                let last_index = 0;
                AppState::Initialized {
                    duration,
                    counter_sleep,
                    counter_tick,
                    current_url: url,
                    current_index,
                    last_index,
                    current_items: items,
                    show_file:false,
                    frame_start:0,
                    frame_end:0, // the frame info should not be placed here
                }
            },
            Err(e) => {
                error!("☹️ failed to init: can not get the item list from url: {}", e);
                AppState::Init
            }
        }
    }

    pub fn is_initialized(&self) -> bool {
        matches!(self, &Self::Initialized { .. })
    }

    pub fn incr_sleep(&mut self) {
        if let Self::Initialized { counter_sleep, .. } = self {
            *counter_sleep += 1;
        }
    }

    pub fn get_index(&self) -> i32{
        if let Self::Initialized { current_index, .. } = self {
            return *current_index;
        }
        return -1;
    }

    pub fn get_last_index(&self) -> i32{
        if let Self::Initialized { last_index, .. } = self {
            return *last_index;
        }
        return -1;
    }

    pub fn set_frame(&mut self, start:usize, end:usize){
        if let Self::Initialized { frame_start, frame_end, .. } = self {
            *frame_start = start;
            *frame_end = end;
        }
    }

    pub fn get_frame(&self) -> (usize, usize){
        if let Self::Initialized { frame_start, frame_end, .. } = self {
            return (*frame_start, *frame_end);
        }
        return (0, 0);
    }

    pub fn rows(&self) -> Vec<Item>{
        if let Self::Initialized { current_items, .. } = self {
            current_items.clone()
        }else{
            Vec::new()
        }
    }

    pub fn incr_tick(&mut self) {
        if let Self::Initialized { counter_tick, .. } = self {
            *counter_tick += 1;
        }
    }

    pub fn count_sleep(&self) -> Option<u32> {
        if let Self::Initialized { counter_sleep, .. } = self {
            Some(*counter_sleep)
        } else {
            None
        }
    }

    pub fn count_tick(&self) -> Option<u64> {
        if let Self::Initialized { counter_tick, .. } = self {
            Some(*counter_tick)
        } else {
            None
        }
    }

    pub fn duration(&self) -> Option<&Duration> {
        if let Self::Initialized { duration, .. } = self {
            Some(duration)
        } else {
            None
        }
    }

    pub fn increment_delay(&mut self) {
        if let Self::Initialized { duration, .. } = self {
            // Set the duration, note that the duration is in 1s..10s
            let secs = (duration.as_secs() + 1).clamp(1, 10);
            *duration = Duration::from_secs(secs);
        }
    }

    pub fn decrement_delay(&mut self) {
        if let Self::Initialized { duration, .. } = self {
            // Set the duration, note that the duration is in 1s..10s
            let secs = (duration.as_secs() - 1).clamp(1, 10);
            *duration = Duration::from_secs(secs);
        }
    }

    pub fn back_to_previours(&mut self) {
        if let Self::Initialized { current_url, .. } = self {
            current_url.find('/').map(|idx| {
                current_url.truncate(idx);
            });
        }
    }

    pub fn step_into(&mut self) {
        info!("👉 step into");
        if let Self::Initialized { 
            current_url,
            current_index,
            current_items,
            show_file,
                ..
        } = self {
            if current_items.len() == 0 {
                return self.back_to_previours();
            } 
            let item = &current_items[*current_index as usize];
            if item.size == -1{
                current_url.push_str(&item.name);
                *current_index = 0;
                *show_file = false;
            }else{
                *show_file = true;
            }
            info!("👉 step into: {}", current_url);
        }
    }

    pub fn move_up(&mut self){
        info!("👆 move up");
        if let Self::Initialized { 
            current_index,
            last_index , 
            current_items,
            frame_start,
            frame_end,
            .. } = self {
            *last_index = *current_index;
            *current_index = (*current_index - 1).clamp(0, current_items.len() as i32 - 1);
            if *current_index < *frame_start as i32{
                *frame_start -= 1;
                *frame_end -= 1;
            }
            info!("new index: {}", current_index);
        }
    }

    pub fn move_down(&mut self){
        info!("👇 move down");
        if let Self::Initialized { 
            current_index,
            last_index , 
            current_items,
            frame_start,
            frame_end,
            .. } = self {
            *last_index = *current_index;
            *current_index = (*current_index + 1).clamp(0, current_items.len() as i32 - 1);
            if *current_index + 1 == *frame_end as i32{
                *frame_start += 1;
                *frame_end += 1;
            }
            info!("new index: {}", current_index);
        }
    }

}

impl Default for AppState {
    fn default() -> Self {
        Self::Init
    }
}
