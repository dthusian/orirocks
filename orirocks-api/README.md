# orirocks-api

The plugin system for Orirocks.

Plugins are dynamically-linked libraries (.so or .dll) which export two functions:

```
struct PluginManifest* _orirocks_plugin_init();
void _orirocks_plugin_destroy(struct PluginManifest*);
```

All struct definitions can be found in `src/ffi.rs`.