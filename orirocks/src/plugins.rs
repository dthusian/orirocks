use std::collections::HashMap;
use orirocks_api_v3::{DeploymentProvider, EnvironmentProvider};
use orirocks_qemu::QemuEnvironmentProvider;

fn collect_plugins() -> (Vec<Box<dyn EnvironmentProvider>>, Vec<Box<dyn DeploymentProvider>>) {
  let mut env_providers: Vec<Box<dyn EnvironmentProvider>> = vec![];
  let mut dep_providers: Vec<Box<dyn DeploymentProvider>> = vec![];

  #[cfg(feature = "plugin-qemu")]
  env_providers.push(Box::new(QemuEnvironmentProvider::default()));

  (env_providers, dep_providers)
}

pub struct PluginHive {
  env: HashMap<String, Box<dyn EnvironmentProvider>>,
  dep: HashMap<String, Box<dyn DeploymentProvider>>
}

impl PluginHive {
  pub fn new() -> Self {
    let (collected_envs, collected_deps) = collect_plugins();
    PluginHive {
      env: collected_envs.into_iter().map(|v| (v.name().to_string(), v)).collect(),
      dep: collected_deps.into_iter().map(|v| (v.name().to_string(), v)).collect(),
    }
  }
}
