use std::{
    fs::File,
    hash::Hash,
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
};

#[derive(Debug)]
pub struct Agent {
    pub name: String,
    pub path_to_exe: Option<PathBuf>,
    pub path_to_log_dir: Option<PathBuf>,
    pub match_number: AtomicUsize,
    pub id: u32,
    pub compile: bool,
    pub args: Option<Vec<String>>,
    pub error_message: Option<String>,
    // pub scores: Vec<f32>,
}

impl PartialEq for Agent {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Eq for Agent {}

impl Hash for Agent {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        self.id.hash(state);
        self.args.hash(state);
    }
}

impl Agent {
    pub fn new(
        name: String,
        path_to_exe: Option<PathBuf>,
        path_to_log_dir: Option<PathBuf>,
        id: u32,
        args: Option<Vec<String>>,
    ) -> Agent {
        Agent {
            name,
            compile: path_to_exe.is_some(),
            path_to_exe,
            path_to_log_dir,
            match_number: AtomicUsize::new(1),
            id, // scores: vec![],
            args,
            error_message: None,
        }
    }

    pub fn with_error(name: String, id: u32, msg: String) -> Agent {
        Agent {
            name,
            path_to_exe: None,
            path_to_log_dir: None,
            match_number: AtomicUsize::new(1),
            id,
            compile: false,
            args: None,
            error_message: Some(msg),
        }
    }

    pub fn create_new_match_log_file(&self) -> File {
        let dir_path = self
            .path_to_log_dir
            .as_ref()
            .expect("agent has no log directory. Cannot create match log file");

        let id = self.match_number.fetch_add(1, Ordering::Relaxed);

        let path = dir_path.join(format!("match_{id}.txt"));

        File::create_new(&path).expect(&format!("file {} already exists", path.display()))
    }

    pub fn should_be_logged(&self) -> bool {
        self.path_to_log_dir.is_some()
    }
}
