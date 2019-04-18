# DotFiles

CLI tool for managing dotfiles in your home directory.

## Usage

`dotfiles` command used `~/dotfiles.y[a]ml` to manage dotfiles. The file must contain:

```yaml
# The path to the repository you wish to put the files
repository: string
# Files and directories relative to this file that you wish to track
files: [string, ...]
```
