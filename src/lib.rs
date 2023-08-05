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

    // read end parse package.json
    let file = fs::read_to_string(current_dir.join("package.json"))?;
    let json: serde_json::Value = serde_json::from_str(&file)?;

    if !json["dependencies"].is_object() {
        return Err("No dependencies found".into());
    }

    let dependencies = json["dependencies"].as_object();
    // let dev_dependencies = json["devDependencies"].as_object();

    for (name, version) in dependencies.unwrap().iter() {
        let mut package = Package {
            name: name.to_string(),
            current_version: version.to_string().replace("\"", ""),
            latest_version: "".to_string(),
        };

        // get latest version from npm registry
        let url = format!("https://registry.npmjs.org/{}/latest", package.name);
        let latest_version: Registry = reqwest::blocking::get(url)?.json()?;

        let specifier = package.current_version.chars().nth(0).unwrap();
        let mut final_specifier = specifier.to_string();

        // there is no specifier if the version is numeric
        if specifier.is_numeric() {
            final_specifier = "".to_string();
        }

        package.latest_version = format_args!(
            "{}{}",
            final_specifier,
            latest_version.version.to_string().replace("\"", "")
        )
        .to_string();

        let compared_version = compare_versions(
            package.current_version.clone(),
            package.latest_version.clone(),
        );

        if package.current_version.trim() != package.latest_version.trim() && specifier != '*' {
            println!(
                "{}: {} - {}",
                package.name,
                package.current_version.cyan(),
                compared_version
            );

            packages.push(package);
        }
    }

    return Ok(packages);
}

fn compare_versions(mut cv: String, mut lv: String) -> String {
    let mut v1_specifier: String = String::new();

    if !cv.chars().nth(0).unwrap().is_numeric() {
        cv.remove(0);
        v1_specifier = lv.remove(0).to_string();
    }

    let cv_arr = cv.split(".").collect::<Vec<&str>>();
    let lv_arr = lv.split(".").collect::<Vec<&str>>();
    let ver = String::new();

    if cv == "" {
        return ver;
    }

    if cv_arr[0].parse::<i32>().unwrap() < lv_arr[0].parse::<i32>().unwrap() {
        return format_args!(
            "{}{}.{}.{}",
            v1_specifier,
            lv_arr[0].red(),
            lv_arr[1].red(),
            lv_arr[2].red()
        )
        .to_string();
    }

    if cv_arr[1].parse::<i32>().unwrap() < lv_arr[1].parse::<i32>().unwrap() {
        return format_args!(
            "{}{}.{}.{}",
            v1_specifier,
            lv_arr[0],
            lv_arr[1].yellow(),
            lv_arr[2].yellow()
        )
        .to_string();
    }

    if cv_arr[2].parse::<i32>().unwrap() < lv_arr[2].parse::<i32>().unwrap() {
        return format_args!(
            "{}{}.{}.{}",
            v1_specifier,
            lv_arr[0],
            lv_arr[1],
            lv_arr[2].green()
        )
        .to_string();
    }

    return ver;
}
