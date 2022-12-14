use std::{env::args_os, time::Instant};

use database::{Database, DENO_JSON, PACKAGE_JSON, RUN_YAML};

use crate::exit::Exit;

use beautify::Beautify;

mod beautify;
mod database;
mod exit;
mod file;
mod script;

fn main() {
    // Get the first arg.
    let first_arg = args_os().nth(1);

    // Convert it to `Option<&str>`.
    let param = first_arg.as_ref().and_then(|arg| arg.to_str());

    // Match parameter.
    match param {
        // If initialization flag is set, initialize a new run.yaml file.
        Some("-i" | "--init") => {
            let start_time = Instant::now();

            // If package.json file exists in current directory, generate a script database using package.json scripts.
            if file::exists(PACKAGE_JSON) {
                let package_json = file::read(PACKAGE_JSON).exit();

                let db = Database::from_package_json(&package_json).exit();

                db.save().exit();
            }
            // If deno.json file exists in current directory, generate a script database using deno.json tasks.
            else if file::exists(DENO_JSON) {
                let deno_json = file::read(DENO_JSON).exit();

                let db = Database::from_deno_json(&deno_json).exit();
                db.save().exit();
            }
            // If no file above exists in current directory, generate a script database using example.
            else {
                let db = Database::from_example();

                db.save().exit();
            }

            let end_time = start_time.elapsed();

            println!(
                "{} {}\n\n{}",
                "run.yaml".green(),
                "is generated".yellow(),
                format!("{} {}", "in".green(), format!("{:.2?}", end_time).yellow())
            );
        }

        // If help flag is set, print a help message.
        Some("-h" | "--help") => {
            println!(
                "{}\n{}\n\n{}\n    {}\n\n{}\n    {}  {}\n    {} {}",
                "Run 0.1.0".yellow(),
                "A tool to manage end execute your scripts.".green(),
                "Usage:".green(),
                "run <SCRIPT NAME>".yellow(),
                "Flags:".green(),
                "--help, -h".yellow(),
                "Displays a help message.".green(),
                "--init, -i".yellow(),
                "Creates a run.yaml file.".green(),
            );
        }

        // If an alias or name is given, run the script associated with it.
        Some(alias_or_name) => {
            let run_yaml = file::read(RUN_YAML).exit();

            let db = Database::from_run_yaml(&run_yaml).exit();

            let exit_code = db.run(alias_or_name).exit();

            std::process::exit(exit_code);
        }

        // If no arg is given, print all the available scripts.
        None => {
            let run_yaml = file::read(RUN_YAML).exit();

            let database = Database::from_run_yaml(&run_yaml).exit();

            database.print();
        }
    }
}
