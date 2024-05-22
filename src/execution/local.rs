use crate::misc::logging::ProgressBar;
use crate::setup::instance::{Instance, Run, RunKind};
use anyhow::Result;
use log::{info, trace};
use pretty_duration::pretty_duration;
use std::path::PathBuf;
use std::process::Command;
use std::sync::{mpsc, Arc, Mutex};
use std::thread::{self, sleep};
use std::time::{Duration, Instant};

#[derive(Clone, PartialEq, Eq)]
enum State {
    Unprocessed,
    Processing,
    Processed,
}

pub fn execute(instance: Instance, threads: usize) -> Result<()> {
    let runs = Arc::new(Mutex::new(
        instance
            .runs
            .iter()
            .cloned()
            .zip(
                instance
                    .runs
                    .iter()
                    .map(|run| match run.skip {
                        true => State::Processed,
                        false => State::Unprocessed,
                    })
                    .collect::<Vec<_>>(),
            )
            .collect::<Vec<(Run, State)>>(),
    ));
    let pb = ProgressBar::new(runs.lock().unwrap().len());
    let (tx, rx) = mpsc::channel();
    for n in 0..threads {
        let tx = tx.clone();
        let runs = runs.clone();
        thread::spawn(move || loop {
            let run = {
                let mut runs = runs.lock().unwrap();
                let run = runs
                    .iter()
                    .enumerate()
                    .filter_map(|(i, (run, state))| match state {
                        State::Unprocessed => Some((i, run)),
                        _ => None,
                    })
                    .find(|(_, run)| match run.kind {
                        RunKind::Learner => true,
                        RunKind::Solver {
                            problem_index: _,
                            depends,
                        } => {
                            if let Some(depends) = depends {
                                runs[depends].1 == State::Processed
                            } else {
                                true
                            }
                        }
                    })
                    .map(|(i, run)| (i.to_owned(), run.to_owned()));
                if let Some((i, _)) = run {
                    runs[i].1 = State::Processing;
                }
                run
            };
            if let Some((i, run)) = run {
                let _ = tx.send((n, Some(i)));
                let _ = _execute(&run.dir, &run.exe);
                runs.lock().unwrap()[i].1 = State::Processed;
            } else {
                let _ = tx.send((n, None));
                if runs
                    .lock()
                    .unwrap()
                    .iter()
                    .all(|(_, state)| state != &State::Unprocessed)
                {
                    break;
                } else {
                    sleep(Duration::from_millis(50));
                }
            }
        });
    }
    drop(tx);
    while let Ok((_, i)) = rx.recv() {
        if let Some(_) = i {
            pb.inc();
        }
        let runs = runs.lock().unwrap();
        let msg: String = runs
            .iter()
            .filter_map(|(run, state)| match state {
                State::Processing => Some(match run.kind {
                    RunKind::Learner => format!(
                        "{}.{}",
                        instance.runners[run.runner_index].name,
                        instance.tasks[run.task_index].name
                    ),
                    RunKind::Solver {
                        problem_index,
                        depends: _,
                    } => format!(
                        "{}.{}.{}",
                        instance.runners[run.runner_index].name,
                        instance.tasks[run.task_index].name,
                        instance.tasks[run.task_index].solve[problem_index]
                    ),
                }),
                _ => None,
            })
            .collect::<Vec<String>>()
            .join(", ");
        pb.msg(msg);
    }
    Ok(())
}

fn _execute(dir: &PathBuf, exe: &PathBuf) -> Result<()> {
    let dir_name = dir.file_stem().expect("Could not retrieve name of dir");
    let mut command = Command::new(exe);
    command.current_dir(dir);
    trace!("Running command: {:?}", command);
    let t = Instant::now();
    let _ = command.output().expect("Failed to run command");
    let elapsed = t.elapsed();
    info!(
        "{} - {}",
        dir_name.to_str().expect("Could not convert name to string"),
        pretty_duration(&elapsed, None),
    );
    Ok(())
}
