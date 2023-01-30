# orirocks

Packer is great, but it leaves a lot to be desired. For example:
- The tasks of image preperation and uploading the image to various registries
  is intertwined.
- Packer seems to be oriented around building images using hosting providers' build
  services.
- If the plugin you're using doesn't provide a custom communicator, you must use
  the SSH communicator, which can be problematic.

Orirocks is an alternative to Packer that:
- Builds completely offline
- Seperates deployment and building
- Can be extended with plugins
- Allows easy reuse of intermediate artifacts