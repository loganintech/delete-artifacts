use std::collections::HashSet;
use clap::Parser;
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::PathBuf;
use walkdir::WalkDir;

static DIRS_TO_DELETE: [&str; 3] = ["node_modules", "vendor", "target"];

/// Simple program to delete specific build artifact directories
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Starting directory for search
    #[clap(value_parser)]
    start_dir: PathBuf,

    /// Actually commit the deletion
    #[clap(short, long)]
    commit: bool,

    /// Don't create a log file with all the deleted directories.
    #[clap(short, long)]
    skip_log_file: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    do_delete(args)
}

fn do_delete(args: Args) -> Result<(), Box<dyn std::error::Error>> {
    let dirs_to_delete: HashSet<&'static str> = HashSet::from(DIRS_TO_DELETE);
    let mut deleted_dirs: Vec<PathBuf> = Vec::new();

    for entry in WalkDir::new(&args.start_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_dir())
    {
        if dirs_to_delete.contains(&entry.file_name().to_str().unwrap()) {
            let dir_path = entry.path();

            if args.commit {
                println!("Deleting {}", dir_path.display());
                if let Err(e) = fs::remove_dir_all(dir_path) {
                    eprintln!("Error deleting directory: {}", e);
                } else {
                    deleted_dirs.push(dir_path.to_path_buf());
                }
            } else {
                println!("Would delete {}", dir_path.display());
            }
        }
    }

    if args.commit && !args.skip_log_file {
        let mut file = File::create("deleted_dirs_log.txt")?;
        for dir in deleted_dirs {
            writeln!(file, "{}", dir.display())?;
        }
    }

    Ok(())
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::create_dir_all;

    #[test]
    fn test_ignore_delete() -> Result<(), Box<dyn std::error::Error>> {
        let temp_path = PathBuf::from("./test");

        // Create test directories
        let test_dirs = DIRS_TO_DELETE;
        for dir in test_dirs.iter() {
            let dir_path = temp_path.join(dir);
            create_dir_all(&dir_path)?;
            File::create(dir_path.join("test_file.txt"))?;
        }

        // Create an additional directory that should not be deleted
        let dir_path = temp_path.join("should_remain");
        create_dir_all(&dir_path)?;
        File::create(dir_path.join("test_file.txt"))?;

        // Run the program in dry run mode
        let args = Args {
            start_dir: temp_path.to_path_buf(),
            commit: false,
            skip_log_file: true,
        };
        main_with_args(args)?;

        // Assert the directories still exist (since it's a dry run)
        for dir in test_dirs.iter() {
            assert!(temp_path.join(dir).exists());
        }
        assert!(temp_path.join("should_remain").exists());

        Ok(())
    }

    #[test]
    fn test_delete() -> Result<(), Box<dyn std::error::Error>> {
        let temp_path = PathBuf::from("./test");

        // Create test directories
        let test_dirs = DIRS_TO_DELETE;
        for dir in test_dirs.iter() {
            let dir_path = temp_path.join(dir);
            create_dir_all(&dir_path)?;
            File::create(dir_path.join("test_file.txt"))?;
        }

        // Create an additional directory that should not be deleted
        let dir_path = temp_path.join("should_remain");
        create_dir_all(&dir_path)?;
        File::create(dir_path.join("test_file.txt"))?;

        // Run the program in dry run mode
        let args = Args {
            start_dir: temp_path.to_path_buf(),
            commit: true,
            skip_log_file: false,
        };
        main_with_args(args)?;

        // Assert the directories still exist (since it's a dry run)
        for dir in test_dirs.iter() {
            assert!(!temp_path.join(dir).exists());
        }
        assert!(temp_path.join("should_remain").exists());

        Ok(())
    }

    // Helper function to run the program with given arguments
    fn main_with_args(args: Args) -> Result<(), Box<dyn std::error::Error>> {
        do_delete(args).map_err(|e| e)
    }
}
