#![allow(dead_code)]

use std::path::PathBuf;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Agent {
    pub name: String,
    pub compile: bool,
    pub path_to_exe: Option<PathBuf>,
    // pub scores: Vec<f32>,
}

impl Agent {
    pub fn new(name: String, path_to_exe: Option<PathBuf>) -> Agent {
        Agent {
            name,
            compile: path_to_exe.is_some(),
            path_to_exe,
            // scores: vec![],
        }
    }
}
