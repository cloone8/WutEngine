# Logging

The logging API in WutEngine is meant to be as simple as possible. There are five log levels, each accessible through its respective macro (See the API docs for `wutengine::log` for the actual API).
From lowest to highest severity: `trace`, `debug`, `info`, `warning`, `error`. Currently, release builds show `info` and up, and debug builds show `debug` and up. In the future, WutEngine will allow for configuring verbosity levels at runtime and through the config.

## Native

The engine runtime has two main logging outputs for native applications:

1. Straight into the terminal that hosts the runtime
2. To a file per subsystem, keyed on the organization and application name given through the runtime startup options (see API docs for `wutengine::runtime::StartupOptions`):

   | OS      | Log directory                                                                             |
   | ------- | ----------------------------------------------------------------------------------------- |
   | Linux   | `$XDG_DATA_HOME/<org>/<app>/logs/`, or `~/.local/share/<org>/<app>/logs/` if it isn't set |
   | macOS   | `~/Library/Application Support/<org>/<app>/logs/`                                         |
   | Windows | `%LOCALAPPDATA%\<org>\<app>\logs\`                                                        |

Currently, there is one log file (`engine.log`), with plans for a file per subsystem. If the log file already exists when the application is started, the existing log is rotated to `*_prev.log` (overwriting the current previous log).

## Web

On the web, the log subsystem forwards its logs to the browser/host console through the JavaScript `console` API.
