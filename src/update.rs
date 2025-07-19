use colored::*;
use node_semver::{Range, Version};
use serde::{Deserialize, Serialize};
use std::{env, error::Error, fs};

#[derive(Debug, Serialize, Deserialize)]
pub struct Package {
    name: String,
    current_version: String,
    latest_version: String,
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

    if !json["dependencies"].is_object() {
        return Err("No dependencies found".into());
    }

    //let dependencies = json["dependencies"].as_object();
    let dependencies = json["devDependencies"].as_object();

    for (name, version) in dependencies.unwrap().iter() {
        let mut package = Package {
            name: name.to_string(),
            current_version: version.to_string().replace("\"", ""),
            latest_version: "".to_string(),
            specifier: "".to_string(),
            update_available: false,
        };

        // get latest version from npm registry
        let url = format!("https://registry.npmjs.org/{}/latest", package.name);
        let latest_version: Registry = reqwest::blocking::get(url)?.json()?;

        package.latest_version = latest_version.version.clone();

        let specifier = package.current_version.chars().nth(0).unwrap();
        let mut final_specifier = specifier.to_string();

        // there is no specifier if the version is numeric
        if specifier.is_numeric() {
            final_specifier = "".to_string();
        }

        package.specifier = final_specifier.clone();

        let compared_version = compare_versions_v2(&mut package);

        if package.update_available && specifier != '*' {
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

fn compare_versions_v2(package: &mut Package) -> String {
    let version: Range = package
        .current_version
        .parse()
        .expect("Failed to parse range");
    let min_version = version.min_version().expect("Failed to get min version");

    let latest: Version = package
        .latest_version
        .parse()
        .expect("Failed to parse version");

    if latest.gt(&min_version) {
        package.update_available = true;

        if latest.major > min_version.major {
            return format_args!(
                "{}{}.{}.{}",
                package.specifier,
                latest.major.to_string().red(),
                latest.minor.to_string().red(),
                latest.patch.to_string().red()
            )
            .to_string();
        }

        if latest.minor > min_version.minor {
            return format_args!(
                "{}{}.{}.{}",
                package.specifier,
                latest.major.to_string(),
                latest.minor.to_string().yellow(),
                latest.patch.to_string().yellow()
            )
            .to_string();
        }

        if latest.patch > min_version.patch {
            return format_args!(
                "{}{}.{}.{}",
                package.specifier,
                latest.major.to_string(),
                latest.minor.to_string(),
                latest.patch.to_string().green()
            )
            .to_string();
        }

        return String::from("nothing?");
    } else {
        return String::from("no update");
    }
}
