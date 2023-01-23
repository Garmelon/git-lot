use std::{
    collections::{hash_map::Entry, HashMap},
    path::PathBuf,
};

use clap::Parser;
use git_repository::{
    objs::tree::EntryMode,
    traverse::{commit::Sorting, tree::Recorder},
    Commit, ObjectId, Repository,
};
use terminal_size::{Height, Width};
use textplots::{Chart, Plot, Shape};
use time::{format_description::FormatItem, macros::format_description};

const TIME_FORMAT: &[FormatItem<'_>] =
    format_description!("[year]-[month]-[day] [hour]:[minute]:[second] [offset_hour sign:mandatory]:[offset_minute]:[offset_second]");

#[derive(Debug, Parser)]
struct Args {
    /// Path to a git repository.
    #[arg(default_value = ".")]
    repo: PathBuf,
    #[arg(long)]
    width: Option<u32>,
    #[arg(long)]
    height: Option<u32>,
    #[arg(long)]
    topo: bool,
}

fn count_lines(
    repo: &Repository,
    commit: &Commit,
    line_cache: &mut HashMap<ObjectId, Option<usize>>,
) -> anyhow::Result<usize> {
    let mut lines = 0;
    let mut recorder = Recorder::default();
    commit.tree()?.traverse().breadthfirst(&mut recorder)?;
    for entry in recorder.records {
        match line_cache.entry(entry.oid) {
            Entry::Occupied(occupied) => {
                lines += occupied.get().unwrap_or(0);
            }
            Entry::Vacant(vacant) => {
                if matches!(entry.mode, EntryMode::Blob | EntryMode::BlobExecutable) {
                    let object = repo.find_object(entry.oid)?;
                    let data = object.detach().data;
                    let line_count = String::from_utf8(data).ok().map(|s| s.lines().count());
                    vacant.insert(line_count);
                    lines += line_count.unwrap_or(0);
                }
            }
        }
    }
    Ok(lines)
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let (Width(width), Height(height)) =
        terminal_size::terminal_size().unwrap_or((Width(80), Height(24)));
    let width = args.width.unwrap_or(width as u32 - 12) * 2;
    let height = args.height.unwrap_or(height as u32 - 6) * 4;
    let sorting = if args.topo {
        Sorting::Topological
    } else {
        Sorting::ByCommitTimeNewestFirst
    };

    let mut repo = git_repository::discover(args.repo)?;
    repo.object_cache_size(Some(100 * 1024 * 1024));
    let commit = repo.head_commit()?;

    let mut lines = vec![];
    let mut line_cache = HashMap::new();
    for ancestor in commit.ancestors().sorting(sorting).all()? {
        let ancestor = repo.find_object(ancestor.unwrap())?.try_into_commit()?;
        let time = ancestor.time()?.format(TIME_FORMAT);
        let line_count = count_lines(&repo, &ancestor, &mut line_cache)?;
        println!("{} {time} - {line_count}", ancestor.id);
        lines.push(line_count);
    }

    let xmax = lines.len() - 1;
    let ymax = lines.iter().copied().max().unwrap_or(0);
    let lines = lines
        .into_iter()
        .rev()
        .enumerate()
        .map(|(i, l)| (i as f32, l as f32))
        .collect::<Vec<_>>();

    println!();
    let mut chart = Chart::new_with_y_range(width, height, 0_f32, xmax as f32, 0_f32, ymax as f32);
    chart.lineplot(&Shape::Lines(&lines)).nice();

    Ok(())
}
