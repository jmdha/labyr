mod local;
mod slurm;

use self::{local::Local, slurm::Slurm};
use crate::{setup::generation::Instance, Args};
use clap::ValueEnum;
use std::{collections::HashMap, process::ExitStatus, time::Duration};

#[derive(Debug, Copy, Clone, PartialEq, Default, ValueEnum)]
pub enum RunnerKind {
    #[default]
    Local,
    Slurm,
}

pub trait Runner {
    fn run<'a>(&'a self, instances: &Vec<Instance<'a>>) -> Vec<Result>;
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
    pub attributes: HashMap<String, String>,
}

impl Result {
    pub fn extract(
        instance: &Instance,
        id: usize,
        status: ExitStatus,
        time: Duration,
        out: String,
    ) -> Self {
        let attributes = match instance.attributes {
            Some(att) => att
                .patterns
                .iter()
                .map(|pattern| {
                    (
                        pattern.name.to_owned(),
                        match pattern.pattern.captures(&out) {
                            Some(c) => c[1].to_string(),
                            None => "".to_string(),
                        },
                    )
                })
                .collect::<HashMap<String, String>>(),
            None => HashMap::new(),
        };
        Self {
            id,
            exit_status: status,
            time,
            attributes,
        }
    }
}
