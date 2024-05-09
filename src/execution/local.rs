use crate::{instance::Instance, register::Register};
use anyhow::Result;
use rayon::iter::{IntoParallelRefIterator, ParallelBridge, ParallelIterator};
use std::{path::PathBuf, process::Command, sync::Mutex, time::Instant};

pub fn execute(register: &mut Register, instances: Vec<Instance>) -> Result<()> {
    println!("executing learners...");
    let register = Mutex::new(register);
    instances
        .par_iter()
        .filter_map(|i| i.learn.as_ref())
        .for_each(|learn| {
            if _execute(&learn.dir, &learn.path).is_ok() {
                let mut register = register.lock().unwrap();
                register.register_learn(learn);
            }
        });
    println!("executing solvers...");
    instances
        .iter()
        .filter_map(|instance| instance.solve.as_ref())
        .flat_map(|s| s)
        .par_bridge()
        .for_each(|solve| {
            if _execute(&solve.dir, &solve.path).is_ok() {
                let mut register = register.lock().unwrap();
                register.register_solve(solve);
            }
        });
    Ok(())
}

pub fn _execute(dir: &PathBuf, exe: &PathBuf) -> Result<()> {
    println!("executing {:?}...", exe);
    let t = Instant::now();
    let out = Command::new(exe).current_dir(dir).output();
    match out {
        Ok(o) => println!(
            "exectuion of {:?} finished: {} - {}s",
            exe,
            o.status,
            t.elapsed().as_secs()
        ),
        Err(e) => println!(
            "execution of {:?} failed: {} - {}s",
            exe,
            e,
            t.elapsed().as_secs()
        ),
    };
    Ok(())
}
