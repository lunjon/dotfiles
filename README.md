# DotFiles

CLI tool for managing dotfiles in your home directory.

![dotfiles](https://user-images.githubusercontent.com/14161483/186741239-8e8f26d5-51f9-4ab5-a6d5-09a639334e74.png)

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
- files: arbitrary filepaths to track, such as `notes/*.md` to track all markdown files in `~/notes/` directory.
- builtin: has support for known configurations, such as a git and nvim.

It is better demonstrated with an example.

```toml
# The path to the repository you wish to sync the files.
repository = "string"

[files]
# Files relative to your home directory that you wish to track.
# Formula:
#  name = string | string[] | table
# Examples:
vim = ".vimrc"                 # Simplest type, just a filepath
glob = "notes/**/*.txt"        # Glob pattern
list = [ ".zshrc", ".bashrc" ] # List of filepaths
# Table form (* required field):
#   files* ([string]): file paths to use
#   ignore ([string]): optional list of glob patterns to ignore
table = { files = ["scripts/*"], ignore = [ "*.out", ".cache" ] }

[builtin]
# An easier way to track known configurations (see list below).
# Formula:
#   name = bool | table
git = true                      # Simplest form, just track your git configuration file at ~/.gitconfig
nvim = {}                       # Table form, empty uses all defaults
helix = { ignore = ["*.temp"] } # Table form with ignore
```

Example of sub-commands:
  - `dotf status`: see current status of files tracked
  - `dotf sync`: sync files between home and repository
  - `dotf edit`: edit the dotfile in your favorite editor
  - `dotf git`: run arbitrary git commands in the configured repository to sync files to

For more information use `dotf [sub-command] --help`.

## Built-in configuration options
The section called `[build]` can be used to track known configurations.

List of supported programs:
- Git
- Tmux
- [Helix](https://github.com/helix-editor/helix)
