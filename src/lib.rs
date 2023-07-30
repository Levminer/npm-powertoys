use dialoguer::{theme::ColorfulTheme, MultiSelect};
use std::{env, error::Error, fs};
use walkdir::WalkDir;

pub fn rm_node_modules() -> Result<Vec<String>, Box<dyn Error>> {
    let mut paths = Vec::new();

    let current_dir = env::current_dir()?;

    // recursively walk all child directories
    for dir_entry in WalkDir::new(current_dir) {
        let entry = dir_entry?;
        let path = entry.path();

        if path.ends_with("node_modules") && path.is_dir() {
            // filter out node_modules inside other node_modules
            let full_path = path.to_str().unwrap().to_string();
            let splitted_path: Vec<&str> = full_path.split("node_modules").collect();

            if !paths.contains(&splitted_path[0].to_string()) {
                paths.push(splitted_path[0].to_string());
            }
        }
    }

    // add node_modules to each path for deletion
    for val in paths.iter_mut() {
        *val = val.to_owned() + "node_modules";
    }

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
