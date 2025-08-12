use std::path::PathBuf;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Agent {
    pub name: String,
    pub path_to_exe: Option<PathBuf>,
    pub id: u32,
    pub compile: bool,
    pub args: Option<Vec<String>>,
    // pub scores: Vec<f32>,
}

impl Agent {
    pub fn new(
        name: String,
        path_to_exe: Option<PathBuf>,
        id: u32,
        args: Option<Vec<String>>,
    ) -> Agent {
        Agent {
            name,
            compile: path_to_exe.is_some(),
            path_to_exe,
            id, // scores: vec![],
            args,
        }
    }
}
