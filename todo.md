## Fix

## Features
- [ ] Use crate `inquire` for prompts
- [?] Sub-command for removing files from repo
  - For instance, if a file is removed in home that is no longer wanted, it stays in the repo.
  - Name? clean, nuke, etc.
- [?] Display files nested if coming from same entry, e.g. a glob pattern which lists multiple files.
- [ ] Ability to turn off backups via object notation
  ```toml
  myfile = { path = ".cool", backup = false }
  ```

## CI/CD
- [x] Add GH actions for building, etc.
- [ ] Add workflow for releases
