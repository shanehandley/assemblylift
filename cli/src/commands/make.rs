use clap::ArgMatches;
use handlebars::to_json;
use serde_json::value::{Map, Value as Json};

use crate::bom;
use crate::bom::DocumentSet;
use crate::projectfs::{locate_asml_manifest, Project};

pub fn command(matches: Option<&ArgMatches>) {
    let matches = match matches {
        Some(matches) => matches,
        _ => panic!("could not get matches for make command"),
    };

    let manifest = match locate_asml_manifest() {
        Some(manifest) => manifest,
        None => panic!("could not find assemblylift.toml in tree"),
    };

    let project = Project::new(manifest.0.project.name, Some(manifest.1));

    let mut resource_type: Option<&str> = None;
    let mut resource_name: Option<&str> = None;
    for el in matches.values_of("resource").unwrap() {
        if resource_type.is_none() {
            resource_type = Some(el);
            continue;
        }
        if resource_name.is_none() {
            resource_name = Some(el);
            continue;
        }
    }

    match resource_type {
        Some("service") => {
            let data = &mut Map::<String, Json>::new();
            data.insert(
                "service_name".to_string(),
                to_json(resource_name.unwrap().to_string()),
            );
            let path = project
                .service_dir(String::from(resource_name.unwrap()))
                .dir();
            bom::service::Manifest::write(&path, data);
        }

        Some("function") => {
            // TODO if language==rust
            let resource_name = resource_name.unwrap().to_string();
            let function_name: Vec<&str> = resource_name.split(".").collect();
            if function_name.len() != 2 {
                panic!("syntax is `make function <service>.<function>`")
            }

            let data = &mut Map::<String, Json>::new();
            data.insert("function_name".to_string(), to_json(function_name[1]));
            let path = project
                .service_dir(String::from(function_name[0]))
                .function_dir(String::from(function_name[1]));
            bom::function::RustFunction::write(&path, data);
        }

        Some(_) => panic!("must specify either 'service' or 'function' as an argument to make"),
        None => panic!("must specify either 'service' or 'function' as an argument to make"),
    }
}
