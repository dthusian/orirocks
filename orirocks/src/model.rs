use std::collections::{BTreeMap, HashMap};
use serde::{Deserialize, Serialize};
use orirocks_api_v3::{CmpFloat, Value, ValueType};

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, Hash)]
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

#[derive(Serialize, Deserialize, Default, Clone, Debug, Eq, PartialEq, Hash)]
pub struct FunctionDoc {
  pub name: String,
  pub parameter_spec: ParameterSpec,
  pub steps: Vec<Step>
}

#[derive(Serialize, Deserialize, Default, Clone, Debug, Eq, PartialEq, Hash)]
pub struct BuildDoc {
  pub name: String,
  pub from: Option<String>,
  pub depends: Option<Vec<String>>,
  pub envs: Vec<Environment>
}

#[derive(Serialize, Deserialize, Default, Clone, Debug, Eq, PartialEq, Hash)]
pub struct DeployDoc {
  pub name: String,
  pub deploy_to: String,
  pub artifact: String,
  #[serde(flatten)]
  pub parameters: Parameters
}

#[derive(Serialize, Deserialize, Default, Clone, Debug, Eq, PartialEq, Hash)]
pub struct Import {
  pub require: String,
  pub version: String
}

#[derive(Serialize, Deserialize, Default, Clone, Debug, Eq, PartialEq, Hash)]
pub struct Environment {
  pub name: String,
  #[serde(flatten)]
  pub parameters: Parameters,
  pub steps: Vec<Step>
}

#[derive(Serialize, Deserialize, Default, Clone, Debug, Eq, PartialEq, Hash)]
#[serde(untagged)]
pub enum Step {
  EnvironmentStep(EnvironmentStep),
  InvokeFunctionStep(InvokeFunctionStep),
  #[default]
  Null
}

#[derive(Serialize, Deserialize, Default, Clone, Debug, Eq, PartialEq, Hash)]
pub struct EnvironmentStep {
  pub action: String,
  #[serde(flatten)]
  pub parameters: Parameters
}

#[derive(Serialize, Deserialize, Default, Clone, Debug, Eq, PartialEq, Hash)]
pub struct InvokeFunctionStep {
  pub invoke_fn: String,
  #[serde(flatten)]
  pub parameters: Parameters
}

pub type ParameterSpec = BTreeMap<String, Parameter>;
pub type Parameters = BTreeMap<String, Value>;

#[derive(Serialize, Deserialize, Default, Clone, Debug, Eq, PartialEq, Hash)]
pub struct Parameter {
  #[serde(rename = "type")]
  pub type_: ValueType,
  pub default: Option<Value>
}
