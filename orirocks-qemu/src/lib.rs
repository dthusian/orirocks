use std::collections::HashMap;
use orirocks_api_v3::{Environment, EnvironmentProvider, Value};

#[derive(Default, Debug, Clone)]
pub struct QemuEnvironmentProvider;

impl EnvironmentProvider for QemuEnvironmentProvider {
  fn name(&self) -> &str {
    "qemu"
  }

  fn create(&self, dependencies: HashMap<String, String>, options: HashMap<String, Value>) -> Result<Box<dyn Environment>, String> {
    todo!()
  }
}

#[derive(Default, Debug)]
pub struct QemuEnvironment {

}

impl Environment for QemuEnvironment {
  fn action(&mut self, name: &str, options: HashMap<String, Value>) -> Result<(), String> {
    todo!()
  }

  fn finish(self, out_path: &str) -> Result<(), String> {
    todo!()
  }
}