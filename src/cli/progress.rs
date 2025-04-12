use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;

pub struct ProgressIndicator {
    progress_bar: ProgressBar,
}

impl ProgressIndicator {
    pub fn new(message: &str) -> Self {
        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::default_spinner()
                .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏")
                .template("{spinner:.cyan} {msg}")
                .unwrap(),
        );
        pb.set_message(message.to_string());
        pb.enable_steady_tick(Duration::from_millis(100));
        
        Self { progress_bar: pb }
    }
    
    pub fn update_message(&self, message: &str) {
        self.progress_bar.set_message(message.to_string());
    }
    
    pub fn finish_with_message(&self, message: &str) {
        self.progress_bar.finish_with_message(message.to_string());
    }
    
    pub fn finish(&self) {
        self.progress_bar.finish();
    }
}

impl Drop for ProgressIndicator {
    fn drop(&mut self) {
        if !self.progress_bar.is_finished() {
            self.progress_bar.finish_and_clear();
        }
    }
}
