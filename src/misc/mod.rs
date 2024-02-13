pub mod path_set {
    use std::path::PathBuf;

    use glob::glob;
    use log::trace;
    use serde::{Deserialize, Deserializer};

    fn glob_vec(pattern: &str) -> Vec<PathBuf> {
        glob(pattern)
            .unwrap()
            .map(|r| r.unwrap())
            .map(|p| p.canonicalize().unwrap())
            .collect()
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<PathBuf>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: String = String::deserialize(deserializer)?;
        trace!("Globbing {}", &s);
        Ok(glob_vec(&s))
    }
}

pub mod abs_path {
    use std::path::PathBuf;

    use log::trace;
    use serde::{Deserialize, Deserializer};

    pub fn deserialize<'de, D>(deserializer: D) -> Result<PathBuf, D::Error>
    where
        D: Deserializer<'de>,
    {
        let path: PathBuf = PathBuf::deserialize(deserializer)?;
        trace!("Canonicalizing {:?}", path);
        Ok(path.canonicalize().unwrap())
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
