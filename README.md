# DotFiles

CLI tool for managing configuration files.

![dotf](https://user-images.githubusercontent.com/14161483/196793590-65b571ca-8d14-4d5e-9c64-cd686d816e98.png)

## Features

- Fast and easy tracking of configuration files
- Sync files between local file system and repository
- Configuration file in flexible format using TOML
- And more

## Installation

Clone the repository and install with cargo:

```sh
$ cargo install --path . --locked
```

This will install a binary called `dotf`.

## Usage

`dotf` uses `~/.config/dotfiles.toml`, called the _dotfile_, to manage dotfiles.

The dotfile must contain at least one section of files to track:
- `home`: filepaths relative to home directory to track, such as `notes/*.md` to track all markdown files in `~/notes/` directory.
- `config`: known folder configuration directories:
  - Linux: `$XDG_CONFIG_HOME` or `$HOME/.config`
  - MacOS: `$HOME/Library/Application Support`
  - Windows: [`FOLDERID_LocalAppData`](https://learn.microsoft.com/sv-se/windows/win32/shell/knownfolderid?redirectedfrom=MSDN)

It is better demonstrated with an example.

```toml
# The path to the repository you wish to sync the files to.
# This is required.
repository = "string"

# All following sections support the following types:
#  name = string | [string] | table

# Files that are relative to your home directory.
[home]
vim = ".vimrc"                 # Simplest type, just a filepath
glob = "notes/**/*.txt"        # Glob pattern
list = [ ".zshrc", ".bashrc" ] # List of filepaths
# Table form:
#   files* ([string]): file paths to use
#   ignore ([string]): optional list of glob patterns to ignore
table = { files = ["scripts/*"], ignore = [ "*.out", ".cache" ] }

# Files in standard configuration directory.
# On linux this will typically be ~/.config/nvim/**/*
[config]
nvim = "nvim/**/*"
```

\* Required field.

With a dotfile you can now use the `dotf` command:
  - `dotf status`: see current status of files tracked
  - `dotf sync`: sync files between home and repository
  - `dotf edit`: edit the dotfile in your favorite editor
  - `dotf git`: run arbitrary git commands in the configured repository to sync files to

For more information use `dotf --help`.
