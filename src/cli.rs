use clap::Parser;

/// fsearch — fast text search across file contents
#[derive(Parser, Debug, Clone)]
#[command(
    name = "fsearch",
    version = "0.1.0",
    about = "Fast text search across file contents in directories",
    long_about = None
)]
pub struct Args {
    /// Search pattern (literal text or regex)
    pub pattern: String,

    /// Directory to search (default: current directory)
    #[arg(default_value = ".")]
    pub path: String,

    /// Treat pattern as regular expression
    #[arg(short = 'r', long = "regex")]
    pub regex: bool,

    /// Case-sensitive search (default: case insensitive)
    #[arg(short = 's', long = "case-sensitive")]
    pub case_sensitive: bool,

    /// File extensions to include, comma-separated (e.g. "txt,rs,md")
    #[arg(short = 'e', long = "extensions")]
    pub extensions: Option<String>,

    /// File extensions to exclude, comma-separated (e.g. "exe,dll")
    #[arg(short = 'x', long = "exclude")]
    pub exclude: Option<String>,

    /// Maximum file size in MB to search (default: 100)
    #[arg(short = 'm', long = "max-filesize", default_value = "100")]
    pub max_filesize: u64,

    /// Number of threads (default: number of CPU cores)
    #[arg(short = 't', long = "threads")]
    pub threads: Option<usize>,

    /// Lines of context around each match
    #[arg(short = 'C', long = "context", default_value = "0")]
    pub context: usize,

    /// Only show match counts per file
    #[arg(short = 'c', long = "count")]
    pub count: bool,

    /// Only print filenames with matches
    #[arg(short = 'f', long = "files-only")]
    pub files_only: bool,

    /// Also search binary files
    #[arg(short = 'b', long = "binary")]
    pub binary: bool,

    /// Include hidden files and directories
    #[arg(long = "hidden")]
    pub hidden: bool,

    /// Output results to file
    #[arg(short = 'o', long = "output")]
    pub output: Option<String>,

    /// Show search statistics at end
    #[arg(long = "stats")]
    pub stats: bool,

    /// Maximum number of matches to show per file (0 = unlimited)
    #[arg(long = "max-matches", default_value = "0")]
    pub max_matches: usize,
}
