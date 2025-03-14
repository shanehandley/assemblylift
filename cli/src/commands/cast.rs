use std::fs;
use std::path::PathBuf;
use std::process;
use std::process::Stdio;

use clap::ArgMatches;

use crate::artifact;
use crate::bom;
use crate::bom::DocumentSet;
use crate::projectfs::Project;
use crate::terraform;
use crate::terraform::function::TerraformFunction;
use crate::terraform::service::TerraformService;

pub fn command(matches: Option<&ArgMatches>) {
    use std::io::Read;

    let _matches = match matches {
        Some(matches) => matches,
        _ => panic!("could not get matches for cast command"),
    };

    // Init the project structure -- panic if the project isn't in the current working dir
    let cwd = std::env::current_dir().unwrap();
    let asml_manifest = bom::manifest::Manifest::read(&cwd);
    let project = Project::new(asml_manifest.project.name.clone(), Some(cwd));

    // Download the latest runtime binary
    let runtime_url = &*format!(
        "http://runtime.assemblylift.akkoro.io/aws-lambda/{}/bootstrap.zip",
        clap::crate_version!()
    );
    let mut response = reqwest::blocking::get(runtime_url).unwrap();
    if !response.status().is_success() {
        panic!("unable to fetch asml runtime from {}", runtime_url);
    }
    let mut response_buffer = Vec::new();
    response.read_to_end(&mut response_buffer).unwrap();

    fs::create_dir_all("./.asml/runtime").unwrap();
    fs::write("./.asml/runtime/bootstrap.zip", response_buffer).unwrap();

    terraform::fetch(&*project.dir());

    let mut functions: Vec<TerraformFunction> = Vec::new();
    let mut services: Vec<TerraformService> = Vec::new();

    for (_, service) in asml_manifest.services {
        let service_manifest =
            bom::service::Manifest::read(&*project.service_dir(service.name.clone()).dir());
        let service_name = service_manifest.service.name.clone();

        let tf_service = TerraformService {
            name: service_name.clone(),
            has_layer: service_manifest.iomod.is_some(),
            has_http_api: service_manifest
                .api
                .functions
                .values()
                .any(|f| f.http.is_some()),
        };
        services.push(tf_service.clone());

        terraform::service::write(&*project.dir(), tf_service.clone()).unwrap();

        if let Some(iomod) = service_manifest.iomod {
            let mut dependencies: Vec<String> = Vec::new();
            for (name, dependency) in iomod.dependencies {
                match dependency.dependency_type.as_str() {
                    "file" => {
                        // copy file & rename it to `name`

                        let dependency_name = name.clone();
                        let dependency_path = dependency.from.clone();

                        let runtime_path = format!("./.asml/runtime/{}", dependency_name);
                        fs::copy(dependency_path, &runtime_path).unwrap();

                        dependencies.push(runtime_path);
                    }
                    _ => unimplemented!("only type=file is available currently"),
                }
            }

            artifact::zip_files(
                dependencies,
                format!("./.asml/runtime/{}.zip", &service_name),
                Some("iomod/"),
                false,
            );
        }

        for (_id, function) in service_manifest.api.functions {
            let function_artifact_path =
                format!("./net/services/{}/{}", &service_name, function.name);
            fs::create_dir_all(PathBuf::from(function_artifact_path.clone())).expect(&*format!(
                "unable to create path {}",
                function_artifact_path
            ));

            // Compile the function
            // TODO switch on function language, toggle compilation on/off

            let function_path = PathBuf::from(format!(
                "{}/Cargo.toml",
                project
                    .service_dir(service_name.clone())
                    .function_dir(function.name.clone())
                    .into_os_string()
                    .into_string()
                    .unwrap()
            ));

            let mode = "release"; // TODO should this really be the default?

            let mut cargo_build = process::Command::new("cargo")
                .arg("build")
                .arg(format!("--{}", mode))
                .arg("--manifest-path")
                .arg(function_path)
                .arg("--target")
                .arg("wasm32-unknown-unknown")
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .spawn()
                .unwrap();

            match cargo_build.wait() {
                Ok(_) => {}
                Err(_) => {}
            }

            let function_name_snaked = function.name.replace("-", "_");
            let copy_result = fs::copy(
                format!(
                    "{}/target/wasm32-unknown-unknown/{}/{}.wasm",
                    project
                        .service_dir(service_name.clone())
                        .function_dir(function.name.clone())
                        .into_os_string()
                        .into_string()
                        .unwrap(),
                    mode,
                    function_name_snaked
                ),
                format!("{}/{}.wasm", function_artifact_path.clone(), &function.name),
            );

            if copy_result.is_err() {
                println!("ERROR: {:?}", copy_result.err());
            }

            let wasm_path = format!("{}/{}.wasm", function_artifact_path.clone(), &function.name);

            artifact::zip_files(
                vec![wasm_path],
                format!("{}/{}.zip", function_artifact_path.clone(), &function.name),
                None,
                false,
            );

            let function_http = function.http.clone();
            let tf_function_service = tf_service.clone();
            let tf_function = TerraformFunction {
                name: function.name.clone(),
                handler_name: function.handler_name,
                service: service.name.clone(),
                service_has_layer: tf_function_service.has_layer,
                service_has_http_api: tf_function_service.has_http_api,
                http_verb: match function_http.as_ref() {
                    Some(http) => Some(http.verb.to_string()),
                    None => None,
                },
                http_path: match function_http.as_ref() {
                    Some(http) => Some(http.path.to_string()),
                    None => None,
                },
            };

            terraform::function::write(&*project.dir(), &tf_function).unwrap();
            functions.push(tf_function.clone());
        }
    }

    terraform::write(
        &*project.dir(),
        asml_manifest.project.name,
        functions,
        services,
    )
    .unwrap();

    terraform::commands::init();
    terraform::commands::plan();
}
