use std::time::Duration;

pub mod handler;
// For this dummy application we only need two IO event
#[derive(Debug, Clone)]
pub enum IoEvent {
    Initialize(String),      // Launch to initialize the application
    Sleep(Duration), // Just take a little break
    StepIn,  // Step in a directory or File
    StepOut,  // Go back to the previous director
    MoveUp,          // Move up to the parent directory
    MoveDown,        // Move down to the child directory
}
