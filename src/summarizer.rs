use crate::explorer;
use serde::Serialize;
use serde_json;
use std::path::PathBuf;

#[derive(Serialize, Debug, Default)]
struct LLMInput {
    prompt_directives: String,
    project_name: String,
    project_language: String,
    project_dependencies: String,
    project_structure: Vec<String>,
}

impl LLMInput {
    fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
}

fn build_input(path: PathBuf) -> Result<String, Box<dyn std::error::Error>> {
    let mut llm_input = LLMInput::default();
    let repo = explorer::walk_repo(path)?;
    llm_input.project_name = repo.repo_name;
    llm_input.project_language = repo
        .languages
        .iter()
        .map(|(lang, count)| format!("{}: {} files", lang, count))
        .collect::<Vec<String>>()
        .join(", ");

    llm_input
        .project_structure
        .push(format!("Folders: {}", repo.folders.join(", ")));

    for file in repo.files {
        let functions_detail = file
            .functions
            .iter()
            .map(|(name, meta)| {
                format!(
                    "{} (params: [{}], returns: {}, visibility: {})",
                    name,
                    meta.params.join(", "),
                    meta.returns,
                    meta.visibility
                )
            })
            .collect::<Vec<_>>()
            .join(" ; ");

        let file_details = format!(
            "File: {} | Functions: [{}] | Structs: [{}] | Variables: [{}] | Enums: [{}]",
            file.file_name,
            functions_detail,
            file.structs
                .keys()
                .cloned()
                .collect::<Vec<String>>()
                .join(", "),
            file.variables.join(", "),
            file.enums
                .keys()
                .cloned()
                .collect::<Vec<String>>()
                .join(", ")
        );
        llm_input.project_structure.push(file_details);
    }

    // Build dependency information.
    let mut deps = Vec::new();
    for cargo in repo.dependencies {
        if let Some(pkg) = cargo.package {
            let pkg_desc = pkg
                .description
                .map_or(String::new(), |desc| format!(" - {}", desc));
            deps.push(format!(
                "Package: {} v{}{}",
                pkg.name, pkg.version, pkg_desc
            ));
        }

        if let Some(dep_map) = cargo.dependencies {
            for (dep_name, dep_value) in dep_map {
                deps.push(format!("Dependency: {}: {:?}", dep_name, dep_value));
            }
        }

        if let Some(dev_dep_map) = cargo.dev_dependencies {
            for (dep_name, dep_value) in dev_dep_map {
                deps.push(format!("Dev Dependency: {}: {:?}", dep_name, dep_value));
            }
        }
    }
    llm_input.project_dependencies = deps.join(", ");

    llm_input.prompt_directives = r#"You are a README generator.
You will be provided with a project name, its language, its dependencies, and its structure.
You will generate a README file for the project.
The README should include:
- Project name
- Project language
- Project dependencies
- Project structure
- A brief description of each file and its contents
- How to run the project
- Any other relevant information
The README should be in Markdown format.
The README should be clear and concise.
The README should be easy to read and understand."#
        .trim()
        .to_owned();

    // Serialize to JSON and return.
    llm_input.to_json().map_err(|e| e.into())
}
