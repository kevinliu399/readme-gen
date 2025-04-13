use quote::ToTokens;
use serde::Deserialize;
use std::{
    collections::HashMap,
    fs::{self, File},
    io::{self, BufReader, Read},
    path::PathBuf,
};
use walkdir::{DirEntry, WalkDir};

#[derive(Debug)]
pub enum RepoError {
    Io(io::Error),
    Toml(toml::de::Error),
    Syn(syn::Error),
}

impl std::fmt::Display for RepoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RepoError::Io(e) => write!(f, "IO Error: {}", e),
            RepoError::Toml(e) => write!(f, "TOML Parse Error: {}", e),
            RepoError::Syn(e) => write!(f, "Syntax Parse Error: {}", e),
        }
    }
}

impl std::error::Error for RepoError {}

impl From<io::Error> for RepoError {
    fn from(error: io::Error) -> Self {
        RepoError::Io(error)
    }
}

impl From<toml::de::Error> for RepoError {
    fn from(error: toml::de::Error) -> Self {
        RepoError::Toml(error)
    }
}

impl From<syn::Error> for RepoError {
    fn from(error: syn::Error) -> Self {
        RepoError::Syn(error)
    }
}

/// Cargo.toml
#[derive(Deserialize, Debug)]
pub struct CargoToml {
    pub package: Option<CargoPackage>,
    pub dependencies: Option<HashMap<String, toml::Value>>,
    #[serde(rename = "dev-dependencies")]
    pub dev_dependencies: Option<HashMap<String, toml::Value>>,
}

#[derive(Deserialize, Debug)]
pub struct CargoPackage {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
}

pub struct FileInformation {
    pub file_name: String,
    pub structs: HashMap<String, Vec<String>>,
    pub functions: HashMap<String, FunctionMeta>,
    pub variables: Vec<String>,
    pub enums: HashMap<String, Vec<String>>,
    pub others: Vec<String>, // e.g. comments
}

pub struct FunctionMeta {
    pub params: Vec<String>,
    pub returns: String,
    pub visibility: String,
}

pub struct RepoCodeContext {
    pub repo_name: String,
    pub languages: HashMap<String, usize>,
    pub files: Vec<FileInformation>,
    pub folders: Vec<String>,
    pub dependencies: Vec<CargoToml>,
}

/// Implementation for parsing Cargo.toml

impl CargoToml {
    /// Now returns a Result rather than silently printing errors.
    pub fn parse(file: File) -> Result<Self, RepoError> {
        let mut reader = BufReader::new(file);
        let mut contents = String::new();
        reader.read_to_string(&mut contents)?;
        let cargo: CargoToml = toml::from_str(&contents)?;
        Ok(cargo)
    }
}

impl FileInformation {
    pub fn new(file_name: String) -> Self {
        Self {
            file_name,
            structs: HashMap::new(),
            functions: HashMap::new(),
            variables: Vec::new(),
            enums: HashMap::new(),
            others: Vec::new(),
        }
    }
}

impl RepoCodeContext {
    fn new(repo_name: String) -> Self {
        Self {
            repo_name,
            folders: Vec::new(),
            languages: HashMap::new(),
            files: Vec::new(),
            dependencies: Vec::new(),
        }
    }
}

