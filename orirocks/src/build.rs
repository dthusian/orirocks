use std::collections::HashMap;
use std::fs::File;
use std::hash::{Hash, Hasher};
use std::io::Read;
use std::path::Path;
use log::info;
use serde::{Deserialize, Serialize};
use crate::model::{BuildDoc, DeployDoc, Document, FunctionDoc, Import, Step};
use crate::util::{ORError, ORResult, YamlLocation, validate_identifier, Located, SHA256Hasher, sha256_trunc};

#[derive(Default, Clone, Debug, Eq, PartialEq)]
pub struct Project {
  imports: Vec<Located<Import>>,
  functions: HashMap<String, Located<FunctionDoc>>,
  builds: HashMap<String, Located<BuildDoc>>,
  deploys: HashMap<String, Located<DeployDoc>>
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
          project.deploys.insert(deploy_doc.name.clone(), Located::new(location.clone(), deploy_doc));
        }
      }
    }
  }
  Ok(project)
}

pub fn validate_project(project: &Project) -> ORResult<()> {
  fn validate_step(step: &Step, loc: &mut YamlLocation) -> ORResult<()> {
    validate_identifier(match step {
      Step::EnvironmentStep(step) => &step.action,
      Step::InvokeFunctionStep(step) => &step.invoke_fn,
      Step::Null => Err(ORError::GenericInvalid(loc.clone()))?
    }, &loc)?;
    Ok(())
  }

  for import in &project.imports {
    validate_identifier(&import.require, Located::location(&import))?;
    //TODO maybe validate semver
  }
  for (_, function) in &project.functions {
    let mut loc = Located::location(&function).clone();
    validate_identifier(&function.name, &loc)?;
    for (i, step) in function.steps.iter().enumerate() {
      loc.push(format!("step #{}", i));
      validate_step(&step, &mut loc)?;
      loc.pop();
    }
  }
  for (_, build) in &project.builds {
    let mut loc = Located::location(&build).clone();
    validate_identifier(&build.name, &loc)?;
    for env in &build.envs {
      loc.push(env.name.clone());
      let (plugin, env_name) = env.name.split_once("/").ok_or_else(|| ORError::InvalidEnvironmentName(loc.clone()))?;
      validate_identifier(plugin, &loc)?;
      validate_identifier(env_name, &loc)?;
      for (i, step) in env.steps.iter().enumerate() {
        loc.push(format!("step #{}", i));
        validate_step(&step, &mut loc)?;
        loc.pop();
      }
      loc.pop();
    }
  }
  Ok(())
}

pub struct BuildOptions {
  /// Instructs orirocks to build and deploy all artifacts regardless of dirty status.
  pub rebuild: bool,
  /// Specifies the directory to store the build cache and intermediate artifacts
  pub build_dir: String
}

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct BuildCache {
  import_hashes: HashMap<String, u64>,
  fn_hashes: HashMap<String, u64>,
  build_hashes: HashMap<String, u64>,
  deploy_hashes: HashMap<String, u64>
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
pub fn update_cache(project: &Project, build_cache: &mut BuildCache) -> ORResult<(Vec<String>, Vec<String>)> {
  let mut import_clean = HashMap::new();
  let mut fn_clean = HashMap::new();
  let mut build_clean = HashMap::new();
  let mut deploy_clean = HashMap::new();
  fn is_clean(clean: &mut HashMap<String, bool>, cache: &mut HashMap<String, u64>, s: &str, obj: &impl Hash) -> bool {
    if clean.contains_key(s) {
      *clean.get(s).unwrap()
    } else {
      let hash = sha256_trunc(&obj);
      let is_clean = cache.get(s)
        .map(|v| *v == hash)
        .unwrap_or(false);
      cache.insert(s.into(), hash);
      clean.insert(s.into(), is_clean);
      is_clean
    }
  }
  let mut dirty_artifacts = vec![];
  let mut dirty_deploys = vec![];
  for (name, artifact) in &project.builds {
    let artifact_is_clean =
      is_clean(
        &mut build_clean,
        &mut build_cache.build_hashes,
        name,
        &**artifact
      ) &&
      artifact.envs.iter().all(|v| {
        let import_name = v.name.split_once("/").unwrap().0;
        is_clean(
          &mut import_clean,
          &mut build_cache.import_hashes,
          &v.name,
          &**project.imports.iter().find(|v| v.require == import_name).unwrap()
        ) && v.steps.iter().all(|v| match v {
          Step::EnvironmentStep(v) => true,
          Step::InvokeFunctionStep(v) => is_clean(
            &mut fn_clean,
            &mut build_cache.fn_hashes,
            &v.invoke_fn,
            &**project.functions.get(&v.invoke_fn).unwrap()
          ),
          Step::Null => panic!("Project contains null step")
        })
      });
    if !artifact_is_clean {
      dirty_artifacts.push(name.clone());
    }
  }
  for (name, deploy) in &project.deploys {
    let deploy_is_clean = is_clean(
      &mut deploy_clean,
      &mut build_cache.deploy_hashes,
      name,
      &**deploy
    );
    if !deploy_is_clean {
      dirty_deploys.push(name.clone());
    }
  }
  Ok((dirty_artifacts, dirty_deploys))
}

fn create_build_plan(project: &Project, buildcache: Option<BuildCache>, build_dir: Box<Path>) -> OrderedDependencyGraph {

  let mut graph = OrderedDependencyGraph::default();

  graph
}

/// Primary build function
pub fn build(project: &Project, buildcache: Option<BuildCache>, opts: BuildOptions) {
  info!("starting build");

}