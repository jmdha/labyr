use crate::{
    instance::{Learn, Solve},
    suite::Attribute,
};
use anyhow::Result;
use itertools::Itertools;
use std::{
    collections::{hash_map::Entry, HashMap},
    fs::{self, File},
    io::Write,
    path::PathBuf,
};

#[derive(Debug, Default)]
pub struct Register {
    learn: Vec<LearnResult>,
    solve: Vec<SolveResult>,
}

#[derive(Debug)]
pub struct LearnResult {
    pub task: &'static String,
    pub args: Vec<String>,
    pub exit: Option<usize>,
    pub pars: HashMap<&'static String, String>,
    pub csvs: HashMap<PathBuf, String>,
}

#[derive(Debug)]
pub struct SolveResult {
    pub task: &'static String,
    pub prob: &'static String,
    pub args: Vec<String>,
    pub exit: Option<usize>,
    pub pars: HashMap<&'static String, String>,
    pub csvs: HashMap<PathBuf, String>,
}

impl Register {
    fn parse_exit(dir: &PathBuf) -> Result<usize> {
        Ok(fs::read_to_string(dir.join("exit_code"))?
            .trim()
            .parse::<usize>()?)
    }

    fn parse_log(
        dir: &PathBuf,
        attributes: &'static Vec<Attribute>,
    ) -> Result<HashMap<&'static String, String>> {
        let content = fs::read_to_string(dir.join("log"))?;
        Ok(attributes
            .iter()
            .filter_map(|attribute| match attribute.pattern.captures(&content) {
                Some(m) => Some((&attribute.name, m[1].to_string())),
                None => None,
            })
            .collect())
    }

    fn find_files(dir: &PathBuf, ext: &str) -> Vec<PathBuf> {
        let mut files = vec![];
        for entry in walkdir::WalkDir::new(dir)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let name = entry.file_name().to_string_lossy();
            if name.ends_with(ext) {
                files.push(entry.path().strip_prefix(dir).unwrap().to_path_buf());
            }
        }
        files
    }

    fn collect_csvs(dir: &PathBuf) -> HashMap<PathBuf, String> {
        let csvs = Register::find_files(dir, ".csv");
        csvs.into_iter()
            .map(|path| {
                let content = fs::read_to_string(&dir.join(&path)).unwrap_or("".to_string());
                (path, content)
            })
            .collect()
    }

    pub fn register_learn(&mut self, learn: &Learn) {
        self.learn.push(LearnResult {
            task: &learn.task.name,
            args: learn.args.to_owned(),
            exit: Register::parse_exit(&learn.dir).ok(),
            pars: Register::parse_log(&learn.dir, &learn.attributes).unwrap_or(HashMap::new()),
            csvs: Register::collect_csvs(&learn.dir),
        })
    }
    pub fn register_solve(&mut self, solve: &Solve) {
        self.solve.push(SolveResult {
            task: &solve.task.name,
            prob: &solve.problem,
            args: solve.args.to_owned(),
            exit: Register::parse_exit(&solve.dir).ok(),
            pars: Register::parse_log(&solve.dir, &solve.attributes).unwrap_or(HashMap::new()),
            csvs: Register::collect_csvs(&solve.dir),
        })
    }

    pub fn export(&self, dir: &PathBuf) -> Result<()> {
        fs::create_dir_all(dir)?;
        let _ = self.export_learn(&dir.join("learn.csv"));
        let _ = self.export_solve(&dir.join("solve.csv"));
        let _ = Register::explot_csv(
            &dir.join("learn"),
            &self.learn.iter().map(|l| &l.csvs).collect(),
        );
        let _ = Register::explot_csv(
            &dir.join("solve"),
            &self.solve.iter().map(|l| &l.csvs).collect(),
        );
        Ok(())
    }

    fn arg_header(count: usize) -> Vec<String> {
        (0..count).map(|i| format!("arg_{}", i)).collect()
    }

    fn explot_csv(dir: &PathBuf, csvs: &Vec<&HashMap<PathBuf, String>>) -> Result<()> {
        let mut map: HashMap<PathBuf, String> = HashMap::new();
        for m in csvs.iter() {
            for m in m.iter() {
                match map.entry(m.0.to_owned()) {
                    Entry::Occupied(mut e) => {
                        e.get_mut()
                            .push_str(&m.1.lines().skip(1).collect::<String>());
                    }
                    Entry::Vacant(e) => {
                        e.insert(m.1.to_owned());
                    }
                }
            }
        }
        for m in map.into_iter() {
            let path = dir.join(m.0);
            if let Some(dir) = path.parent() {
                println!("creating dirs {:?}...", dir);
                let _ = fs::create_dir_all(dir);
                println!("writing to {:?}...", path);
                let _ = fs::write(path, m.1);
            }
        }
        Ok(())
    }

    fn export_learn(&self, path: &PathBuf) -> Result<()> {
        if self.learn.is_empty() {
            return Ok(());
        }
        assert!(self
            .learn
            .iter()
            .map(|result| result.args.len())
            .all_equal());
        let arg_count = self.learn[0].args.len();
        let attributes: Vec<&'static String> = self
            .learn
            .iter()
            .flat_map(|result| {
                result
                    .pars
                    .iter()
                    .map(|p| *p.0)
                    .collect::<Vec<&'static String>>()
            })
            .unique()
            .collect();
        let header = vec![
            "domain".to_string(),
            "problem".to_string(),
            "name".to_string(),
            "exit_code".to_string(),
        ]
        .into_iter()
        .chain(Register::arg_header(arg_count).into_iter())
        .chain(attributes.iter().map(|s| s.to_string()))
        .join(",");
        let mut file = File::create(path)?;
        let _ = file.write(format!("{}\n", header).as_bytes());
        for result in self.learn.iter() {
            let line = vec![
                result.task.to_string(),
                result.exit.unwrap_or(404).to_string(),
                result.args.iter().join("_"),
            ]
            .into_iter()
            .chain(result.args.iter().map(|a| a.to_string()))
            .chain(attributes.iter().map(|attribute| {
                result
                    .pars
                    .get(attribute)
                    .unwrap_or(&"".to_owned())
                    .to_string()
            }))
            .join(",");
            let _ = file.write(format!("{}\n", line).as_bytes());
        }
        Ok(())
    }
    fn export_solve(&self, path: &PathBuf) -> Result<()> {
        if self.solve.is_empty() {
            return Ok(());
        }
        assert!(self
            .solve
            .iter()
            .map(|result| result.args.len())
            .all_equal());
        let arg_count = self.solve[0].args.len();
        let attributes: Vec<&'static String> = self
            .solve
            .iter()
            .flat_map(|result| {
                result
                    .pars
                    .iter()
                    .map(|p| *p.0)
                    .collect::<Vec<&'static String>>()
            })
            .unique()
            .collect();
        let header = vec![
            "domain".to_string(),
            "problem".to_string(),
            "name".to_string(),
            "exit_code".to_string(),
        ]
        .into_iter()
        .chain(Register::arg_header(arg_count).into_iter())
        .chain(attributes.iter().map(|s| s.to_string()))
        .join(",");
        let mut file = File::create(path)?;
        let _ = file.write(format!("{}\n", header).as_bytes());
        for result in self.solve.iter() {
            let line = vec![
                result.task.to_string(),
                result.prob.to_string(),
                result.args.iter().join("_"),
                result.exit.unwrap_or(404).to_string(),
            ]
            .into_iter()
            .chain(result.args.iter().map(|a| a.to_string()))
            .chain(attributes.iter().map(|attribute| {
                result
                    .pars
                    .get(attribute)
                    .unwrap_or(&"".to_owned())
                    .to_string()
            }))
            .join(",");
            let _ = file.write(format!("{}\n", line).as_bytes());
        }
        Ok(())
    }
}
