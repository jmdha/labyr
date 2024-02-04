pub mod path_set {
    use std::path::PathBuf;

    use glob::glob;
    use log::trace;
    use serde::{Deserialize, Deserializer};

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<PathBuf>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: String = String::deserialize(deserializer)?;
        trace!("Globbing {}", &s);
        let files = glob(&s)
            .unwrap()
            .filter_map(|p| {
                let path = match p.is_ok() {
                    true => p.unwrap(),
                    false => return None,
                };
                match path.is_file() {
                    true => Some(path),
                    false => return None,
                }
            })
            .collect();
        Ok(files)
    }
}

pub mod regex_pattern {
    use log::trace;
    use regex::Regex;
    use serde::{Deserialize, Deserializer};

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Regex, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: String = String::deserialize(deserializer)?;
        trace!("Parsing regex: {}", &s);
        let pattern: Regex = Regex::new(&s).expect(&format!("Failed to parse regex: {}", &s));
        Ok(pattern)
    }
}
