use serde::Deserialize;
use std::{
    collections::HashMap,
    fs::{self, File},
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
/// We want this format
/// {
///  "functions": [
///    { "name": "create_user", "params": ["name: string"], "returns": "User", "visibility": "public", "doc": "Creates a user" }
///  ],

struct FileInformation {
    file_name: String,
    structs: HashMap<String, Vec<String>>,
    functions: HashMap<String, Vec<String>>,
    variables: HashMap<String, Vec<String>>,
    others : Vec<String>, // e.g. comments
}

struct RepoCodeContext {
    repo_name: String,
    languages: HashMap<String, usize>,
    structure: Vec<FileInformation>,
    folders : Vec<String>,
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

impl FileInformation {
    fn new(file_name: String) -> Self {
        Self {
            file_name : file_name,
            structs : HashMap::new(),
            functions: HashMap::new(),
            variables : HashMap::new(),
            others : Vec::new(),
        }
    }
}

impl RepoCodeContext {
    fn new(repo_name: String) -> Self {
        Self {
            repo_name,
            folders : Vec::new(),
            languages: HashMap::new(),
            structure: Vec::new(),
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

fn parse_rust_file(entry: &DirEntry) -> Option<FileInformation> {
    let file_name = entry.path().file_name()?.to_string_lossy().to_string();
    let src = fs::read_to_string(entry.path()).ok()?;
    let syntax_tree: syn::File = syn::parse_str(&src).ok()?;

    let mut file_info = FileInformation::new(file_name);

    for items in syntax_tree.items { 
        match items {
            syn::Item::Fn(func) => {

            },
            syn::Item::Const(var) => {

            },
            syn::Item::Enum(en) => {},
            syn::Item::Struct(struc) => {},
            syn::Item::Static(var) => {},
            _ => {}
        }
    }

    Some(file_info)
    
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
                let file_name = entry.path().file_name().unwrap().to_string_lossy().to_string();

                *repo.languages.entry(lang.clone()).or_insert(0) += 1;

                // dependency file
                if is_cargo_toml(&entry) {
                    let f = fs::File::open(entry.path()).unwrap();
                    let cargo_file = CargoToml::parse(f);
                    repo.dependencies.push(cargo_file);
                }

                let mut new_file: FileInformation = FileInformation::new(file_name.clone());

            } 
        } else if entry.file_type().is_dir() && !invalid_path(&entry) {
            if let Some(folder_name) = entry.path().file_name().and_then(|s| s.to_str()) {
                repo.folders.push(folder_name.to_string());
            }
        }
    }

    Ok(repo)
}
