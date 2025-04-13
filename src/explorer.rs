use serde::Deserialize;
use std::{
    collections::HashMap,
    fs,
    io::{self, BufReader, Read},
    path::PathBuf,
};
use walkdir::{DirEntry, WalkDir};

/// Cargo.toml structure
#[derive(Deserialize, Debug)]
struct CargoToml {
    package: Option<CargoPackage>,
    dependencies: Option<HashMap<String, toml::Value>>,
    #[serde(rename = "dev-dependencies")]
    dev_dependencies: Option<HashMap<String, toml::Value>>,
}

#[derive(Deserialize, Debug)]
struct CargoPackage {
    name: String,
    version: String,
    description: Option<String>,
}

struct FileInformation {
    structs: HashMap<String, Vec<String>>,
    function: HashMap<String, Vec<String>>,
    variables: HashMap<String, Vec<String>>,
}

struct RepoCodeContext {
    repo_name: String,
    languages: HashMap<String, usize>,
    structure: Vec<FileInformation>,
    config_files: Vec<PathBuf>,
    dependencies: Vec<CargoToml>,
}

/// Implementation for parsing Cargo.toml

impl CargoToml {
    fn parse(file: fs::File) -> Self {
        let mut reader = BufReader::new(file);
        let mut contents = String::new();

        if let Err(e) = reader.read_to_string(&mut contents) {
            eprintln!("Error reading file: {}", e);
            return Self {
                package: None,
                dependencies: None,
                dev_dependencies: None,
            };
        }

        toml::from_str(&contents).unwrap_or_else(|e| {
            eprintln!("Error parsing Cargo.toml: {}", e);
            Self {
                package: None,
                dependencies: None,
                dev_dependencies: None,
            }
        })
    }
}

impl RepoCodeContext {
    fn new(repo_name: String) -> Self {
        Self {
            repo_name,
            languages: HashMap::new(),
            structure: Vec::new(),
            config_files: Vec::new(),
            dependencies: Vec::new(),
        }
    }
}

/// Helper functions

fn invalid_path(entry: &DirEntry) -> bool {
    let invalid = ["target", "node_modules", "venv", "__pycache__"];
    entry
        .file_name()
        .to_str()
        .map(|s| s.starts_with(".") || invalid.contains(&s))
        .unwrap_or(false)
}

fn is_cargo_toml(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s == "Cargo.toml")
        .unwrap_or(false)
}

fn map_extension_to_language(ext: &str) -> String {
    match ext {
        "rs" => "Rust".to_string(),
        _ => ext.to_string(),
    }
}

/// Main traversal logic

fn walk_repo(dir_path: PathBuf) -> Result<RepoCodeContext, io::Error> {
    let repo_name = dir_path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();
    let mut repo = RepoCodeContext::new(repo_name.clone());

    for entry in WalkDir::new(dir_path).into_iter().filter_map(Result::ok) {
        if entry.file_type().is_file() && !invalid_path(&entry) {
            if let Some(ext) = entry.path().extension().and_then(|s| s.to_str()) {
                let lang = map_extension_to_language(ext);
                *repo.languages.entry(lang.clone()).or_insert(0) += 1;

                if is_cargo_toml(&entry) {
                    let f = fs::File::open(entry.path()).unwrap();
                    let cargo_file = CargoToml::parse(f);
                    repo.dependencies.push(cargo_file);
                }
            }
        }
    }

    Ok(repo)
}
