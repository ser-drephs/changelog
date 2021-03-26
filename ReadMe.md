<!-- omit in toc -->
# Changelog

Build a changelog based on commits with [Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/) format.

- [Todo](#todo)
- [Notes](#notes)

## Todo

- [x] Check for repository
- [ ] Get all commits
    - [ ] Get all commits from last tag
    - [ ] Get all commits from datetime of changelog
- [ ] Build changelog based on conventional commits
- [ ] Build changelog not based on conventional commits
- [ ] Make it configurable per repository (with checkin maybe?)
    - [X] Configuration for "from tag" or "from changelog.md"
    - [ ] Configuration for "conventional" or "non-conventional"
- [ ] Set Version in changelog
    - [ ] Ask user before building changelog
    - [ ] Calculate using what?
        - [ ] Several implementations to read:
            - [ ] package.json
            - [ ] *.nuspec
            - [ ] *.psd1
            - [ ] *.toml


## Notes

`'a` Defines the lifetime of an object. So that if one object with lifetime `'a` is removed, all child object with `'a` are removed.
Ex:
```rust
pub struct LogWalker<'a> {
    repo: &'a Repository,
    revwalk: Option<Revwalk<'a>>,
}

impl<'a> LogWalker<'a> { }
```

Functions return value is the last line of a function without an `;` at the end.

Using handlebars syntax for `diff_format`. `{{latest}}` and `{{base}}` mus be defined.