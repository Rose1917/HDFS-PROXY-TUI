use std::time::Duration;
use serde::Serialize;
use serde::Deserialize;
use crate::request::get_item_list;
use crate::request::get_file_chunk;
use log::error;
use log::info;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Item{
    pub name: String,
    pub size: i64,
}

pub enum ContentState{
    ItemList,
    FileChunk,
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
        current_items: Option<Vec<Item>>,
        last_index:i32,
        show_file:bool,
        frame_start:usize,
        frame_end:usize,
        file_chunk:Option<String>,
    },
}

impl AppState {
    pub async fn initialized(url:String) -> Self {
        let duration = Duration::from_secs(1);
        let counter_sleep = 0;
        let counter_tick = 0;
        if url.ends_with('/'){
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
                        current_items: Some(items),
                        show_file:false,
                        frame_start:0,
                        frame_end:0, // the frame info should not be placed here
                        file_chunk:None,
                    }
                },
                Err(e) => {
                    error!("☹️ failed to init: can not get the item list from url: {}", e);
                    AppState::Init
                }
            } 
        }else{
            let file_chunk = get_file_chunk(&url).await;
            match file_chunk{
                Ok(chunk) => {
                    AppState::Initialized {
                        duration,
                        counter_sleep,
                        counter_tick,
                        current_url: url,
                        current_index:0,
                        last_index:0,
                        current_items: None,
                        show_file:true,
                        frame_start:0,
                        frame_end:0, // the frame info should not be placed here
                        file_chunk:Some(chunk),
                    }
                },
                Err(e) => {
                    error!("☹️ failed to init: can not get the file chunk from url: {}", e);
                    AppState::Init
                }
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
        if let Self::Initialized { current_items:Some(current_items), .. } = self {
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

    pub async fn update_state(&mut self, content_type:ContentState){
        if let Self::Initialized { current_url, .. } = self {
            match content_type{
                ContentState::ItemList => {
                    let current_items  = get_item_list(&current_url).await;
                    let new_state = 
                        match current_items{
                            Ok(items) => {
                                let current_index = 0;
                                let last_index = 0;
                                AppState::Initialized {
                                    current_url: current_url.clone(),
                                    current_index,
                                    last_index,
                                    current_items: Some(items),
                                    show_file:false,
                                    frame_start:0,
                                    frame_end:0, // the frame info should not be placed here
                                    file_chunk:None,
                                    duration:Duration::from_secs(1),
                                    counter_sleep:0,
                                    counter_tick:self.count_tick().unwrap_or(0),
                                }
                            },
                            Err(e) => {
                                panic!("☹️ failed to init: can not get the item list from url: {}", e);
                            }
                        };
                    *self = new_state;
                }   
                ContentState::FileChunk => {
                    let file_chunk = get_file_chunk(&current_url).await;
                    let new_state = 
                        match file_chunk{
                            Ok(chunk) => {
                                AppState::Initialized {
                                    current_url: current_url.clone(),
                                    current_index:0,
                                    last_index:0,
                                    current_items: None,
                                    show_file:true,
                                    frame_start:0,
                                    frame_end:0, // the frame info should not be placed here
                                    file_chunk:Some(chunk),
                                    duration:Duration::from_secs(1),
                                    counter_sleep:0,
                                    counter_tick:self.count_tick().unwrap_or(0),
                                }
                            },
                            Err(e) => {
                                panic!("☹️ failed to init: can not get the file chunk from url: {}", e);
                            }
                        };
                    *self = new_state;
                }
            }
        }

    }

    pub async fn back_to_previours(&mut self) {
        info!("👈 back to previours");
        let content_state = self.get_state();
        if let Self::Initialized { current_url, .. } = self {
            match content_state{
                ContentState::FileChunk => {
                    let last_slash_index = current_url.rfind('/').expect("can not find the last slash");
                    *current_url = current_url[..=last_slash_index].to_string();
                }
                ContentState::ItemList => {
                    let remove_last_slash = current_url.trim_end_matches('/');
                    let last_slash_index = remove_last_slash.rfind('/').expect("can not find the last slash");
                    *current_url = current_url[..=last_slash_index].to_string();
                }
            }
            self.update_state(content_state).await;
        }
    }

    pub async fn step_into(&mut self) {
        info!("👉 step into");
        if let Self::Initialized { 
            current_url,
            current_index,
            current_items,
            show_file,
            file_chunk,
                ..
        } = self {
            let item = &current_items.as_ref().unwrap()[*current_index as usize];
            current_url.push_str(&item.name);
            if item.size == -1{
                current_url.push('/');
                let new_current_items = get_item_list(&current_url).await;
                match new_current_items{
                    Ok(items) => {
                        *current_items = Some(items);
                        *current_index = 0;
                        *show_file = false;
                    },
                    Err(e) => {
                        error!("☹️ failed to step into: can not get the item list from url: {}", e);
                    }
                }
            }else{
                let cur_file_chunk = get_file_chunk(&current_url).await;
                match cur_file_chunk{
                    Ok(chunk) => {
                        *current_items = None;
                        info!("👉 file chunk: {}", &chunk);
                        *file_chunk = Some(chunk);
                    },
                    Err(e) => {
                        error!("☹️ failed to step into: can not get the file chunk from url: {}", e);
                    }
                }
                *show_file = true;
            }
            info!("👉 step into: {}", current_url);
            self.reset_frame();
        }
    }

    pub fn reset_frame(&mut self){
        if let Self::Initialized { 
            current_index,
            frame_start,
            frame_end,
            .. } = self {
            *current_index = 0;
            *frame_start = 0;
            *frame_end = 0;
        }
    }

    /// return 0 if it is a directory
    /// return 1 if it is a file
    pub fn get_state(&self) -> ContentState{
        if let Self::Initialized { 
            current_items:Some(_),
            .. } = self {
                return ContentState::ItemList;
        }else if let Self::Initialized { 
            current_items:None,
            .. } = self {
                return ContentState::FileChunk;
        }else{
            panic!("☹️ invalid state");
        }
    }

    pub fn get_file_chunk(&self) -> String{
        if let Self::Initialized { 
            file_chunk:Some(file_chunk),
            .. } = self {
                return file_chunk.clone();
        }else{
            panic!("☹️ invalid state");
        }
    }

    pub fn move_up(&mut self){
        info!("👆 move up");
        if let Self::Initialized { 
            current_index,
            last_index , 
            current_items:Some(current_items),
            frame_start,
            frame_end,
            .. } = self {
            *last_index = *current_index;
            *current_index = (*current_index - 1).clamp(0, current_items.len() as i32 - 1);
            if *current_index + 1 == *frame_end as i32 && current_items.len() > (*frame_end - *frame_start){
                *frame_start -= 1;
                *frame_end -= 1;
            }
            info!("new index: {}", current_index);
        }else if let Self::Initialized { 
            current_index,
            last_index , 
            current_items:None,
            file_chunk:Some(file_chunk),
            .. } = self {
            *last_index = *current_index;
            let lines_count = file_chunk.lines().count();
            *current_index = (*current_index - 1).clamp(0, lines_count as i32 - 1);
            info!("new index: {}", current_index);
        }
    }

    pub fn move_down(&mut self){
        info!("👇 move down");
        if let Self::Initialized { 
            current_index,
            last_index , 
            current_items:Some(current_items),
            file_chunk:None,
            frame_start,
            frame_end,
            .. } = self {
            *last_index = *current_index;
            *current_index = (*current_index + 1).clamp(0, current_items.len() as i32 - 1);
            if *current_index + 1 == *frame_end as i32 && current_items.len() > (*frame_end - *frame_start){
                *frame_start += 1;
                *frame_end += 1;
            }
            info!("new index: {}", current_index);
        }else if let Self::Initialized { 
            current_index,
            last_index , 
            current_items:None,
            file_chunk:Some(file_chunk),
            .. } = self {
            *last_index = *current_index;
            let lines_count = file_chunk.lines().count();
            *current_index = (*current_index + 1).clamp(0, lines_count as i32 - 1);
            info!("new index: {}", current_index);
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::Init
    }
}
