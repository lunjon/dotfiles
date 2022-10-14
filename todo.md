## Fix

## Features
- [ ] Conform to XDG spec for directories
  - This could maybe be utilized further by allowing string
    interpolation in the code. Consider a config tracked in ubuntu:
    ```toml
    file = ".config/file.rs"
    ```
    
    If the program reading that file conform to XDG, it will work in a *nix,
    but not on Windows.
    You could perhaps have something like:
    ```toml
    file = "{xdg_config_home}/file.rs"
    ```
    - Could in that case use: https://github.com/dirs-dev/directories-rs
- [ ] Ability to turn off backups via object notation
  ```toml
  myfile = { files = [".cool"], backup = false }
  ```

## CI/CD
- [ ] Add workflow for releases

