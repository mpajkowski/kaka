use std::{collections::HashMap, fs::File, path::Path};

use serde::Deserialize;

#[derive(Debug)]
pub struct Languages {
    pub languages: HashMap<String, Language>,
}

#[derive(Debug, Deserialize)]
pub struct Language {
    pub extensions: Vec<String>,
    pub treesitter: String,
}

impl Languages {
    pub fn from_yaml(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let file = File::open(path)?;
        let languages = serde_yaml::from_reader(file)?;

        Ok(Self { languages })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse_own_languages_yaml() {
        let file = "../usr.share.kaka/languages.yaml";
        let languages = Languages::from_yaml(file).unwrap();

        println!("Languages: {languages:#?}");
    }
}
