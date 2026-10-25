use crate::cli::Args;
use anyhow::Result;
use std::path::PathBuf;
use walkdir::WalkDir;

pub struct FileWalker {
    path: String,
    extensions_include: Vec<String>,
    extensions_exclude: Vec<String>,
    max_filesize: u64,
    hidden: bool,
}

impl FileWalker {
    pub fn new(args: &Args) -> Self {
        let extensions_include = args
            .extensions
            .as_ref()
            .map(|e| {
                e.split(',')
                    .map(|s| s.trim().to_lowercase())
                    .filter(|s| !s.is_empty())
                    .collect()
            })
            .unwrap_or_default();

        let extensions_exclude = args
            .exclude
            .as_ref()
            .map(|e| {
                e.split(',')
                    .map(|s| s.trim().to_lowercase())
                    .filter(|s| !s.is_empty())
                    .collect()
            })
            .unwrap_or_default();

        Self {
            path: args.path.clone(),
            extensions_include,
            extensions_exclude,
            max_filesize: args.max_filesize * 1024 * 1024,
            hidden: args.hidden,
        }
    }

    pub fn collect_files(&self) -> Result<Vec<PathBuf>> {
        let mut files = Vec::new();

        let walker = WalkDir::new(&self.path)
            .follow_links(true)
            .into_iter()
            .filter_entry(|e| {
                // Skip hidden
                if !self.hidden {
                    let name = e.file_name().to_string_lossy();
                    if name.starts_with('.') && e.depth() > 0 {
                        return false;
                    }
                }
                true
            });

        for entry in walker.filter_map(|e| e.ok()) {
            if !entry.file_type().is_file() {
                continue;
            }

            let path = entry.path().to_path_buf();

            // Check file size
            if let Ok(meta) = entry.metadata() {
                if meta.len() > self.max_filesize {
                    continue;
                }
            }

            // Extension filter
            let ext = path
                .extension()
                .map(|e| e.to_string_lossy().to_lowercase())
                .unwrap_or_default();

            if !self.extensions_include.is_empty() && !self.extensions_include.contains(&ext.to_string()) {
                continue;
            }

            if !self.extensions_exclude.is_empty() && self.extensions_exclude.contains(&ext.to_string()) {
                continue;
            }

            files.push(path);
        }

        Ok(files)
    }
}
