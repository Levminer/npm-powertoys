use colored::*;
use dialoguer::{theme::ColorfulTheme, MultiSelect};
use node_semver::{Range, Version};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::{env, error::Error, fs};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Package {
    name: String,
    current_version: String,
    latest_version: String,
    formatted_latest_version: String,
    specifier: String,
    update_available: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Registry {
    name: String,
    version: String,
}

pub fn command() -> Result<Vec<Package>, Box<dyn Error>> {
    let mut packages = Vec::new();
    let current_dir = env::current_dir()?;

    // Read and parse package.json
    let file_path = current_dir.join("package.json");

    if !file_path.exists() {
        return Err("package.json not found in the current directory".into());
    }

    let file = fs::read_to_string(file_path)?;
    let json: serde_json::Value = serde_json::from_str(&file)?;

    // Process dependencies if they exist and are objects
    if let Some(dependencies) = json["dependencies"].as_object() {
        process_dependencies(dependencies, &mut packages)?;
    }

    if let Some(dev_dependencies) = json["devDependencies"].as_object() {
        process_dependencies(dev_dependencies, &mut packages)?;
    }

    // No updates available
    if packages.is_empty() {
        println!("No package updates available!");
        return Ok(packages);
    }

    // Prompt user to select packages to update
    clearscreen::clear().expect("Failed to clear screen");

    let selected_indexes = MultiSelect::with_theme(&ColorfulTheme::default())
        .with_prompt("Select package(s) to update")
        .items(
            &packages
                .iter()
                .map(|p| {
                    format!(
                        "{} ({} -> {})",
                        p.name,
                        p.current_version.cyan(),
                        p.formatted_latest_version
                    )
                })
                .collect::<Vec<String>>(),
        )
        .interact()
        .unwrap();

    // Apply updates to package.json
    modify_package_json(&packages, &selected_indexes)?;

    return Ok(packages);
}

fn process_dependencies(
    dependencies: &serde_json::Map<String, serde_json::Value>,
    packages: &mut Vec<Package>,
) -> Result<(), Box<dyn Error>> {
    let deps_vec: Vec<_> = dependencies.iter().collect();
    let results: Result<Vec<_>, Box<dyn Error + Send + Sync>> = deps_vec
        .par_iter()
        .map(
            |(name, version)| -> Result<Option<Package>, Box<dyn Error + Send + Sync>> {
                let mut package = Package {
                    name: name.to_string(),
                    current_version: version.to_string().replace("\"", ""),
                    latest_version: "".to_string(),
                    formatted_latest_version: "".to_string(),
                    specifier: "".to_string(),
                    update_available: false,
                };

                // get latest version from npm registry
                let url = format!("https://registry.npmjs.org/{}/latest", package.name);
                let latest_version: Registry = reqwest::blocking::get(&url)
                    .map_err(|e| -> Box<dyn Error + Send + Sync> { Box::new(e) })?
                    .json()
                    .map_err(|e| -> Box<dyn Error + Send + Sync> { Box::new(e) })?;

                package.latest_version = latest_version.version.clone();

                let specifier = package.current_version.chars().nth(0).unwrap();
                let mut final_specifier = specifier.to_string();

                // there is no specifier if the version is numeric
                if specifier.is_numeric() {
                    final_specifier = "".to_string();
                }

                package.specifier = final_specifier.clone();

                compare_versions(&mut package).unwrap();

                if package.update_available && specifier != '*' {
                    println!(
                        "{}: {} - {}",
                        package.name,
                        package.current_version.cyan(),
                        package.formatted_latest_version
                    );
                    Ok(Some(package))
                } else {
                    Ok(None)
                }
            },
        )
        .collect();

    match results {
        Ok(packages_vec) => {
            packages.extend(packages_vec.into_iter().flatten());
            Ok(())
        }
        Err(e) => Err(format!("Error processing dependencies: {}", e).into()),
    }
}

fn compare_versions(package: &mut Package) -> Result<(), Box<dyn Error>> {
    let version: Range = match package.current_version.parse() {
        Ok(v) => v,
        Err(e) => {
            return Err(format!("Failed to parse range for {}: {}", package.name, e).into());
        }
    };

    let min_version = match version.min_version() {
        Some(v) => v,
        None => {
            return Err(format!("Failed to get min version for {}", package.name).into());
        }
    };

    let latest: Version = match package.latest_version.parse() {
        Ok(v) => v,
        Err(e) => {
            return Err(format!("Failed to parse version for {}: {}", package.name, e).into());
        }
    };

    if latest.gt(&min_version) {
        package.update_available = true;

        if latest.major > min_version.major {
            package.formatted_latest_version = format_args!(
                "{}{}.{}.{}",
                package.specifier,
                latest.major.to_string(),
                latest.minor.to_string(),
                latest.patch.to_string(),
            )
            .to_string()
            .red()
            .to_string();

            return Ok(());
        }

        if latest.minor > min_version.minor {
            // if major is 0 minor updates are considered breaking changes
            if latest.major == 0 {
                package.formatted_latest_version = format_args!(
                    "{}{}.{}.{}",
                    package.specifier,
                    latest.major.to_string(),
                    latest.minor.to_string(),
                    latest.patch.to_string(),
                )
                .to_string()
                .red()
                .to_string();
            } else {
                package.formatted_latest_version = format_args!(
                    "{}{}.{}.{}",
                    package.specifier,
                    latest.major.to_string(),
                    latest.minor.to_string(),
                    latest.patch.to_string(),
                )
                .to_string()
                .yellow()
                .to_string();
            }

            return Ok(());
        }

        if latest.patch > min_version.patch {
            package.formatted_latest_version = format_args!(
                "{}{}.{}.{}",
                package.specifier,
                latest.major.to_string(),
                latest.minor.to_string(),
                latest.patch.to_string(),
            )
            .to_string()
            .green()
            .to_string();

            return Ok(());
        }

        return Ok(());
    } else {
        return Ok(());
    }
}

fn modify_package_json(
    packages: &Vec<Package>,
    selected_indexes: &Vec<usize>,
) -> Result<(), Box<dyn Error>> {
    let current_dir = env::current_dir()?;
    let file_path = current_dir.join("package.json");

    if !file_path.exists() {
        return Err("package.json not found in the current directory".into());
    }

    let file = fs::read_to_string(&file_path)?;
    let mut json: serde_json::Value = serde_json::from_str(&file)?;

    for &index in selected_indexes {
        if let Some(package) = packages.get(index) {
            // Check and update dependencies only if it exists and is an object
            if json.get("dependencies").is_some() && json["dependencies"].is_object() {
                if let Some(dependencies) = json["dependencies"].as_object_mut() {
                    if dependencies.contains_key(&package.name) {
                        if let Some(entry) = dependencies.get_mut(&package.name) {
                            *entry = serde_json::Value::String(format!(
                                "{}{}",
                                package.specifier, package.latest_version
                            ));
                        }
                    }
                }
            }

            // Check and update devDependencies only if it exists and is an object
            if json.get("devDependencies").is_some() && json["devDependencies"].is_object() {
                if let Some(dev_dependencies) = json["devDependencies"].as_object_mut() {
                    if dev_dependencies.contains_key(&package.name) {
                        if let Some(entry) = dev_dependencies.get_mut(&package.name) {
                            *entry = serde_json::Value::String(format!(
                                "{}{}",
                                package.specifier, package.latest_version
                            ));
                        }
                    }
                }
            }
        }
    }

    let updated_content = serde_json::to_string_pretty(&json)?;
    fs::write(&file_path, updated_content)?;

    println!(
        "{}",
        String::from(
            "package.json updated successfully. Run the install command to apply the updates."
        )
        .green()
        .bold()
    );

    Ok(())
}
