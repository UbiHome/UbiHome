# Logger

```yaml title="Example configuration entry"
logger:
  level: INFO
  directory: ./right-here
  logs:
    libmdns: DEBUG
```

| Log Level | Description                                                      |
| --------- | ---------------------------------------------------------------- |
| ERROR     | Only log very serious errors.                                    |
| WARNING   | Log warnings and errors.                                         |
| INFO      | Log informational messages, warnings and errors.                 |
| DEBUG     | Log debug messages, informational messages, warnings and errors. |
| TRACE     | Log all messages, including debug and trace messages.            |

> From Rust [LogLevels](https://docs.rs/log/latest/log/enum.Level.html);

## Default Logging Directory

| OS      | Directory                                    | Example Path |
| ------- | -------------------------------------------- |-- |
| Linux   | `$XDG_DATA_HOME/ubihome/` or `$HOME/.local/share/ubihome/` | `/home/alice/.local/share/ubihome/logs` |
| Windows | `{FOLDERID_LocalAppData}\ubihome\` | `C:\Users\Alice\AppData\Local\ubihome\` |
| MacOS   | `$HOME/Library/Application Support/ubihome/` | `/Users/Alice/Library/Application Support/ubihome/` |

> From Rust [Directories](https://docs.rs/directories/latest/directories/struct.BaseDirs.html#method.data_local_dir)