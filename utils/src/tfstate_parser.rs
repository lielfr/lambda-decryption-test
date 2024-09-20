use serde::Deserialize;
use serde_json::Value;
#[derive(Deserialize)]
struct TerraformInstance {
    attributes: Value,
}

#[derive(Deserialize)]
struct TerraformResource {
    module: String,
    #[serde(rename = "type")]
    resource_type: String,
    instances: Vec<TerraformInstance>,
}

#[derive(Deserialize)]
pub struct TerraformState {
    resources: Vec<TerraformResource>,
}

#[derive(Debug)]
pub struct LambdaFunctionName<'a> {
    pub module_name: &'a str,
    pub function_name: &'a str,
}

pub fn get_lambda_function_names(state: &TerraformState) -> Vec<LambdaFunctionName> {
    state
        .resources
        .iter()
        .filter(|r| r.resource_type.as_str() == "aws_lambda_function")
        .flat_map(|r| {
            let module_name = r
                .module
                .strip_prefix("module.")
                .unwrap_or(r.module.as_str());
            r.instances
                .iter()
                .filter_map(|inst| inst.attributes.get("function_name"))
                .filter_map(|f_name| {
                    f_name.as_str().map(|n| LambdaFunctionName {
                        module_name,
                        function_name: n,
                    })
                })
        })
        .collect()
}
