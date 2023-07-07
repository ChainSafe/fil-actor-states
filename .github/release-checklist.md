# Release checklist 🛂

`fil-actor-states` doesn't follow a fixed schedule but releases are be expected with every network upgrade. 
A _release officer_ is volunteered for each release, and they are responsible for either following the checklist 
or, in case of absence, passing the task to a different team member.

## Prepare the release

Make a pull request with the following changes:

- Update the version of all the crates to be released.
- Make sure to include the `Cargo.lock` crate version change in the release.
- Tag `Release` label to this pull request, this enables additional forest sync check test to
  ensure no breaking changes are released.

## Release on crates.io

- Create a version tag using `git tag -a <version> -m "Tag message"`
  Replace `<version>` with the actual version number or tag name you want to create,
  and "Tag message" with a brief description of the tag (e.g., "v1.0.0 - Initial release").
- Push the newly created version tag to GitHub using `git push origin <version>` (e.g., git push origin v1.0.0)
- Once the push is successful, all `fil-actor-states` crates will be build and published to [crates.io](https://crates.io/).
