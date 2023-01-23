use std::path::PathBuf;

use clap::Parser;
use git_repository::{objs::tree::EntryMode, traverse::tree::Recorder, Commit, Repository};

#[derive(Debug, Parser)]
struct Args {
    /// Path to a git repository.
    #[arg(default_value = ".")]
    repo: PathBuf,
}

fn count_lines(repo: &Repository, commit: &Commit) -> anyhow::Result<usize> {
    let mut lines = 0;
    let mut recorder = Recorder::default();
    commit.tree()?.traverse().breadthfirst(&mut recorder)?;
    for entry in recorder.records {
        if matches!(entry.mode, EntryMode::Blob | EntryMode::BlobExecutable) {
            let object = repo.find_object(entry.oid)?;
            let data = object.detach().data;
            let text = String::from_utf8(data)?;
            lines += text.lines().count();
        }
    }
    Ok(lines)
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let repo = git_repository::discover(args.repo)?;
    let commit = repo.head_commit()?;
    for ancestor in commit.ancestors().all()? {
        let ancestor = repo.find_object(ancestor.unwrap())?.try_into_commit()?;
        let lines = count_lines(&repo, &ancestor)?;
        println!("{} {lines}", ancestor.id);
    }
    Ok(())
}
