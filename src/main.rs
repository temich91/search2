mod cli;
mod output;
mod searcher;
mod stats;
mod walker;

use anyhow::Result;
use clap::Parser;
use cli::Args;
use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use output::OutputManager;
use rayon::prelude::*;
use searcher::Searcher;
use stats::Stats;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use walker::FileWalker;

fn main() -> Result<()> {
    let args = Args::parse();

    // Set thread pool size
    let threads = args.threads.unwrap_or_else(num_cpus::get);
    rayon::ThreadPoolBuilder::new()
        .num_threads(threads)
        .build_global()
        .ok();

    let start = Instant::now();

    // Collect files
    let walker = FileWalker::new(&args);
    let files = walker.collect_files()?;

    if files.is_empty() {
        eprintln!("{}", "No files found matching criteria.".yellow());
        return Ok(());
    }

    // Progress bar показываем только в --count / --files-only режимах,
    // где вывод результатов не конфликтует с перерисовкой прогресс-бара.
    // В обычном режиме (полный вывод матчей) прогресс-бар не нужен — 
    // пользователь видит результаты по мере поиска.
    let pb: Option<Arc<ProgressBar>> = if args.count || args.files_only {
        let pb = ProgressBar::new(files.len() as u64);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} files ({eta})")?
                .progress_chars("#>-"),
        );
        Some(Arc::new(pb))
    } else {
        None
    };

    // Build searcher
    let searcher = Arc::new(Searcher::new(&args)?);
    let stats = Arc::new(Mutex::new(Stats::new()));
    let output = Arc::new(Mutex::new(OutputManager::new(&args)?));

    // Search in parallel
    files.par_iter().for_each(|path| {
        let searcher = Arc::clone(&searcher);
        let stats = Arc::clone(&stats);
        let output = Arc::clone(&output);

        if let Ok(results) = searcher.search_file(path) {
            let file_size = std::fs::metadata(path).map(|m| m.len()).unwrap_or(0);

            {
                let mut s = stats.lock().unwrap();
                s.files_searched += 1;
                s.bytes_searched += file_size;
                if !results.is_empty() {
                    s.files_matched += 1;
                    s.total_matches += results.iter().map(|r| r.matches.len()).sum::<usize>();
                }
            }

            if !results.is_empty() {
                let mut out = output.lock().unwrap();
                out.print_results(path, &results, &searcher.pattern_str).ok();
            }
        }

        if let Some(ref pb) = pb {
            pb.inc(1);
        }
    });

    if let Some(pb) = pb {
        pb.finish_and_clear();
    }

    // Print stats
    if args.stats {
        let s = stats.lock().unwrap();
        s.print(start.elapsed());
    } else if !args.count && !args.files_only {
        // Краткая сводка в конце в обычном режиме
        let s = stats.lock().unwrap();
        if s.files_matched == 0 {
            eprintln!("{}", "No matches found.".yellow());
        }
    }

    Ok(())
}
