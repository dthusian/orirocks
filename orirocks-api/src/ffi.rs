use std::ffi::c_void;

/// A fixed-length array. It is valid in some contexts for `ptr` to be null.
/// However, if not specified, `Array`s must not be null.
#[repr(C)]
pub struct Array<T> {
  pub ptr: *const T,
  pub len: u64
}

#[repr(C)]
pub struct Param {
  pub key: Array<u8>,
  pub value: Array<u8>
}

/// Exported under `_orirocks_plugin_init`.
/// Returns the manifest for this plugin. The pointer returned must
/// live until it is passed to PluginDestroyFn
/// ### Thread Safety
/// No guarantees are required. This function will not be called concurrently.
pub type PluginInitFn = unsafe extern "C" fn() -> *const PluginManifest;

/// Exported by plugins under `_orirocks_plugin_destroy`
/// Frees the memory associated with the given plugin manifest.
/// ### Thread Safety
/// No guarantees are required. This function will not be called concurrently.
pub type PluginDestroyFn = unsafe extern "C" fn(*const PluginManifest);

#[repr(C)]
pub struct PluginManifest {
  pub version: u32,
  pub name: Array<u8>,
  pub environments: Array<EnvironmentProvider>,
  pub deployments: Array<DeploymentProvider>
}

#[repr(C)]
pub struct HostAPIFunctions {
  /// Resolves a <prefix>:<path> location to a real filepath or URL
  /// ### Lifetime Guarantees
  /// The returned array is valid until passed to `free_memory`.
  /// ### Thread Safety
  /// This function may be called concurrently.
  pub resolve_location: unsafe extern "C" fn(Array<u8>) -> Array<u8>,

  /// Frees memory allocated by any function in this list.
  /// The array must have been acquired from a previous call to another
  /// function in the `HostAPIFunctions` struct.
  /// ### Thread Safety
  /// This function may be called concurrently.
  pub free_memory: unsafe extern "C" fn(Array<u8>)
}

#[repr(C)]
pub struct EnvironmentProvider {
  /// UTF-8 string describing the name of the environment.
  /// ### Lifetime Guarantees
  /// Must live as long as owning PluginManifest object.
  pub name: Array<u8>,

  /// Creates a new environment, outputting an opaque pointer to it.
  /// Returns an error message if the call failed or a null array otherwise.
  /// ### Lifetime Guarantees
  /// The pointer to the environment remains valid until it is passed to `EnvironmentProvider::finish`.
  /// The returned error message, if present, must be valid for 'static.
  /// ### Thread Safety
  /// This function needs to be callable on multiple threads non-concurrently.
  pub create: unsafe extern "C" fn(Array<Param>, *mut *mut c_void /* env */) -> Array<u8>,

  /// Performs an action on the environment.
  /// Returns an error message if the call failed or a null array otherwise.
  /// ### Lifetime Guarantees
  /// The returned error message, if present, must be valid for 'static.
  /// ### Thread Safety
  /// This function needs to be callable on multiple threads concurrently. Concurrent invocations will not have the same `env` parameter.
  pub action: unsafe extern "C" fn(*mut c_void /* env */, Array<u8> /* name */, Array<Param>) -> Array<u8>,

  /// Destroys an environment, requesting that the result image be saved to a certain path.
  /// After calling this function, the environment referred to by `env` is no longer valid.
  /// Returns an error message if the call failed or a null array otherwise.
  /// ### Thread Safety
  /// This function needs to be callable on multiple threads concurrently. Concurrent invocations will not have the same `env` parameter.
  pub finish: unsafe extern "C" fn(*mut c_void /* env */, Array<u8> /* path */) -> Array<u8>
}

#[repr(C)]
pub struct DeploymentProvider {
  /// UTF-8 string describing the name of the environment.
  /// ### Lifetime Guarantees
  /// Must live as long as owning PluginManifest object.
  pub name: Array<u8>,

  /// Performs a deployment.
  /// ### Lifetime Guarantees
  /// The returned error message, if present, must be valid for 'static.
  /// ### Thread Safety
  /// This function needs to be callable on multiple threads concurrently.
  pub deploy: unsafe extern "C" fn(Array<u8> /* path */, Array<Param>) -> Array<u8>
}