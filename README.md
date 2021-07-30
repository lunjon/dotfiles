# DotFiles

CLI tool for managing dotfiles in your home directory.

## Usage

`dotfiles` uses `~/dotfiles.y[a]ml` to manage dotfiles, which must contain:

```yaml
# The path to the repository you wish to put the files
repository: string

# Files and directories relative to this file that you wish to track
files: [string, ...]
```

When that file exists, you can use the following sub-commands:
- `sync`: sync files between home and repository
- `status`: show the current status of the tracked files
- `edit`: edit the file

## Limitations

- Only files relative to home directory is supported
