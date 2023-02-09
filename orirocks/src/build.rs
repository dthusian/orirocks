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
          project.imports.extend(import_doc.into_iter().map(|v| Located::new(location.clone(), v)));
        },
        Document::Function(function_doc) => {
          if project.functions.contains_key(&function_doc.name) {
            return Err(ORError::DuplicateSymbol(location.clone(), "function".into(), function_doc.name));
          }
          project.functions.insert(function_doc.name.clone(), Located::new(location.clone(), function_doc));
        }
        Document::Build(build_doc) => {
          if project.builds.contains_key(&build_doc.name) {
            return Err(ORError::DuplicateSymbol(location.clone(), "artifact".into(), build_doc.name));
          }
          project.builds.insert(build_doc.name.clone(), Located::new(location.clone(), build_doc));
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
  for (_, function) in &project.functions {
    validate_identifier(&function.name, Located::location(&function))?;
  }
  for (_, build) in &project.builds {
    validate_identifier(&build.name, Located::location(&build))?;
  }
  Ok(())
}

pub struct BuildOptions {
  /// Instructs orirocks to deploy all artifacts regardless of dirty status.
  /// By default, only deploy blocks that reference dirty artifacts are redeployed.
  pub redeploy: bool,
  /// Instructs orirocks to build all artifacts regardless of dirty status.
  pub rebuild: bool,
  /// Specifies the directory to store the build cache and intermediate artifacts
  pub build_dir: String
}

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct BuildCache {
  import_hashes: HashMap<String, u64>,
  fn_hashes: HashMap<String, u64>,
  build_hashes: HashMap<String, u64>,
  deploy_hashes: HashMap<String, u64>,
  clean_set: Vec<String>
}

#[derive(Default, Clone, Debug)]
struct OrderedDependencyGraph {
  order: Vec<String>,
  dependencies: HashMap<String, Vec<String>>
}

// 1. Check hashes of all documents
// 2. Remove artifacts from the clean set if they require rebuilding
// 3. Construct build plan with details on which artifacts to rebuild in which order and which can be parallelized
// 4. Execute the build plan

/// Reads and updates the build cache and returns a list of dirty artifacts and dirty deploy blocks
fn update_cache(project: &Project, build_cache: &mut BuildCache) -> (Vec<String>, Vec<String>) {
  todo!()
}

fn create_build_plan(project: &Project, buildcache: Option<BuildCache>, build_dir: Box<Path>) -> OrderedDependencyGraph {
  let can_use_cached = |artifact: &str| -> bool {
    let dependencies_cache_ok = project.builds.get(artifact).unwrap();
    let res = File::open(build_dir);
    todo!()
  };
  let mut graph = OrderedDependencyGraph::default();
  for deployment in &project.deploys {
    let artifact = &deployment.artifact;

  }
  graph
}

/// Primary build function
pub fn build(project: &Project, buildcache: Option<BuildCache>, opts: BuildOptions) {
  info!("starting build");

}