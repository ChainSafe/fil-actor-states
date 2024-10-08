# Release checklist 🛂

`fil-actor-states` doesn't follow a fixed schedule but releases are be expected with every network upgrade.
A _release officer_ is volunteered for each release, and they are responsible for either following the checklist 
or, in case of absence, passing the task to a different team member.

## Prepare the release

Make a pull request with the following changes:

- Update the version of all the crates to be released.
- Tag `Release` label to this pull request, this enables additional forest sync check test to
  ensure no breaking changes are released.

## Release on GitHub

- Create a [new release][1]. Click on `Choose a tag` button and create a new
  one. The tag must start with a lowercase `v`, e.g., `v10.0.0`. Follow the
  title convention of the previous releases, and write a small summary of the
  release.

## Release on crates.io

- `fil-actor-states` crates will be automatically built and published on a successful GH release.

  [1]: https://github.com/ChainSafe/fil-actor-states/releases/new
