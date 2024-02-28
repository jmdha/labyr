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
pub struct Result<'a> {
    pub id: String,
    pub domain: &'a str,
    pub problem: &'a str,
    pub solver: &'a str,
    pub exit_status: ExitStatus,
    pub time: Duration,
    pub attributes: HashMap<String, String>,
}

impl<'a> Result<'a> {
    pub fn extract(
        instance: Instance<'a>,
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
            id: instance.id,
            domain: instance.domain,
            problem: instance.problem,
            solver: instance.solver,
            exit_status: status,
            time,
            attributes,
        }
    }
}
