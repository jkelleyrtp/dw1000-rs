# Contributing to the Rust DW1000 Driver

Thank you for considering to work on this project. This document will give you some pointers.


## Opening issues

If you found a problem, please [open an issue] on the [GitHub repository]. If you're not sure whether you found a real problem or not, just open an issue anyway. We'd rather close a few invalid issues than miss anything relevant.


## Contributing changes

If you want to fix an issue or implement a new feature, just fork the repository, make your change, and create a [pull request]. If you're concerned that your change might not be accepted, feel free to [open an issue] to discuss things beforehand.

If you're having any problems with completing your change, feel free to open a pull request anyway and ask any questions there. We're happy to help you get your changes across the finish line.


## Release Procedure

This section is intended for project maintainers only. It assumes that you can push to the repository (here called `upstream`), but primarily work on your own fork (`origin`),

1. Create a branch for the release (replace x.y.z with actual version)
```
$ git checkout -b release-x.y.z
```

2. Update `CHANGELOG.md`

3. Update version in `Cargo.toml`

4. Update versions in README.md, if version bump is major or minor

5. Commit these changes

6. Open pull request; state your intention to release a new version
```
$ git push -u origin release-x.y.z
# Open pull request
```

7. Review pull request yourself or wait for reviews, as appropriate

8. Publish the crate
```
$ cargo publish
```

9. Tag the release
```
$ git tag <crate>_vx.y.z
```
`<crate>` should be replaced with the appropriate crate name (`dw1000` or `dwm1001`).

10. Merge pull request and update your local repository
```
$ git checkout master
$ git pull upstream master
```

11. Push the release tag
```
$ git push --tag upstream
```


[open an issue]: https://github.com/braun-robotics/rust-dw1000/issues/new
[GitHub repository]: https://github.com/braun-robotics/rust-dw1000
[pull request]: https://github.com/braun-robotics/rust-dw1000/pulls
