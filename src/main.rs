use clap::Parser;
use git2::{Repository, IndexAddOption};
use rand::RngExt;
use std::time::Instant;

#[derive(Parser, Debug)]
#[command(name = "git-auto-committer")]
#[command(about = "Batch auto-commits for Git history", long_about = None)]
struct Args {
    /// Path to git repository (default: current directory)
    #[arg(short, long, default_value = ".")]
    repo_path: String,

    /// Number of commits per batch
    #[arg(short = 'n', long, default_value = "1000")]
    batch_size: u32,

    /// Commit message prefix
    #[arg(short = 'm', long, default_value = "Auto commit")]
    message_prefix: String,

    /// Delay between batches in seconds
    #[arg(short, long, default_value = "1")]
    batch_delay: u64,

    /// Total commits to make (0 = infinite)
    #[arg(short, long, default_value = "0")]
    total: u32,
}

fn generate_message(prefix: &str) -> String {
    let mut rng = rand::rng();
    let hash: u32 = rng.random();
    format!("{} #{}", prefix, hash)
}

fn make_empty_commit(repo: &Repository, message: &str) -> Result<(), git2::Error> {
    let mut index = repo.index()?;
    index.add_all(["*"], IndexAddOption::DEFAULT, None)?;
    let oid = index.write_tree()?;

    let signature = repo.signature()?;
    let tree = repo.find_tree(oid)?;
    
    let parent = repo.head().ok().and_then(|h| h.peel_to_commit().ok());
    let parents: Vec<&git2::Commit> = parent.iter().collect();

    repo.commit(
        Some("HEAD"),
        &signature,
        &signature,
        message,
        &tree,
        &parents,
    )?;

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    
    let repo = Repository::open(&args.repo_path)?;
    
    let mut total_commits = 0u32;
    let mut batch_count = 0u32;
    
    println!("Starting auto-committer...");
    println!("Batch size: {}", args.batch_size);
    println!("Repository: {}", args.repo_path);
    println!("---");

    loop {
        if args.total > 0 && total_commits >= args.total {
            println!("\nCompleted {} commits total!", total_commits);
            break;
        }

        let batch_start = Instant::now();
        let commits_in_batch = if args.total > 0 {
            std::cmp::min(args.batch_size, args.total - total_commits)
        } else {
            args.batch_size
        };

        for i in 0..commits_in_batch {
            let msg = generate_message(&args.message_prefix);
            match make_empty_commit(&repo, &msg) {
                Ok(_) => {
                    total_commits += 1;
                    if (i + 1) % 100 == 0 {
                        print!(".");
                    }
                }
                Err(e) => {
                    eprintln!("\nError at commit {}: {}", total_commits + 1, e);
                }
            }
        }

        let batch_time = batch_start.elapsed();
        batch_count += 1;
        
        println!("\nBatch {} done: {} commits in {:?} ({:.1} comm/s)", 
            batch_count, 
            commits_in_batch,
            batch_time,
            commits_in_batch as f64 / batch_time.as_secs_f64()
        );

        if args.total == 0 || total_commits < args.total {
            if args.batch_delay > 0 {
                std::thread::sleep(std::time::Duration::from_secs(args.batch_delay));
            }
        }
    }

    println!("\nTotal: {} commits in {} batches", total_commits, batch_count);
    Ok(())
}