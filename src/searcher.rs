use crate::cli::Args;
use anyhow::Result;
use memmap2::Mmap;
use regex::{Regex, RegexBuilder};
use std::fs::File;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct MatchLine {
    pub line_number: usize,
    pub line: String,
    pub match_start: usize,
    pub match_end: usize,
}

#[derive(Debug)]
pub struct FileResult {
    pub matches: Vec<MatchLine>,
    pub context_before: Vec<Vec<String>>,
    pub context_after: Vec<Vec<String>>,
}

pub struct Searcher {
    pub pattern_str: String,
    regex: Regex,
    context: usize,
    binary: bool,
    max_matches: usize,
}

impl Searcher {
    pub fn new(args: &Args) -> Result<Self> {
        let pattern_str = if args.regex {
            args.pattern.clone()
        } else {
            regex::escape(&args.pattern)
        };

        let regex = RegexBuilder::new(&pattern_str)
            .case_insensitive(!args.case_sensitive)
            .multi_line(true)
            .build()?;

        Ok(Self {
            pattern_str: args.pattern.clone(),
            regex,
            context: args.context,
            binary: args.binary,
            max_matches: args.max_matches,
        })
    }

    pub fn search_file(&self, path: &Path) -> Result<Vec<FileResult>> {
        let file = File::open(path)?;
        let metadata = file.metadata()?;

        if metadata.len() == 0 {
            return Ok(vec![]);
        }

        // Use mmap for files
        let mmap = unsafe { Mmap::map(&file)? };
        let bytes = &mmap[..];

        // Binary detection
        if !self.binary && is_binary(bytes) {
            return Ok(vec![]);
        }

        // Decode bytes to string
        let content = decode_bytes(bytes);

        let mut results = Vec::new();
        let mut file_result = FileResult {
            matches: Vec::new(),
            context_before: Vec::new(),
            context_after: Vec::new(),
        };

        let lines: Vec<&str> = content.lines().collect();

        for (i, line) in lines.iter().enumerate() {
            if let Some(mat) = self.regex.find(line) {
                let match_line = MatchLine {
                    line_number: i + 1,
                    line: line.to_string(),
                    match_start: mat.start(),
                    match_end: mat.end(),
                };

                // Context before
                let before: Vec<String> = if self.context > 0 {
                    let start = i.saturating_sub(self.context);
                    lines[start..i].iter().map(|s| s.to_string()).collect()
                } else {
                    vec![]
                };

                // Context after
                let after: Vec<String> = if self.context > 0 {
                    let end = (i + 1 + self.context).min(lines.len());
                    lines[i + 1..end].iter().map(|s| s.to_string()).collect()
                } else {
                    vec![]
                };

                file_result.matches.push(match_line);
                file_result.context_before.push(before);
                file_result.context_after.push(after);

                if self.max_matches > 0 && file_result.matches.len() >= self.max_matches {
                    break;
                }
            }
        }

        if !file_result.matches.is_empty() {
            results.push(file_result);
        }

        Ok(results)
    }
}

fn is_binary(bytes: &[u8]) -> bool {
    let check_len = bytes.len().min(8192);
    let sample = &bytes[..check_len];
    // Check for null bytes (strong binary indicator)
    let null_count = sample.iter().filter(|&&b| b == 0).count();
    null_count > 0 && (null_count as f64 / check_len as f64) > 0.001
}

fn decode_bytes(bytes: &[u8]) -> String {
    // Try UTF-8 first
    if let Ok(s) = std::str::from_utf8(bytes) {
        return s.to_string();
    }

    // Try Windows-1251 (common on Russian Windows)
    let (cow, _enc, had_errors) = encoding_rs::WINDOWS_1251.decode(bytes);
    if !had_errors {
        return cow.into_owned();
    }

    // Fallback: UTF-8 with replacement
    String::from_utf8_lossy(bytes).into_owned()
}
