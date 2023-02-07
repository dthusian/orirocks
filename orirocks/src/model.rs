use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use orirocks_api_v2::{CmpFloat, Value, ValueType};

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub enum Document {
  #[serde(rename = "import")]
  Import(ImportDoc),
  #[serde(rename = "function")]
  Function(FunctionDoc),
  #[serde(rename = "build")]
  Build(BuildDoc),
  #[serde(rename = "deploy")]
  Deploy(DeployDoc)
}

pub type ImportDoc = Vec<Import>;

#[derive(Serialize, Deserialize, Default, Clone, Debug, Eq, PartialEq)]
pub struct FunctionDoc {
  pub name: String,
  pub parameter_spec: ParameterSpec,
  pub steps: Vec<Step>
}

#[derive(Serialize, Deserialize, Default, Clone, Debug, Eq, PartialEq)]
pub struct BuildDoc {
  pub artifact_name: String,
  pub from: String,
  pub steps: Vec<Step>
}

#[derive(Serialize, Deserialize, Default, Clone, Debug, Eq, PartialEq)]
pub struct DeployDoc {
  pub deploy_to: String,
  pub artifact: String,
  #[serde(flatten)]
  pub parameters: Parameters
}

#[derive(Serialize, Deserialize, Default, Clone, Debug, Eq, PartialEq)]
pub struct Import {
  pub require: String,
  pub version: String
}

#[derive(Serialize, Deserialize, Default, Clone, Debug, Eq, PartialEq)]
#[serde(untagged)]
pub enum Step {
  EnvironmentStep(EnvironmentStep),
  InvokeFunctionStep(InvokeFunctionStep),
  #[default]
  Null
}

#[derive(Serialize, Deserialize, Default, Clone, Debug, Eq, PartialEq)]
pub struct EnvironmentStep {
  pub action: String,
  #[serde(flatten)]
  pub parameters: Parameters
}

#[derive(Serialize, Deserialize, Default, Clone, Debug, Eq, PartialEq)]
pub struct InvokeFunctionStep {
  pub invoke_fn: String,
  #[serde(flatten)]
  pub parameters: Parameters
}

pub type ParameterSpec = HashMap<String, Parameter>;
pub type Parameters = HashMap<String, Value>;

#[derive(Serialize, Deserialize, Default, Clone, Debug, Eq, PartialEq)]
pub struct Parameter {
  #[serde(rename = "type")]
  pub type_: ValueType,
  pub default: Option<Value>
}
