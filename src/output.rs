use crate::cli::Args;
use crate::searcher::FileResult;
use anyhow::Result;
use colored::Colorize;
use regex::Regex;
use std::fs::OpenOptions;
use std::io::{BufWriter, Write};
use std::path::Path;

pub struct OutputManager {
    count_only: bool,
    files_only: bool,
    writer: Option<BufWriter<std::fs::File>>,
}

impl OutputManager {
    pub fn new(args: &Args) -> Result<Self> {
        let writer = if let Some(ref path) = args.output {
            let file = OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(path)?;
            Some(BufWriter::new(file))
        } else {
            None
        };

        Ok(Self {
            count_only: args.count,
            files_only: args.files_only,
            writer,
        })
    }

    pub fn print_results(
        &mut self,
        path: &Path,
        results: &[FileResult],
        pattern: &str,
    ) -> Result<()> {
        let path_str = path.display().to_string();

        if self.files_only {
            if let Some(ref mut w) = self.writer {
                w.write_all(format!("{}\n", path_str).as_bytes())?;
            } else {
                println!("{}", path_str.bold().cyan());
            }
            return Ok(());
        }

        let total_matches: usize = results.iter().map(|r| r.matches.len()).sum();

        if self.count_only {
            if let Some(ref mut w) = self.writer {
                w.write_all(format!("{}: {}\n", path_str, total_matches).as_bytes())?;
            } else {
                println!(
                    "{}: {}",
                    path_str.bold().cyan(),
                    total_matches.to_string().yellow()
                );
            }
            return Ok(());
        }

        // --- Полный вывод ---
        // Заголовок файла
        if let Some(ref mut w) = self.writer {
            w.write_all(format!("=== {} ===\n", path_str).as_bytes())?;
        } else {
            println!("\n{}", format!("=== {} ===", path_str).bold().cyan());
        }

        // Regex для подсветки совпадений
        let highlight_re = Regex::new(&format!("(?i){}", regex::escape(pattern))).ok();

        for file_result in results {
            for (idx, match_line) in file_result.matches.iter().enumerate() {
                // Context before
                if let Some(before) = file_result.context_before.get(idx) {
                    for (j, ctx) in before.iter().enumerate() {
                        let ctx_lineno = match_line.line_number - before.len() + j;
                        if let Some(ref mut w) = self.writer {
                            w.write_all(
                                format!("  {:>6}  {}\n", ctx_lineno, ctx).as_bytes(),
                            )?;
                        } else {
                            println!(
                                "  {}  {}",
                                format!("{:>6}", ctx_lineno).dimmed(),
                                ctx.dimmed()
                            );
                        }
                    }
                }

                // Match line с подсветкой совпадения
                let highlighted = if let Some(ref re) = highlight_re {
                    highlight_matches(&match_line.line, re)
                } else {
                    match_line.line.clone()
                };

                if let Some(ref mut w) = self.writer {
                    w.write_all(
                        format!("  {:>6}  {}\n", match_line.line_number, match_line.line)
                            .as_bytes(),
                    )?;
                } else {
                    println!(
                        "  {}  {}",
                        format!("{:>6}", match_line.line_number).yellow().bold(),
                        highlighted
                    );
                }

                // Context after
                if let Some(after) = file_result.context_after.get(idx) {
                    for (j, ctx) in after.iter().enumerate() {
                        let ctx_lineno = match_line.line_number + 1 + j;
                        if let Some(ref mut w) = self.writer {
                            w.write_all(
                                format!("  {:>6}  {}\n", ctx_lineno, ctx).as_bytes(),
                            )?;
                        } else {
                            println!(
                                "  {}  {}",
                                format!("{:>6}", ctx_lineno).dimmed(),
                                ctx.dimmed()
                            );
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

fn highlight_matches(line: &str, re: &Regex) -> String {
    let mut result = String::new();
    let mut last_end = 0;

    for mat in re.find_iter(line) {
        result.push_str(&line[last_end..mat.start()]);
        result.push_str(&mat.as_str().red().bold().to_string());
        last_end = mat.end();
    }
    result.push_str(&line[last_end..]);
    result
}
