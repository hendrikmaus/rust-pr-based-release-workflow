# Design of The Process

The abstract idea is to be able to create a pull-request, merge it, and that would automate the entire release process.

_Please mind that this entire project focusses on **Rust** only._

## Implementation

### Pull-Request

The pull-request change-set will be created and maintained **by hand**.

Like that, the process embraces maximum flexibility. You can also implement/use something that aids its creation and maintenance.

Typically, this includes:

- Changelog
- Version in `Cargo.toml`
- Documentation
  - e.g. Upgrade guide

### Triggering a Release

My current idea is to leverage the merge commit message to control the process.

In its simplest form, one could merge the release change-set with a message like:

```
chore(main): release 1.6.0
```

And the rest would be automated. Now what the "rest" really is, depends heavily on the project and its process.

### Technical Notes

- The GitHub Actions workflow runs on every merge to `main`
- It looks at the latest commit message, e.g. `chore(main): release 1.6.0`
  - or it uses `gh` to determine the pull-request by commit sha
	  to then inspect the pull-request for a specific label
- It can determine to release `1.6.0`
- Now it could:
  - Compile all binaries
  - Create Linux container image
  - Generate checksums for everything
  - Create a GitHub Release resource
    - Which also creates a `git tag`
    - With the changelog for a body
    - All release assets and checksums attached
  - Open pull-requests to other places, like a homebrew-tap
  - Publish to crates.io
  - Promote the release on the internet

This does not necessarily need to yield in a one-util-fit-all tool, because release processes across project are so opinioned.
