# Logging

The logging API in WutEngine is meant to be as simple as possible. There are five log levels, each accessible through its respective macro (See the API docs for `wutengine::log` for the actual API).

The engine runtime has two main logging outputs for native applications:

1. Straight into the terminal that hosts the runtime
2. To a file per subsystem:

   | OS      | Log directory                                                                             |
   | ------- | ----------------------------------------------------------------------------------------- |
   | Linux   | `$XDG_DATA_HOME/<org>/<app>/logs/`, or `~/.local/share/<org>/<app>/logs/` if it isn't set |
   | macOS   | `~/Library/Application Support/<org>/<app>/logs/`                                         |
   | Windows | `%LOCALAPPDATA%\<org>\<app>\logs\`                                                        |

In the future, WutEngine will allow for configuring verbosity levels at runtime and through the config.
