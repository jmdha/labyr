mod local;

use anyhow::Result;
use clap::ValueEnum;

use crate::{instance::Instance, register::Register};

#[derive(Debug, Copy, Clone, PartialEq, Default, ValueEnum)]
pub enum ExecutionKind {
    #[default]
    Local,
}

pub fn execute(
    register: &mut Register,
    instances: Vec<Instance>,
    kind: ExecutionKind,
) -> Result<()> {
    println!("execution method: {:?}", kind);
    match kind {
        ExecutionKind::Local => local::execute(register, instances),
    }
}
