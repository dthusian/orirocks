mod float;

pub use float::CmpFloat;

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

#[derive(Serialize, Deserialize)]
pub enum Request {
  ListEnvs { },
  CreateEnv {
    name: String,
    options: HashMap<String, Value>
  },
  ActionEnv {
    env: u128,
    name: String,
    options: HashMap<String, Value>
  },
  FinishEnv {
    env: u128,
    out_path: String
  },
  ListDeps { },
  Deploy {
    name: String,
    path: String,
    options: HashMap<String, Value>
  }
}

#[derive(Serialize, Deserialize)]
pub enum Response {
  ListEnvs {
    envs: Vec<String>
  },
  CreateEnv {
    env: u128
  },
  ActionEnv { },
  FinishEnv { },
  ListDeps {
    envs: Vec<String>
  },
  Deploy { }
}

#[derive(Serialize, Deserialize)]
pub enum ReverseRequest {
  GetRealPath {
    artifact: String
  }
}

#[derive(Serialize, Deserialize)]
pub enum ReverseResponse {
  GetRealPath {
    path: String
  }
}