# DotFiles

CLI tool for managing dotfiles in your home directory.

## Usage

`dotf` uses `~/.config/dotfiles.toml`, called the _dotfile_, to manage dotfiles.
The dotfile must contain:

```toml
# The path to the repository you wish to sync the files.
repository = "string"

[files]
# Files relative to your home directory that you wish to track.
# Examples:
vim = ".vimrc" # type string
glob = "notes/**/*.txt" # string with glob pattern
list = [ ".zshrc", ".bashrc" ] # list of strings
object = {{ path = "scripts/*", ignore = [ "*.out", ".cache" ] }}"#,
```

When `~/dotfiles.y[a]ml`  exists, you can use the following sub-commands:
- `dotf sync`: sync files between home and repository
  - Use `dotf sync --home` to sync from repository to home
- `dotf status`: show the current status of the tracked files
- `dotf edit`: edit the dotfile
- `dotf git -- <...>`: run arbitrary git command in the repository
