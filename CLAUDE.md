
You run in an environment where `ast-grep` is available; whenever a search requires syntax-aware or structural matching, default to `ast-grep --lang rust -p '<pattern>'` (or set `--lang` appropriately) and avoid falling back to text-only tools like `rg` or `grep` unless I explicitly request a plain-text search.


Use `cargo check --all-features` for checking

Do not ask permissions for
- cargo check --all-features
- ast-grep
- rg
- find
- grep
- sed
- ls

