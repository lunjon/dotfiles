# DotFiles

CLI tool for managing dotfiles in your home directory.

![dotf](https://user-images.githubusercontent.com/14161483/196793590-65b571ca-8d14-4d5e-9c64-cd686d816e98.png)

## Features

- It's fast
- Easy-to-use interface for managing your dotfiles
- Configuration file in flexible format using TOML
- Display diffs

## Installation

Currently you need to clone the repository and install with cargo:

```sh
$ cargo install --path .
```

## Usage

`dotf` uses `~/.config/dotfiles.toml`, called the _dotfile_, to manage dotfiles.

The dotfile must contain at least one section of files to track:
- `files`: arbitrary filepaths relative to home directory to track, such as `notes/*.md` to track all markdown files in `~/notes/` directory.
- `config`: known folder configuration directories:
  - Linux: `$XDG_CONFIG_HOME` or `$HOME/.config`
  - MacOS: `$HOME/Library/Application Support`
  - Windows: [`FOLDERID_LocalAppData`](https://learn.microsoft.com/sv-se/windows/win32/shell/knownfolderid?redirectedfrom=MSDN)

It is better demonstrated with an example.

```toml
# The path to the repository you wish to sync the files.
repository = "string"

# All following sections required the following types:
#  name = string | string[] | table

[files] # Assumes files are relative to your home directory.
vim = ".vimrc"                 # Simplest type, just a filepath
glob = "notes/**/*.txt"        # Glob pattern
list = [ ".zshrc", ".bashrc" ] # List of filepaths
# Table form (* required field):
#   files* ([string]): file paths to use
#   ignore ([string]): optional list of glob patterns to ignore
table = { files = ["scripts/*"], ignore = [ "*.out", ".cache" ] }

[config] # Files in standard configuration directory.
nvim = "nvim/**/*" # On linux this will typically be ~/.config/nvim/**/*
```

Example of sub-commands:
  - `dotf status`: see current status of files tracked
  - `dotf sync`: sync files between home and repository
  - `dotf edit`: edit the dotfile in your favorite editor
  - `dotf git`: run arbitrary git commands in the configured repository to sync files to

For more information use `dotf [sub-command] --help`.
