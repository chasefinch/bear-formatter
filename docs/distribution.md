# Distribution

bear-formatter ships as a single binary through a personal Homebrew tap:
[`chasefinch/homebrew-tap`](https://github.com/chasefinch/homebrew-tap). The
canonical formula lives here in `homebrew/bear-formatter.rb`; the tap holds a
copy that is updated on release.

## Install

```bash
brew install chasefinch/tap/bear-formatter
```

## Cutting a release

1. Bump `version` in `Cargo.toml`; commit.
2. Tag and push:
   ```bash
   git tag v0.1.0
   git push origin v0.1.0
   ```
3. GitHub auto-generates the source tarball. Grab its checksum:
   ```bash
   curl -sL https://github.com/chasefinch/bear-formatter/archive/refs/tags/v0.1.0.tar.gz | shasum -a 256
   ```
4. In `homebrew/bear-formatter.rb`, update the `url`/`sha256` to the new version
   and checksum. Copy the formula to the tap (`chasefinch/homebrew-tap`,
   `Formula/bear-formatter.rb`) and push.
5. Update the coverage badge in `README.md` from `make coverage` (the TOTAL line %).
6. `brew update && brew upgrade bear-formatter` installs the tagged build.

## Bottles (later)

Once installs are frequent enough to want to skip the source build, add
pre-built bottles: `brew install --build-bottle`, `brew bottle`, upload the
artifacts to the release, and add the resulting `bottle do` block to the
formula.

## Notes

- The build compiles from source (`cargo install`) including a bundled SQLite,
  so `depends_on "rust" => :build` and a C toolchain (Xcode CLT) are required.
- The `axioms` submodule is docs only and isn't needed to build.
