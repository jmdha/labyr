mod csv;
mod learn;
mod solve;

use crate::setup::instance::Instance;
use crate::setup::suite::Attribute;
use crate::Result;
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

pub fn eval(out_dir: &PathBuf, instance: &Instance) -> Result<()> {
    fs::create_dir_all(out_dir)
        .map_err(|e| format!("Failed to create output dir with error: {}", e))?;
    csv::collect(out_dir, instance)?;
    learn::collect(out_dir, instance)?;
    solve::collect(out_dir, instance)?;
    Ok(())
}

pub(super) fn pattern_names(attributes: Vec<&Attribute>) -> Vec<&str> {
    let names: HashSet<&str> = attributes
        .iter()
        .flat_map(|a| {
            a.patterns
                .iter()
                .map(|p| p.name.as_str())
                .collect::<Vec<&str>>()
        })
        .collect();
    names.into_iter().collect()
}

pub(super) fn pattern_values<'a>(
    pattern_names: &Vec<&str>,
    attribute: &Attribute,
    content: &'a str,
) -> Vec<String> {
    pattern_names
        .iter()
        .map(|a| match attribute.patterns.iter().find(|p| &p.name == a) {
            Some(p) => match p.pattern.captures(content) {
                Some(m) => m[1].to_owned(),
                None => "".to_owned(),
            },
            None => "".to_owned(),
        })
        .collect()
}
