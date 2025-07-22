use std::{
    env,
    fs::{self, File},
    io::{self, BufRead},
    path::{Path, PathBuf},
};

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: {} <md_dir> <rs_dir>", args[0]);
        std::process::exit(1);
    }

    let md_root = PathBuf::from(&args[1]);
    let rs_root = PathBuf::from(&args[2]);

    if !md_root.is_dir() {
        eprintln!("Error: '{}' is not a directory", md_root.display());
        std::process::exit(1);
    }

    if !rs_root.is_dir() {
        eprintln!("Error: '{}' is not a directory", rs_root.display());
        std::process::exit(1);
    }

    let mut total_md_bytes = 0u64;
    let mut total_md_lines = 0u64;
    let mut total_rs_bytes = 0u64;
    let mut total_rs_lines = 0u64;

    let mut unmatched_rs_bytes = 0u64;
    let mut unmatched_rs_lines = 0u64;

    let mut matched_md_bytes = 0u64;
    let mut matched_md_lines = 0u64;
    let mut matched_rs_bytes = 0u64;
    let mut matched_rs_lines = 0u64;

    let mut matched_pairs = Vec::new();

    walk_md_dir(&md_root, &rs_root, &mut |md_path, rs_path| {
        let (bytes, lines) = file_stats(md_path).unwrap_or((0, 0));
        total_md_bytes += bytes;
        total_md_lines += lines;
        matched_md_bytes += bytes;
        matched_md_lines += lines;

        let (bytes, lines) = file_stats(rs_path).unwrap_or((0, 0));
        total_rs_bytes += bytes;
        total_rs_lines += lines;
        matched_rs_bytes += bytes;
        matched_rs_lines += lines;

        matched_pairs.push((md_path.clone(), rs_path.clone(), bytes, lines));
    });

    walk_rs_dir(&rs_root, &md_root, &mut |rs_path| {
        let (bytes, lines) = file_stats(rs_path).unwrap_or((0, 0));
        total_rs_bytes += bytes;
        total_rs_lines += lines;
        unmatched_rs_bytes += bytes;
        unmatched_rs_lines += lines;
    });

    println!("\n=== Overall Statistics ===");
    println!("Total .md files: {} bytes, {} lines", total_md_bytes, total_md_lines);
    println!("Total .rs files: {} bytes, {} lines", total_rs_bytes, total_rs_lines);

    println!("\n=== Unmatched .rs Files ===");
    println!("Unmatched .rs files: {} bytes, {} lines", unmatched_rs_bytes, unmatched_rs_lines);

    println!("\n=== Matched File Pairs ===");
    for (md_path, rs_path, rs_bytes, rs_lines) in matched_pairs {
        let (md_bytes, md_lines) = file_stats(&md_path).unwrap_or((0, 0));
        println!(
            "{} ({}, {}) <-> {} ({}, {})",
            md_path.display(),
            md_bytes,
            md_lines,
            rs_path.display(),
            rs_bytes,
            rs_lines
        );
    }
}

fn walk_md_dir<F>(md_root: &Path, rs_root: &Path, callback: &mut F)
where
    F: FnMut(&Path, &Path),
{
    let walker = walkdir::WalkDir::new(md_root);
    for entry in walker {
        match entry {
            Ok(entry) => {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("md") {
                    let relative = path.strip_prefix(md_root).unwrap();
                    let mut rs_path = rs_root.to_path_buf();
                    rs_path.push(relative);
                    rs_path.set_extension("rs");
                    if rs_path.exists() {
                        callback(path, &rs_path);
                    }
                }
            }
            Err(e) => eprintln!("Error walking md directory: {}", e),
        }
    }
}

fn walk_rs_dir<F>(rs_root: &Path, md_root: &Path, callback: &mut F)
where
    F: FnMut(&Path),
{
    let walker = walkdir::WalkDir::new(rs_root);
    for entry in walker {
        match entry {
            Ok(entry) => {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("rs") {
                    let relative = path.strip_prefix(rs_root).unwrap();
                    let mut md_path = md_root.to_path_buf();
                    md_path.push(relative);
                    md_path.set_extension("md");
                    if !md_path.exists() {
                        callback(path);
                    }
                }
            }
            Err(e) => eprintln!("Error walking rs directory: {}", e),
        }
    }
}

fn file_stats(path: &Path) -> io::Result<(u64, u64)> {
    let file = File::open(path)?;
    let metadata = file.metadata()?;
    let bytes = metadata.len();

    let mut lines = 0u64;
    let reader = io::BufReader::new(file);
    for line in reader.lines() {
        line?;
        lines += 1;
    }

    Ok((bytes, lines))
}