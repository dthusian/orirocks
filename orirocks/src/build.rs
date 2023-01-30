use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use log::info;
use serde::{Deserialize, Serialize};
use crate::model::{BuildDoc, DeployDoc, Document, FunctionDoc, Import};
use crate::util::{ORError, ORResult, YamlLocation, validate_identifier, Located};

#[derive(Default, Clone, Debug, Eq, PartialEq)]
pub struct Project {
  imports: Vec<Located<Import>>,
  functions: HashMap<String, Located<FunctionDoc>>,
  builds: HashMap<String, Located<BuildDoc>>,
  deploys: Vec<Located<DeployDoc>>
}

pub fn parse_project(files: Vec<(String, Box<dyn Read>)>) -> ORResult<Project> {
  let mut project = Project::default();
  for (filename, file) in files {
    for (i, document) in serde_yaml::Deserializer::from_reader(file).enumerate() {
      let location = YamlLocation::new(filename.clone(), i, vec![]);
      let value = Document::deserialize(document)
        .map_err(|v| ORError::YamlError(location.clone(), v))?;
      match value {
        Document::Import(import_doc) => {
          project.imports.extend(import_doc)
        },
        Document::Function(function_doc) => {
          if project.functions.contains_key(&function_doc.name) {
            return Err(ORError::DuplicateSymbol(location.clone(), "function".into(), function_doc.name));
          }
          project.functions.insert(function_doc.name.clone(), Located::new(location.clone(), function_doc));
        }
        Document::Build(build_doc) => {
          if project.builds.contains_key(&build_doc.artifact_name) {
            return Err(ORError::DuplicateSymbol(location.clone(), "artifact".into(), build_doc.artifact_name));
          }
          project.builds.insert(build_doc.artifact_name.clone(), Located::new(location.clone(), build_doc));
        }
        Document::Deploy(deploy_doc) => {
          project.deploys.push(Located::new(location.clone(), deploy_doc));
        }
      }
    }
  }
  Ok(project)
}

pub fn validate_project(project: &Project) -> ORResult<()> {
  for import in &project.imports {
    validate_identifier(&import.require, Located::location(&import))?;
    //TODO maybe validate semver
  }
  for function in &project.functions {

  }
  Ok(())
}

pub struct BuildOptions {
  pub no_deploy: bool,
  pub rebuild: bool,
  pub build_dir: String
}

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct BuildCache {
  import_hashes: HashMap<String, [u8; 32]>,
  fn_hashes: HashMap<String, [u8; 32]>,
}

#[derive(Default, Clone, Debug)]
struct OrderedDependencyGraph {
  order: Vec<String>,
  dependencies: HashMap<String, Vec<String>>
}

pub fn build(project: &Project, buildcache: Option<BuildCache>, opts: BuildOptions) {
  info!("starting build");
  fn create_build_plan(project: &Project, buildcache: Option<BuildCache>, build_dir: Box<Path>) -> OrderedDependencyGraph {
    let can_use_cached = |artifact: &str| -> bool {
      let dependencies_cache_ok = project.builds.get(artifact).unwrap().
      let res = File::open(build_dir);

    };
    let mut graph = OrderedDependencyGraph::default();
    for deployment in &project.deploys {
      let artifact = &deployment.artifact;

    }
    graph
  }
}