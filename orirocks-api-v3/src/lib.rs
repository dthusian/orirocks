mod float;

pub use crate::float::CmpFloat;

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
#[serde(untagged)]
pub enum Value {
  Bool(bool),
  Integer(i64),
  Float(CmpFloat),
  Array(Vec<Value>),
  String(String),
  Dict(HashMap<String, Value>)
}

#[derive(Serialize, Deserialize, Default, Clone, Debug, Eq, PartialEq)]
pub enum ValueType {
  #[serde(rename = "integer")]
  Integer,
  #[serde(rename = "float")]
  Float,
  #[serde(rename = "string")]
  #[default]
  String,
  #[serde(rename = "bool")]
  Bool,
  #[serde(rename = "array")]
  Array(Box<ValueType>),
  #[serde(rename = "dict")]
  Dict(Box<ValueType>)
}

/// Represents an object that can construct Environments
pub trait EnvironmentProvider {
  /// Retrieves the name of the environment provider
  fn name(&self) -> &str;
  /// Constructs an environment from this provider.
  /// `dependencies` is a mapping from resource locations to real filepaths.
  /// This ensures that if a plugin step depends on anything, it is declared here to aid dependency resolution.
  /// `options` is a plugin-defined set of options.
  fn create(&self, dependencies: HashMap<String, String>, options: HashMap<String, Value>) -> Result<Box<dyn Environment>, String>;
}

/// Represents an Environment provided by an EnvironmentProvider
pub trait Environment {
  /// Performs an action in this environment.
  /// `name` and `options` specify the name of the actions and plugin-defined options.
  fn action(&mut self, name: &str, options: HashMap<String, Value>) -> Result<(), String>;
  /// Finish executing this environment and clean it up.
  /// `path` is the filepath in which to save the result.
  fn finish(self, path: &str) -> Result<(), String>;
}

/// Represents a possible method of deployment defined in this plugin
pub trait DeploymentProvider {
  /// Returns the name of the deployment provider
  fn name(&self) -> &str;
  /// Executes a deployment. `dependencies` is the same as the parameter in `EnvironmentProvider`,
  /// and `options` is a plugin-defined set of options.
  fn deploy(&self, dependencies: HashMap<String, String>, options: HashMap<String, String>) -> Result<(), String>;
}