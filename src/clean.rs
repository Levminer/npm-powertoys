use dialoguer::{theme::ColorfulTheme, MultiSelect};
use std::{env, error::Error, fs};
use walkdir::WalkDir;

pub fn command() -> Result<Vec<String>, Box<dyn Error>> {
    let mut paths = Vec::new();
    let current_dir = env::current_dir()?;

    // Recursively walk all child directories
    for dir_entry in WalkDir::new(current_dir) {
        let dir = if let Ok(dir) = dir_entry {
            dir
        } else {
            return Err("Error reading directory".into());
        };
        let path = dir.path();

        if path.ends_with("node_modules") && path.is_dir() {
            // Filter out node_modules inside other node_modules
            let full_path = path.to_str().unwrap().to_string();
            let splitted_path: Vec<&str> = full_path.split("node_modules").collect();

            if !paths.contains(&splitted_path[0].to_string()) {
                paths.push(splitted_path[0].to_string());
            }
        }
    }

    // Add node_modules to each path for deletion
    for val in paths.iter_mut() {
        *val = val.to_owned() + "node_modules";
    }

    // No paths found
    if paths.is_empty() {
        return Err("No node_modules found".into());
    }

    // Choose which node_modules to remove
    let chosen_indexes = MultiSelect::with_theme(&ColorfulTheme::default())
        .with_prompt("Choose node_modules to remove")
        .items(&paths)
        .interact()?;

    for (i, path) in paths.iter().enumerate() {
        if chosen_indexes.contains(&i) {
            fs::remove_dir_all(path)?;
        }
    }

    return Ok(paths);
}
