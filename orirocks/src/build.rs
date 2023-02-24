use std::collections::{HashMap, HashSet, VecDeque};
use std::fs::File;
use std::hash::{Hash, Hasher};
use std::io::Read;
use std::iter;
use std::path::Path;
use log::info;
use serde::{Deserialize, Serialize};
use crate::model::{BuildDoc, Document, FunctionDoc, Import, Step};
use crate::util::{ORError, ORResult, YamlLocation, validate_identifier, Located, SHA256Hasher, sha256_trunc};

#[derive(Default, Clone, Debug, Eq, PartialEq)]
pub struct Project {
  imports: Vec<Located<Import>>,
  functions: HashMap<String, Located<FunctionDoc>>,
  builds: HashMap<String, Located<BuildDoc>>
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
  build_hashes: HashMap<String, u64>
}

#[derive(Default, Clone, Debug)]
pub struct OrderedDependencyGraph {
  deploys: Vec<String>,
  artifacts: Vec<(String, Vec<String>)>
}

/// Reads and updates the build cache and returns an ordered dependency graph
pub fn update_cache(project: &Project, build_cache: &mut BuildCache) -> ORResult<OrderedDependencyGraph> {
  #[derive(Default)]
  struct IsCleanCache {
    import_clean: HashMap<String, bool>,
    fn_clean: HashMap<String, bool>,
    build_clean: HashMap<String, bool>
  }
  let mut is_clean_cache = IsCleanCache::default();
  fn is_hash_clean(is_clean_cache: &mut HashMap<String, bool>, hash_cache: &mut HashMap<String, u64>, s: &str, obj: &impl Hash) -> bool {
    if is_clean_cache.contains_key(s) {
      *is_clean_cache.get(s).unwrap()
    } else {
      let hash = sha256_trunc(&obj);
      let is_clean = hash_cache.get(s)
        .map(|v| *v == hash)
        .unwrap_or(false);
      hash_cache.insert(s.into(), hash);
      is_clean_cache.insert(s.into(), is_clean);
      is_clean
    }
  }

  // does not check dependencies but checks function and import blocks used
  fn check_artifact_itself_clean(name: &str, project: &Project, build_cache: &mut BuildCache, icc: &mut IsCleanCache) -> bool {
    let artifact = &project.builds[name];
    let artifact_is_clean =
      is_hash_clean(
        &mut icc.build_clean,
        &mut build_cache.build_hashes,
        name,
        &**artifact
      ) &&
        artifact.envs.iter().all(|v| {
          let import_name = v.name.split_once("/").unwrap().0;
          is_hash_clean(
            &mut icc.import_clean,
            &mut build_cache.import_hashes,
            &v.name,
            &**project.imports.iter().find(|v| v.require == import_name).unwrap()
          ) && v.steps.iter().all(|v| match v {
            Step::EnvironmentStep(v) => true,
            Step::InvokeFunctionStep(v) => is_hash_clean(
              &mut icc.fn_clean,
              &mut build_cache.fn_hashes,
              &v.invoke_fn,
              &**project.functions.get(&v.invoke_fn).unwrap()
            ),
            Step::Null => panic!("Project contains null step")
          })
        });
    artifact_is_clean
  }

  fn is_clean_dfs<'a>(arti: &'a String, project: &'a Project, clean_artifacts: &mut HashSet<&'a String>, dirty_artifacts: &mut HashSet<&'a String>, check_artifact_itself_clean: &mut dyn FnMut(&str) -> bool) -> ORResult<bool> {
    if check_artifact_itself_clean(arti) {
      return Ok(false);
    }
    let mut stack = Vec::new();
    stack.push(arti);
    let mut visited = HashSet::new();
    visited.insert(arti);
    let mut is_clean = true;
    while !stack.is_empty() {
      let arti = stack.pop().unwrap();
      if dirty_artifacts.contains(arti) {
        is_clean = false;
        break;
      } else if clean_artifacts.contains(arti) {
        continue;
      } else {
        let is_itself_clean = check_artifact_itself_clean(arti);
        if !is_itself_clean {
          is_clean = false;
          break;
        }
        let build_doc = &project.builds[arti];
        for dep in build_doc.depends
          .iter()
          .flatten()
          .chain(build_doc.from.iter())
        {
          stack.push(dep);
          visited.insert(dep);
        }
      }
    }
    if is_clean {
      clean_artifacts.insert(arti);
    } else {
      dirty_artifacts.insert(arti);
    }
    Ok(is_clean)
  };
  // dfs
  let mut clean_artifacts = HashSet::new();
  let mut dirty_artifacts = HashSet::new();
  for (name, arti) in &project.builds {
    is_clean_dfs(
      &name, project, &mut clean_artifacts, &mut dirty_artifacts,
      &mut |name: &str| check_artifact_itself_clean(name, project, build_cache, &mut is_clean_cache))?;
  };
  // add dependencies
  let mut dirty_artifacts_with_deps = HashMap::new();
  for name in dirty_artifacts {
    let build = &project.builds[name];
    dirty_artifacts_with_deps.insert(
      name,
      build.depends.iter()
        .flatten()
        .chain(build.from.iter())
        .filter(|v| clean_artifacts.contains(v))
        .collect::<Vec<_>>());
  }
  todo!()
}

/// Primary build function
pub fn build(project: &Project, buildcache: Option<BuildCache>, opts: BuildOptions) {
  info!("starting build");

}