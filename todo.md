## Fix
- [ ] When running `sync` with `--diff` it tries to create a diff for files that doesn't exists.
  - Example: sync from home to repo gives an error if the file doesn't exist in repo

## Features
- [ ] Use crate `inquire` for prompts
- [?] Sub-command for removing files from repo
  - For instance, if a file is removed in home that is no longer wanted,
    it recides in the repo.
- [?] Display files nested if coming from same entry, e.g. a glob pattern which lists multiple files.

## CI/CD
- [x] Add GH actions for building, etc.
- [ ] Add workflow for releases
