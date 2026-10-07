mod run;
mod reactor;
mod state;
mod repository;
mod run_fable_suggestions;

pub struct Index(u64);

pub enum SnapUp<Snapshot, Update> {
    Snapshot(Snapshot),
    Update(Update)
}


