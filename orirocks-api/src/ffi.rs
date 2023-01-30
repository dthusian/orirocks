use std::ffi::c_void;

#[repr(C)]
pub struct Array<T> {
  ptr: *const T,
  len: u64
}

#[repr(C)]
pub struct PluginManifest {
  name: Array<u8>,
  environments: Array<EnvironmentProvider>,
  deployments: Array<DeploymentProvider>
}

#[repr(C)]
pub struct EnvironmentProvider {
  create: unsafe extern "C" fn(Array<Param>, *mut *mut c_void /* env */) -> i32,
  action: unsafe extern "C" fn(*mut c_void /* env */, Array<u8> /* name */, Array<Param>) -> i32,
  finish: unsafe extern "C" fn(*mut c_void /* env */, Array<u8> /* path */) -> i32
}

#[repr(C)]
pub struct DeploymentProvider {
  deploy: unsafe extern "C" fn(Array<u8> /* path */, Array<Param>) -> i32
}

#[repr(C)]
pub struct Param {
  key: Array<u8>,
  value: Array<u8>
}