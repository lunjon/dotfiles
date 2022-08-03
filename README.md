# DotFiles

CLI tool for managing dotfiles in your home directory.

## Usage

`dotf` uses `~/dotfiles.y[a]ml`, called the _dotfile_, to manage dotfiles.
The dotfile must contain:

```yaml
# The path to the repository you wish to sync the files.
repository: string

# Files and directories that you wish to track.
# Files tracked must be limited to your home directory,
# which means all file paths must be relative to the dotfile.
files:
  - .vimrc              # Single file
  - '.config/nvim/**/*' # Glob pattern
  # For directory paths you can use an object
  # that includes/exludes by glob patterns.
  - path: '.config/httpcli/*'
    ignore: # Optional
      - '*.db'
```

When `~/dotfiles.y[a]ml`  exists, you can use the following sub-commands:
- `dotf sync`: sync files between home and repository
  - Use `dotf sync --home` to sync from repository to home
- `dotf status`: show the current status of the tracked files
- `dotf edit`: edit the dotfile
- `dotf git -- <...>`: run arbitrary git command in the repository

## Limitations
- Only files relative to home directory is supported
