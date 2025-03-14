use std::path;
use std::path::PathBuf;
use std::{fs, io};

use clap::crate_version;
use handlebars::{to_json, Handlebars};
use serde::{Deserialize, Serialize};
use serde_json::value::{Map, Value as Json};

use crate::terraform::write_to_file;

static TERRAFORM_SERVICE: &str = r#"# Generated with assemblylift-cli {{asml_version}}
{{#if has_layer}}
resource "aws_lambda_layer_version" "asml_{{name}}_service_layer" {
  filename   = "${path.module}/../../../.asml/runtime/{{name}}.zip"
  layer_name = "{{name}}-service"

  source_code_hash = filebase64sha256("${path.module}/../../../.asml/runtime/{{name}}.zip")
}

output "service_layer_arn" {
  value = aws_lambda_layer_version.asml_{{name}}_service_layer.arn
}
{{/if}}

{{#if has_http_api}}
resource "aws_apigatewayv2_api" "{{name}}_http_api" {
  name          = "{{name}}"
  protocol_type = "HTTP"
}

resource "aws_apigatewayv2_stage" "{{name}}_default_stage" {
  api_id      = aws_apigatewayv2_api.{{name}}_http_api.id
  name        = "$default"
  auto_deploy = true
}

output "http_api_id" {
  value = aws_apigatewayv2_api.{{name}}_http_api.id
}

output "http_api_execution_arn" {
  value = aws_apigatewayv2_api.{{name}}_http_api.execution_arn
}
{{/if}}
"#;

#[derive(Clone, Serialize, Deserialize)]
pub struct TerraformService {
    pub name: String,
    pub has_layer: bool,
    pub has_http_api: bool,
}

pub fn write(project_path: &PathBuf, service: TerraformService) -> Result<(), io::Error> {
    let file_name = "service.tf";

    let mut reg = Handlebars::new();
    reg.register_template_string(file_name, TERRAFORM_SERVICE)
        .unwrap(); // templates are known at compile-time

    let mut data = Map::<String, Json>::new();
    data.insert("asml_version".to_string(), to_json(crate_version!()));
    data.insert("name".to_string(), to_json(&service.name));
    data.insert("has_layer".to_string(), to_json(service.has_layer));
    data.insert("has_http_api".to_string(), to_json(service.has_http_api));

    let render = reg.render(file_name, &data).unwrap();

    let path = &format!(
        "{}/net/services/{}",
        project_path.clone().into_os_string().into_string().unwrap(),
        &service.name
    );

    fs::create_dir_all(path).expect(&*format!("unable to create path {:?}", path));

    let file_path = &format!("{}/{}", path, file_name);
    let file_path = path::Path::new(file_path);

    write_to_file(&file_path, render)
}
