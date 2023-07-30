use clap::Command;
use npm_powertoys;

fn main() {
    let cli = Command::new("npm powertoys")
        .about("npm powertoys - Useful tools for npm")
        .subcommand(Command::new("rm").about("Remove node_modules"))
        .get_matches();

    match cli.subcommand() {
        Some(("rm", _sub_m)) => {
            let res = npm_powertoys::rm_node_modules();

            match res {
                Ok(paths) => {
                    println!("paths: {:?}", paths);
                }
                Err(e) => {
                    println!("Error: {:?}", e);
                }
            }
        }
        _ => {
            println!("No command used, use: npm-powertoys --help");
        }
    }
}
