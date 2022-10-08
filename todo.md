## Fix

## Features
- [ ] Use crate `inquire` for prompts
- [ ] Ability to turn off backups via object notation
  ```toml
  myfile = { files = [".cool"], backup = false }
  ```
- [ ] `--commit/-C MESSAGE` option to `sync`
  - Creates a git commit after syncing the files
  - Only valid when syncing to repo
- [ ] `--push` option to `sync`
  - Pushes git commit after sync
  - Only valid with `--commit` option

## CI/CD
- [x] Add GH actions for building, etc.
- [ ] Add workflow for releases
