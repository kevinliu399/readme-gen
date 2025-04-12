use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs,
    io::{self, BufReader, Read},
    path::PathBuf,
};
use walkdir::{DirEntry, WalkDir};

/// Stuff to note when traversing
/// - The different languages, maybe a percentage and see what are the most used
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

#[derive(Serialize, Deserialize, Debug)]
struct DependencyFile {
    language: String,

    #[serde(default)]
    name: String,

    #[serde(default)]
    version: String,

    #[serde(default)]
    scripts: Option<HashMap<String, String>>,

    #[serde(default)]
    description: String,

    #[serde(rename = "dependencies", default)]
    dependencies: Option<HashMap<String, String>>,

    #[serde(rename = "devDependencies", default)]
    dev_dependencies: Option<HashMap<String, String>>,
}

impl DependencyFile {
    fn new(language: String) -> Self {
        Self {
            language: language,
            name: String::new(),
            version: String::new(),
            scripts: None,
            description: String::new(),
            dependencies: None,
            dev_dependencies: None,
        }
    }

    pub fn parse_json(file: fs::File, language: String) -> Self {
        // TODO: change language to typescript buy looking at the dev dependencies
        let mut dep_file: DependencyFile =
            match serde_json::from_reader::<_, DependencyFile>(BufReader::new(file)) {
                Ok(data) => data,
                Err(_) => Self::new(language.clone()),
            };
        dep_file.language = language;
        dep_file
    }

    pub fn parse_toml(file: fs::File, language: String) -> Self {
        let mut reader = BufReader::new(file);
        let mut contents = String::new();

        if let Err(e) = reader.read_to_string(&mut contents) {
            eprintln!("Error reading file: {}", e);
            return Self::new(language);
        }

        let mut dep_file: DependencyFile = match toml::from_str(&contents) {
            Ok(data) => data,
            Err(e) => {
                eprintln!("Error parsing TOML: {}", e);
                Self::new(language.clone())
            }
        };

        dep_file.language = language;
        dep_file
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
fn parse_dep_file(entry: &DirEntry, repo: &mut RepoCodeContext, file_format: String) {
    let f = match fs::File::open(entry.path()) {
        Ok(file) => file,
        Err(e) => {
            eprintln!("Error opening file: {}", e);
            return;
        }
    };
    let dep_file: DependencyFile = match file_format.as_str() {
        "package.json" => DependencyFile::parse_json(f, "javascript".to_string()),
        ".xml" => todo!(),
        ".toml" => DependencyFile::parse_toml(
            f,
            if file_format == "Cargo.toml" {
                "rust".to_string()
            } else {
                "python".to_string()
            },
        ),
        ".mod" => todo!(),
        ".txt" => todo!(),
        _ => todo!(), // Makefile
    };
}