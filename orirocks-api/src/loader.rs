use std::collections::HashMap;
use std::ffi::{c_void, OsStr};
use std::marker::PhantomData;
use std::ptr::{null, null_mut};
use std::slice::from_raw_parts;
use libloading::{Library};
use crate::{Error, PLUGIN_VERSION};
use crate::ffi::{Array, EnvironmentProvider, Param, PluginDestroyFn, PluginInitFn, PluginManifest};

fn marshal_params(params: &HashMap<String, String>) -> Vec<Param> {
  params.iter().map(|v| Param {
    key: Array {
      ptr: v.0.as_ptr(),
      len: v.0.bytes().len() as u64
    },
    value: Array {
      ptr: v.1.as_ptr(),
      len: v.1.bytes().len() as u64
    }
  }).collect()
}

unsafe fn marshal_string(s: Array<u8>) -> String {
  String::from_utf8_lossy(from_raw_parts(s.ptr, s.len as usize)).to_string()
}

unsafe fn marshal_err(err: Array<u8>) -> Result<(), String> {
  if err.ptr != null() {
    Err(marshal_string(err))
  } else {
    Ok(())
  }
}

/// Safe wrapper for a plugin
pub struct Plugin {
  library: Library,
  mf: *const PluginManifest,
  env: HashMap<String, usize>,
  dep: HashMap<String, usize>
}

impl Plugin {
  pub fn new(path: &OsStr) -> Result<Plugin, Error> {
    unsafe {
      let library = Library::new(path)?;
      let mf = library.get::<PluginInitFn>("_orirocks_plugin_init".as_ref())?();
      // check that it's available
      library.get::<PluginDestroyFn>("_orirocks_plugin_destroy".as_ref())?;
      let destroy = || -> Result<_, _> {
        library.get::<PluginDestroyFn>("_orirocks_plugin_destroy".as_ref())?(mf);
        Ok::<_, Error>(())
      };
      if (*mf).version != PLUGIN_VERSION {
        destroy()?;
        return Err(Error::InvalidVersion(PLUGIN_VERSION, (*mf).version))
      }
      let mut env = HashMap::new();
      for i in 0..(*mf).environments.len {
        env.insert(marshal_string((*mf).environments.ptr.offset(i as isize).read().name), i as usize);
      }
      let mut dep = HashMap::new();
      for i in 0..(*mf).deployments.len {
        dep.insert(marshal_string((*mf).deployments.ptr.offset(i as isize).read().name), i as usize);
      }
      Ok(Plugin {
        library,
        mf,
        env: Default::default(),
        dep: Default::default()
      })
    }
  }

  pub fn create_environment(&self, name: &str, params: &HashMap<String, String>) -> Result<Environment, String> {
    unsafe {
      let params = marshal_params(params);
      let env_provider = (*self.mf).environments.ptr.offset(self.env[name] as isize).as_ref().unwrap();
      let mut env = null_mut();
      let err = (env_provider.create)(Array { ptr: params.as_ptr(), len: params.len() as u64 }, &mut env);
      marshal_err(err)?;
      Ok(Environment::from_raw(env_provider, env))
    }
  }

  pub fn deploy(&self, name: &str, path: &str, params: &HashMap<String, String>) -> Result<(), String> {
    unsafe {
      let params = marshal_params(params);
      let dep_provider = (*self.mf).deployments.ptr.offset(self.dep[name] as isize).as_ref().unwrap();
      let err = (dep_provider.deploy)(Array { ptr: path.as_ptr(), len: path.bytes().len() as u64 }, Array { ptr: params.as_ptr(), len: params.len() as u64 });
      marshal_err(err)?;
      Ok(())
    }
  }
}

impl Drop for Plugin {
  fn drop(&mut self) {
    unsafe {
      self.library.get::<PluginDestroyFn>("_orirocks_plugin_destroy".as_ref()).unwrap()(self.mf);
    }
  }
}

pub struct Environment<'a> {
  parent: *const EnvironmentProvider,
  me: *mut c_void,
  _lifetime: PhantomData<&'a ()>
}

impl<'a> Environment<'a> {
  unsafe fn from_raw(parent: *const EnvironmentProvider, me: *mut c_void) -> Environment<'a> {
    Environment {
      parent,
      me,
      _lifetime: Default::default()
    }
  }

  pub fn action(&mut self, name: &str, params: &HashMap<String, String>) -> Result<(), String> {
    unsafe {
      let params = marshal_params(params);
      let err = ((*self.parent).action)(
        self.me,
        Array { ptr: name.as_ptr(), len: name.bytes().len() as u64 },
        Array { ptr: params.as_ptr(), len: params.len() as u64 }
      );
      marshal_err(err)?;
      Ok(())
    }
  }

  pub fn finish(mut self, save_path: &str) -> Result<(), String> {
    unsafe {
      let err = ((*self.parent).finish)(self.me, Array { ptr: save_path.as_ptr(), len: save_path.bytes().len() as u64 });
      self.me = null_mut();
      marshal_err(err)?;
      Ok(())
    }
  }
}

impl<'a> Drop for Environment<'a> {
  fn drop(&mut self) {
    if self.me != null_mut() {
      panic!("Environment::finish() never called")
    }
  }
}

