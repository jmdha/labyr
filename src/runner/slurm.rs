use super::{Result, Runner};
use crate::setup::generation::Instance;

pub struct Slurm {}

impl Runner for Slurm {
    fn run<'a>(&'a self, instances: Vec<Instance<'a>>) -> Vec<Result> {
        todo!()
    }
}
