<!-- omit in toc -->
# Changelog

Build a changelog based on commits with [Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/) format.

- [Todo](#todo)

## Todo

- [x] Check for repository
- [x] Get all commits
    - [ ] Get all commits from last tag
    - [x] Get all commits from datetime of changelog
- [x] Build changelog based on conventional commits
- [x] Build changelog not based on conventional commits
- [ ] Make it configurable per repository (with checkin maybe?)
    - [ ] Configuration for "from tag" or "from changelog.md"
    - [ ] Configuration for "conventional" or "non-conventional"
- [x] Set Version in changelog
    - [ ] Calculate using what?
        - [ ] Several implementations to read:
            - [ ] package.json
            - [ ] *.nuspec
            - [ ] *.psd1
            - [ ] *.toml
