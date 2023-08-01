use colored::*;
use dialoguer::{theme::ColorfulTheme, MultiSelect};
use serde::{Deserialize, Serialize};
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

#[derive(Debug, Serialize, Deserialize)]
pub struct Package {
    name: String,
    current_version: String,
    latest_version: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Registry {
    name: String,
    version: String,
}

pub fn update_packages() -> Result<Vec<Package>, Box<dyn Error>> {
    let mut packages = Vec::new();

    let current_dir = env::current_dir()?;

    let file = fs::read_to_string(current_dir.join("package.json"))?;
    let json: serde_json::Value = serde_json::from_str(&file)?;

    let dependencies = json["dependencies"].as_object();
    // let dev_dependencies = json["devDependencies"].as_object();

    for (name, version) in dependencies.unwrap().iter() {
        let mut package = Package {
            name: name.to_string(),
            current_version: version.to_string().replace("\"", ""),
            latest_version: "".to_string(),
        };

        let url = format!("https://registry.npmjs.org/{}/latest", package.name);

        let latest: Registry = reqwest::blocking::get(url)?.json()?;

        let specifier = package.current_version.chars().nth(0).unwrap();

        package.latest_version = format_args!(
            "{}{}",
            specifier,
            latest.version.to_string().replace("\"", "")
        )
        .to_string();

        let ver = compare_versions(
            package.current_version.clone(),
            package.latest_version.clone(),
        );

        if package.current_version != package.latest_version {
            println!(
                "{}: {} - {}",
                package.name,
                package.current_version.cyan(),
                ver
            );

            packages.push(package);
        }
    }

    return Ok(packages);
}

fn compare_versions(mut v0: String, mut v1: String) -> String {
    v0.remove(0);
    let v1_specifier = v1.remove(0);

    let v0 = v0.split(".").collect::<Vec<&str>>();
    let v1 = v1.split(".").collect::<Vec<&str>>();
    let ver = String::new();

    if v0[0] < v1[0] {
        return format_args!(
            "{}{}.{}.{}",
            v1_specifier,
            v0[0].red(),
            v0[1].red(),
            v0[2].red()
        )
        .to_string();
    }

    if v0[1] < v1[1] {
        return format_args!(
            "{}{}.{}.{}",
            v1_specifier,
            v0[0],
            v0[1].yellow(),
            v0[2].yellow()
        )
        .to_string();
    }

    if v0[2] < v1[2] {
        return format_args!("{}{}.{}.{}", v1_specifier, v0[0], v0[1], v0[2].green()).to_string();
    }

    return ver;
}
