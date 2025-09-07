use clap::Command;
use npm_powertoys;

fn main() {
    let cli = Command::new("npm powertoys")
        .about("npm powertoys - Useful tools for npm")
        .subcommand(Command::new("clean").about("Remove all node_modules recursively"))
        .subcommand(Command::new("update").about("Update installed packages"))
        .get_matches();

    match cli.subcommand() {
        Some(("clean", _sub_m)) => match npm_powertoys::clean::command() {
            Ok(paths) => {
                println!("Deleted paths: {:?}", paths);
            }
            Err(e) => {
                eprintln!("Error: {:?}", e);
            }
        },
        Some(("update", _sub_m)) => match npm_powertoys::update::command() {
            Ok(packages) => {
                println!("packages: {:?}", packages);
            }
            Err(e) => {
                eprintln!("Error: {:?}", e);
            }
        },
        _ => {
            println!("No command used, use: npm-powertoys --help");
        }
    }
}
