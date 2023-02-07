use std::{alloc, mem};
use std::alloc::Layout;
use std::collections::HashMap;
use std::ffi::c_void;
use std::mem::size_of;
use std::ptr::{null, slice_from_raw_parts};
use crate::ffi;
use crate::ffi::{Array, Param, PluginManifest};
use crate::marshal::{marshal_res, unmarshal_params, unmarshal_string};

pub unsafe fn init_handler(init_fn: impl FnOnce(PluginBuilder) -> PluginBuilder) -> *const PluginManifest {
  let manifest = init_fn(PluginBuilder {
    name: None,
    environments: vec![],
    deployments: vec![]
  }).build();
  let ptr = alloc::alloc(Layout::for_value(&manifest)) as *mut PluginManifest;
  *ptr = manifest;
  ptr
}

pub unsafe fn destroy_handler(plugin: *const PluginManifest) {
  let manifest = plugin.read();
  Vec::from_raw_parts(
    manifest.environments.ptr as *mut ffi::DeploymentProvider,
    manifest.environments.len as usize * size_of::<ffi::EnvironmentProvider>(),
    manifest.environments.len as usize * size_of::<ffi::EnvironmentProvider>());
  Vec::from_raw_parts(
    manifest.deployments.ptr as *mut ffi::DeploymentProvider,
    manifest.deployments.len as usize * size_of::<ffi::DeploymentProvider>(),
    manifest.deployments.len as usize * size_of::<ffi::DeploymentProvider>());
  alloc::dealloc(plugin as *mut u8, Layout::for_value(&*plugin));
}

pub struct PluginBuilder {
  name: Option<&'static str>,
  environments: Vec<ffi::EnvironmentProvider>,
  deployments: Vec<ffi::DeploymentProvider>
}

impl PluginBuilder {
  pub fn build(mut self) -> PluginManifest {
    let name = self.name.unwrap();
    let name_len = name.bytes().len();
    let name_ptr = name.as_ptr();
    self.environments.shrink_to_fit();
    let env_len = self.environments.len();
    let env_ptr = self.environments.as_ptr();
    self.deployments.shrink_to_fit();
    let dep_len = self.deployments.len();
    let dep_ptr = self.deployments.as_ptr();
    mem::forget(self);
    PluginManifest {
      version: 0,
      name: Array { ptr: name_ptr, len: name_len as u64 },
      environments: Array { ptr: env_ptr, len: env_len as u64 },
      deployments: Array { ptr: dep_ptr, len: dep_len as u64 }
    }
  }
}

trait EnvironmentProvider: Sized {
  fn name() -> &'static str;
  fn create(params: HashMap<String, String>) -> Result<Self, &'static str>;
  fn action(&mut self, name: &str, params: HashMap<String, String>) -> Result<(), &'static str>;
  fn finish(self, path: &str) -> Result<(), &'static str>;
}

unsafe fn get_env_ffi_repr<T: EnvironmentProvider>() -> ffi::EnvironmentProvider {
  unsafe extern "C" fn create<T: EnvironmentProvider>(params: Array<Param>, env_return: *mut *mut c_void) -> Array<u8> {
    let params = unmarshal_params(params);
    let env = T::create(params);
    match env {
      Ok(env) => {
        let alloc = alloc::alloc(Layout::for_value(&env)) as *mut T;
        *(env_return as *mut *mut T) = alloc;
        Array { ptr: null(), len: 0 }
      }
      Err(err) => Array { ptr: err.as_ptr(), len: err.bytes().len() as u64 }
    }
  }

  unsafe extern "C" fn action<T: EnvironmentProvider>(env: *mut c_void, name: Array<u8>, params: Array<Param>) -> Array<u8> {
    let params = unmarshal_params(params);
    let res = (env as *mut T).as_mut().unwrap().action(&unmarshal_string(name), params);
    marshal_res(res)
  }

  unsafe extern "C" fn finish<T: EnvironmentProvider>(env: *mut c_void, out_path: Array<u8>) -> Array<u8> {
    let env = env as *mut T;
    let path = unmarshal_string(out_path);
    let res = env.read().finish(&path);
    alloc::dealloc(env as *mut u8, Layout::for_value(&env));
    marshal_res(res)
  }

  ffi::EnvironmentProvider {
    name: Array {
      ptr: T::name().as_ptr(),
      len: T::name().bytes().len() as u64
    },
    create: create::<T>,
    action: action::<T>,
    finish: finish::<T>
  }
}

trait DeploymentProvidor {
  fn name() -> &'static str;
  fn deploy(path: &str, params: HashMap<String, String>) -> Result<(), &'static str>;
}

unsafe fn get_dep_ffi_repr<T: DeploymentProvidor>() -> ffi::DeploymentProvider {
  unsafe extern "C" fn deploy<T: DeploymentProvidor>(path: Array<u8>, params: Array<Param>) -> Array<u8> {
    let params = unmarshal_params(params);
    let res = T::deploy(&unmarshal_string(path), params);
    marshal_res(res)
  }
  
  ffi::DeploymentProvider {
    name: Array {
      ptr: T::name().as_ptr(),
      len: T::name().bytes().len() as u64
    },
    deploy: deploy::<T>
  }
}