use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <foo>", args[0]);
        std::process::exit(1);
    }
    let foo = &args[1];

    let instruct_dir = Path::new("instruct/bin");
    let src_dir = Path::new("src/bin");

    let mut max_num: Option<u32> = None;

    // Scan instruct/bin/ for llm-foo-<number>.md
    if let Ok(entries) = fs::read_dir(instruct_dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().into_string().unwrap_or_default();
            if let Some(rest) = name.strip_prefix(&format!("llm-{}-", foo)) {
                if let Some(num_str) = rest.strip_suffix(".md") {
                    if let Ok(num) = num_str.parse::<u32>() {
                        max_num = Some(max_num.map_or(num, |m| m.max(num)));
                    }
                }
            }
        }
    }

    // Scan src/bin/ for llm-foo-<number>.rs
    if let Ok(entries) = fs::read_dir(src_dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().into_string().unwrap_or_default();
            if let Some(rest) = name.strip_prefix(&format!("llm-{}-", foo)) {
                if let Some(num_str) = rest.strip_suffix(".rs") {
                    if let Ok(num) = num_str.parse::<u32>() {
                        max_num = Some(max_num.map_or(num, |m| m.max(num)));
                    }
                }
            }
        }
    }

    let current_num = match max_num {
        Some(n) => n,
        None => {
            eprintln!("No matching files found for llm-{}-<number>.md or llm-{}-<number>.rs", foo, foo);
            std::process::exit(1);
        }
    };
    let next_num = current_num + 1;

    let md_src = instruct_dir.join(format!("llm-{}-{}.md", foo, current_num));
    let md_dst = instruct_dir.join(format!("llm-{}-{}.md", foo, next_num));

    let rs_src = src_dir.join(format!("llm-{}-{}.rs", foo, current_num));
    let rs_dst = src_dir.join(format!("llm-{}-{}.rs", foo, next_num));

    if !md_src.exists() {
        eprintln!("Source file not found: {}", md_src.display());
        std::process::exit(1);
    }
    if !rs_src.exists() {
        eprintln!("Source file not found: {}", rs_src.display());
        std::process::exit(1);
    }

    if let Err(e) = fs::copy(&md_src, &md_dst) {
        eprintln!("Failed to copy {} to {}: {}", md_src.display(), md_dst.display(), e);
        std::process::exit(1);
    }
    println!("Copied {} to {}", md_src.display(), md_dst.display());

    if let Err(e) = fs::copy(&rs_src, &rs_dst) {
        eprintln!("Failed to copy {} to {}: {}", rs_src.display(), rs_dst.display(), e);
        std::process::exit(1);
    }
    println!("Copied {} to {}", rs_src.display(), rs_dst.display());

    let output = Command::new("git")
        .arg("add")
        .arg(&md_dst)
        .arg(&rs_dst)
        .output()
        .expect("Failed to execute git add");

    if !output.status.success() {
        eprintln!("git add failed: {}", String::from_utf8_lossy(&output.stderr));
        std::process::exit(1);
    }

    let commit_msg = format!("first commit for llm-{}-{}", foo, next_num);
    let output = Command::new("git")
        .arg("commit")
        .arg("-m")
        .arg(&commit_msg)
        .output()
        .expect("Failed to execute git commit");

    if !output.status.success() {
        eprintln!("git commit failed: {}", String::from_utf8_lossy(&output.stderr));
        std::process::exit(1);
    }

    println!("Committed: {}", commit_msg);
}