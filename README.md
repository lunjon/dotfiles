# DotFiles

CLI tool for managing dotfiles in your home directory.

## Usage

`dotf` uses `~/dotfiles.y[a]ml` to manage dotfiles, which must contain:

```yaml
# The path to the repository you wish to put the files
repository: string

# Files and directories relative to this file that you wish to track
files: [string, ...]
```

When `~/dotfiles.y[a]ml`  exists, you can use the following sub-commands:
- `dotf sync`: sync files between home and repository
- `dotf status`: show the current status of the tracked files
- `dotf edit`: edit the file
- `dotf git -- <...>`: run arbitrary git command in the repository

## Limitations

- Only files relative to home directory is supported
