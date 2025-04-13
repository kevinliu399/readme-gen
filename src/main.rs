use std::env;
use std::path::PathBuf;

mod explorer;
mod llm;
mod summarizer;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();

    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <folder_path>", args[0]);
        return Ok(());
    }

    let folder_path = PathBuf::from(&args[1]);

    let markdown_content = llm::generate_md(folder_path).await?;

    std::fs::write("README.md", &markdown_content)?;
    println!("Markdown file 'README.md' generated successfully!");

    Ok(())
}