impl FunctionMeta {
    pub fn new(params: Vec<String>, visibility: String, returns: String) -> Self {
        Self {
            params,
            visibility,
            returns,
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

/// Parse a Rust source file and collect information.
/// Returns a Result wrapping FileInformation.
fn parse_rust_file(entry: &DirEntry) -> Result<FileInformation, RepoError> {
    let file_name = entry
        .path()
        .file_name()
        .and_then(|s| s.to_str())
        .map(String::from)
        .ok_or_else(|| RepoError::Io(io::Error::new(io::ErrorKind::Other, "Invalid file name")))?;

    let src = fs::read_to_string(entry.path())?;
    let syntax_tree: syn::File = syn::parse_str(&src)?;

    let mut file_info = FileInformation::new(file_name);

    for item in syntax_tree.items {
        match item {
            syn::Item::Fn(func) => {
                let func_name = func.sig.ident.to_string();
                let func_vis = match func.vis {
                    syn::Visibility::Public(_) => "public",
                    _ => "private",
                }
                .to_string();
                let func_output = match func.sig.output {
                    syn::ReturnType::Default => "None".to_string(),
                    syn::ReturnType::Type(_, typ) => typ.to_token_stream().to_string(),
                };
                let mut params: Vec<String> = Vec::new();
                for p in func.sig.inputs.iter() {
                    match p {
                        syn::FnArg::Receiver(_) => params.push("self".to_string()),
                        syn::FnArg::Typed(t) => {
                            let pat = t.pat.to_token_stream().to_string();
                            let ty = t.ty.to_token_stream().to_string();
                            params.push(format!("{} : {}", pat, ty))
                        }
                    }
                }

                let fn_meta = FunctionMeta::new(params, func_vis, func_output);
                file_info.functions.insert(func_name, fn_meta);
            }
            syn::Item::Const(var) => {
                let const_name = var.ident.to_string();
                file_info.variables.push(const_name);
            }
            syn::Item::Enum(en) => {
                let enum_name = en.ident.to_string();
                let enum_fields: Vec<String> =
                    en.variants.iter().map(|v| v.ident.to_string()).collect();
                file_info.enums.insert(enum_name, enum_fields);
            }
            syn::Item::Struct(struc) => {
                let struct_name = struc.ident.to_string();
                let mut struct_fields: Vec<String> = Vec::new();
                match struc.fields {
                    syn::Fields::Named(named_fields) => {
                        for field in named_fields.named {
                            if let Some(ident) = field.ident {
                                struct_fields.push(ident.to_string());
                            }
                        }
                    }
                    syn::Fields::Unnamed(unnamed_fields) => {
                        for (i, field) in unnamed_fields.unnamed.into_iter().enumerate() {
                            let field_type = field.ty.to_token_stream().to_string();
                            struct_fields.push(format!("field{}: {}", i, field_type));
                        }
                    }
                    syn::Fields::Unit => {}
                }
                file_info.structs.insert(struct_name, struct_fields);
            }
            syn::Item::Static(var) => {
                let var_name = var.ident.to_string();
                file_info.variables.push(var_name);
            }
            _ => {}
        }
    }

    Ok(file_info)
}

/// Main traversal logic that walks through a repository
/// and gathers information. Returns a Result wrapping RepoCodeContext.
pub fn walk_repo(dir_path: PathBuf) -> Result<RepoCodeContext, RepoError> {
    let repo_name = dir_path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();
    let mut repo = RepoCodeContext::new(repo_name);

    for entry in WalkDir::new(&dir_path).into_iter().filter_map(Result::ok) {
        if entry.file_type().is_file() && !invalid_path(&entry) {
            if let Some(ext) = entry.path().extension().and_then(|s| s.to_str()) {
                let lang = map_extension_to_language(ext);
                *repo.languages.entry(lang.clone()).or_insert(0) += 1;

                // Handle dependency files
                if is_cargo_toml(&entry) {
                    let file = File::open(entry.path())?;
                    let cargo_file = CargoToml::parse(file)?;
                    repo.dependencies.push(cargo_file);
                }

                // Parse Rust source files
                if ext == "rs" {
                    let file_info = parse_rust_file(&entry)?;
                    repo.files.push(file_info);
                } else {
                    todo!(
                        "Parsing for files with extension '{}' is not implemented",
                        ext
                    );
                }
            }
        } else if entry.file_type().is_dir() && !invalid_path(&entry) {
            if let Some(folder_name) = entry.path().file_name().and_then(|s| s.to_str()) {
                repo.folders.push(folder_name.to_string());
            }
        }
    }

    Ok(repo)
}
