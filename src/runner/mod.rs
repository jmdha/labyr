mod local;
mod slurm;

use std::{collections::HashMap, process::ExitStatus, time::Duration};

use self::{local::Local, slurm::Slurm};
use crate::{setup::generation::Instance, Args};
use clap::ValueEnum;

#[derive(Debug, Copy, Clone, PartialEq, Default, ValueEnum)]
pub enum RunnerKind {
    #[default]
    Local,
    Slurm,
}

pub trait Runner {
    fn run<'a>(&'a self, instances: Vec<Instance<'a>>) -> Vec<Result>;
}

pub fn generate(args: &Args) -> Box<dyn Runner> {
    match args.runner {
        RunnerKind::Local => Box::new(Local {}),
        RunnerKind::Slurm => Box::new(Slurm {}),
    }
}

#[derive(Debug)]
pub struct Result {
    pub id: usize,
    pub exit_status: ExitStatus,
    pub time: Duration,
}
