use colored::Colorize;
use std::time::Duration;

pub struct Stats {
    pub files_searched: usize,
    pub files_matched: usize,
    pub total_matches: usize,
    pub bytes_searched: u64,
}

impl Stats {
    pub fn new() -> Self {
        Self {
            files_searched: 0,
            files_matched: 0,
            total_matches: 0,
            bytes_searched: 0,
        }
    }

    pub fn print(&self, elapsed: Duration) {
        let mb = self.bytes_searched as f64 / 1_048_576.0;
        let secs = elapsed.as_secs_f64();
        let throughput = if secs > 0.0 { mb / secs } else { 0.0 };

        println!("\n{}", "─── Search Statistics ───".bold());
        println!("  Files searched : {}", self.files_searched.to_string().yellow());
        println!("  Files matched  : {}", self.files_matched.to_string().green());
        println!("  Total matches  : {}", self.total_matches.to_string().green().bold());
        println!("  Data processed : {:.2} MB", mb);
        println!("  Time elapsed   : {:.3}s", secs);
        println!("  Throughput     : {:.1} MB/s", throughput);
    }
}
