use core::error;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs,
    io::{self, BufRead, BufReader},
    path::PathBuf,
};
use walkdir::{DirEntry, WalkDir};

/// Stuff to note when traversing
/// - The different languages, maybe a percentage and see what are the most used
/// - Dockerfile to see more info
/// - Skip target/, node_modules/, .git/, __pycache__/, venv/
/// - Look for project root files like Cargo.toml, package.json
/// Check for Liscense, Author, Project Structures
/// Find entrypoints depending on the language

// TODO: Add proper error handling
struct RepoCodeContext {
    repo_name: String,
    main_language: String,
    languages: HashMap<String, usize>, // language, file count (loc could be better, tbd)
    descriptions: Vec<String>,
    config_files: Vec<PathBuf>,
    dependencies: Vec<String>, // found in files like Cargo.toml, package.json etc...
}

impl RepoCodeContext {
    fn new(repo_name: String) -> Self {
        Self {
            repo_name: repo_name,
            main_language: String::new(),
            languages: HashMap::new(),
            descriptions: Vec::new(),
            config_files: Vec::new(),
            dependencies: Vec::new(),
        }
    }
}

#[derive(Serialize, Deserialize)]
struct DependencyFile {
    language: String,
    name: String,
    version: String,
    scripts: Vec<String>,
    description: String,
    dependencies: Vec<String>,
    dev_dependencies: Vec<String>,
}

impl DependencyFile {
    fn new(language: String) -> Self {
        Self {
            language: language,
            name: String::new(),
            version: String::new(),
            scripts: Vec::new(),
            description: String::new(),
            dependencies: Vec::new(),
            dev_dependencies: Vec::new(),
        }
    }
}

fn walk_repo(dir_path: PathBuf) -> Result<RepoCodeContext, io::Error> {
    let repo_name = dir_path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();
    let mut repo = RepoCodeContext::new(repo_name);
    let mut count_language_files = 0;

    for entry in WalkDir::new(dir_path).into_iter().filter_map(Result::ok) {
        if entry.file_type().is_file() && !invalid_path(&entry) {
            if let Some(k) = entry.path().extension().and_then(|s| s.to_str()) {
                *repo.languages.entry(k.to_string()).or_insert(0) += 1;

                // Only update main language when we have a highest amount
                if *repo.languages.get(&k.to_string()).unwrap_or(&0) > count_language_files {
                    count_language_files = *repo.languages.get(&k.to_string()).unwrap();
                    repo.main_language = k.to_string();
                }
            }

            if is_dep_file(&entry) {
                // TODO: Parse the file and take the important information
            }
        }
    }

    Ok(repo)
}

fn invalid_path(entry: &DirEntry) -> bool {
    let invalid = ["target", "node_modules", "venv", "__pycache__", ".git"];
    entry
        .file_name()
        .to_str()
        .map(|s| s.starts_with(".") || invalid.contains(&s))
        .unwrap_or(false)
}

// These files contain valuable informations such as how to start the app
// and dependencies for the app
// TODO: Add for Ruby and C# and gradle in the future
fn is_dep_file(entry: &DirEntry) -> bool {
    let dep_file = vec![
        "package.json".to_string(),
        "Cargo.toml".to_string(),
        "requirements.txt".to_string(),
        "pyproject.toml".to_string(),
        "go.mod".to_string(),
        "pom.xml".to_string(),
        "Makefile".to_string(),
    ];
    entry
        .file_name()
        .to_str()
        .map(|s| s.starts_with(".") || dep_file.contains(&s.to_string()))
        .unwrap_or(false)
}

// add function signature
fn parse_dep_file(entry: &DirEntry, repo: &mut RepoCodeContext, file_format: &str) {
    // let f = fs::File::open(entry.path()).unwrap_or(todo!());
    // let mut reader = BufReader::new(f);
    // for line in reader.lines().filter_map(Result::ok) {}
    match file_format {
        ".json" => todo!(),
        ".xml" => todo!(),
        ".toml" => todo!(),
        ".mod" => todo!(),
        ".txt" => todo!(),
        _ => todo!(),
    }
}

fn parse_json_file(f: fs::File) -> DependencyFile {
    // use serde_json
    let mut dep_file = DependencyFile::new("javascript".to_string());

    // TODO: Find dev_dep and update the language to typescript if it's there

    dep_file
}
