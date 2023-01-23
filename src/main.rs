use std::path::PathBuf;

use clap::Parser;
use git_repository::{traverse::tree::Recorder, Commit};

#[derive(Debug, Parser)]
struct Args {
    /// Path to a git repository.
    #[arg(default_value = ".")]
    repo: PathBuf,
}

struct TreeVisitor;

impl git_repository::traverse::tree::Visit for TreeVisitor {
    fn pop_front_tracked_path_and_set_current(&mut self) {
        println!("pop_front_tracked_path_and_set_current");
    }

    fn push_back_tracked_path_component(&mut self, component: &git_repository::bstr::BStr) {
        println!("push_back_tracked_path_component {component:?}");
    }

    fn push_path_component(&mut self, component: &git_repository::bstr::BStr) {
        println!("push_path_component {component:?}");
    }

    fn pop_path_component(&mut self) {
        println!("pop_path_component");
    }

    fn visit_tree(
        &mut self,
        entry: &git_repository::objs::tree::EntryRef<'_>,
    ) -> git_repository::traverse::tree::visit::Action {
        println!("visit_tree {entry:?}");
        git_repository::traverse::tree::visit::Action::Continue
    }

    fn visit_nontree(
        &mut self,
        entry: &git_repository::objs::tree::EntryRef<'_>,
    ) -> git_repository::traverse::tree::visit::Action {
        println!("visit_nontree {entry:?}");
        git_repository::traverse::tree::visit::Action::Continue
    }
}

fn print_commit(commit: &Commit) -> anyhow::Result<()> {
    println!("{commit:?}");
    println!("{:?}", commit.message()?);
    let mut recorder = Recorder::default();
    commit.tree()?.traverse().breadthfirst(&mut recorder)?;
    println!("{recorder:#?}");
    println!();
    Ok(())
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let repo = git_repository::discover(args.repo)?;
    let commit = repo.head_commit()?;
    for ancestor in commit.ancestors().all()? {
        let ancestor = repo.find_object(ancestor.unwrap())?.try_into_commit()?;
        print_commit(&ancestor)?;
    }
    Ok(())
}
