> [!WARNING]
> This project is a work in progress, so there may be features missing or not working
> correctly. Additionally, breaking changes may (and probably will) occur. YOU HAVE BEEN
> WARNED.

# ProMan - a TUI project creator

This is a simple and configurable, terminal-based application to create new programming
projects for a multitude of languages.

## Plugins

The application utilises the RON file format to
specify and create plugins. Default plugins are available in the `default-plugins`
directory and are built into the application during compile-time, so the files do not
need to be present on the user's file system. Plugins can be placed in the user's plugin
directory on the filesystem, to enable them to be used by `proman`.

| Operating System | Directory                                                                                      |
|:----------------:|:-----------------------------------------------------------------------------------------------|
|      Linux       | `$HOME/.config/proman/plugins`                                                                 |
|     Windows      | `$APPDATA\stboyden\proman\plugins\` (PowerShell)<br/>`%APPDATA%\stboyden\proman\plugins` (CMD) |
|      MacOS       | `$HOME/Library/Application Support/com.stboyden.proman/`                                       |

If you're running the application from source, and in debug mode, then the directory for
plugins will be `plugins/` relative to the root of the project.

## Licensing

This project can be licensed in either:

- MIT License
- Apache 2.0 License

At your discretion.