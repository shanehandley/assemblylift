use std::process;

use clap::ArgMatches;
use handlebars::to_json;
use serde_json::value::{Map, Value as Json};

use crate::bom;
use crate::bom::DocumentSet;
use crate::projectfs::Project;
use crate::terraform;

pub fn command(matches: Option<&ArgMatches>) {
    let matches = match matches {
        Some(matches) => matches,
        _ => panic!("could not get matches for init command"),
    };

    let default_service_name = "my-service";
    let default_function_name = "my-function";
    let project_name = matches.value_of("project_name").unwrap();

    let project = Project::new(project_name.parse().unwrap(), None);
    project
        .init(default_service_name, default_function_name)
        .unwrap();

    terraform::fetch(&*project.dir());

    let data = &mut Map::<String, Json>::new();
    data.insert(
        "project_name".to_string(),
        to_json(project_name.to_string()),
    );
    data.insert(
        "default_service_name".to_string(),
        to_json(default_service_name.to_string()),
    );
    bom::manifest::Manifest::write(&*project.dir(), data);

    let data = &mut Map::<String, Json>::new();
    data.insert(
        "service_name".to_string(),
        to_json(default_service_name.to_string()),
    );
    bom::service::Manifest::write(
        &project
            .service_dir(String::from(default_service_name))
            .dir(),
        data,
    );

    match matches.value_of("language") {
        Some("rust") => {
            assert_prereqs();

            let data = &mut Map::<String, Json>::new();
            data.insert(
                "function_name".to_string(),
                to_json(default_function_name.to_string()),
            );
            bom::function::RustFunction::write(
                &project
                    .service_dir(String::from(default_service_name))
                    .function_dir(String::from(default_function_name)),
                data,
            );
        }
        Some(unknown) => panic!("unsupported language: {}", unknown),
        _ => {}
    }

    println!("\r\n✅  Done! Your project root is: {:?}", project.dir())
}

fn check_rust_prereqs() -> bool {
    let cargo_version = process::Command::new("cargo").arg("--version").output();

    match cargo_version {
        Ok(_version) => true,
        Err(_) => {
            println!("ERROR: missing Cargo!");
            false
        }
    }
}

fn assert_prereqs() {
    if !check_rust_prereqs() {
        panic!("missing system dependencies")
    }
}
