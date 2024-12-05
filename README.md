# State-only versions of the Filecoin Actors (smart-contracts)

[![GitHub Workflow Status](https://img.shields.io/github/actions/workflow/status/ChainSafe/fil-actor-states/tests.yml?style=for-the-badge)](https://github.com/ChainSafe/fil-actor-states/actions)

These crates may be used to inspect and validate the internal state of Filecoin
Actors.

For the smart-contracts, see:
https://github.com/filecoin-project/builtin-actors  
NOTE: The code in `actors/` and `fil_actors_shared/` is copy-pasted from the
above repository. Therefore, it's advised to avoid refactoring such code in
order to facilitate a smoother version upgrade process and potentially automate
it.

# Why the copy-paste?

Given the copy-paste nature of the code, it's important to understand the
reasons behind it, as it's not obvious at first glance.

## Maintenance

We copy only a small subset of the code, strictly around actor releases which
happen ~3 times a year. This is not a huge burden to maintain.

Note that the approach here is similar to the one in
[go-state-types](https://github.com/filecoin-project/go-state-types).

## Versioning

The [builtin-actors](https://github.com/filecoin-project/builtin-actors) are not
following semver and in the current structure it would be hard to do so.
Dependency update in a particular actor version might require a major version
bump. Dependencies of the actors code not unified and so actors `vX` might use
completely different version of `cid` than actors `vX+1`. This creates
compilation and linking issues in the downstream projects that depend on the
actors code.

## Security

In the [builtin-actors](https://github.com/filecoin-project/builtin-actors), the
prior versions of the actors are not maintained. Any security issues found in
the actors code are fixed in the latest version and the older versions are not
updated. This creates a security risk for the projects that depend on the actors
code (and not the smart-contracts). By copying the code, the projects can
maintain the security of the actors code by updating the code to the latest
version.

## Tradeoff

Given the current approach, the code that exists here might not be exactly the
same as the one released in the WASM bundles. This shouldn't be an issue, as
most of it is not used for calculations but as state definitions.

## Alternative approaches

The existence of most of the code in the `actors/` and `fil_actors_shared/`
directories could be avoided by having different versioning in the
[builtin-actors](https://github.com/filecoin-project/builtin-actors). Each actor
release, e.g., `actors_v14`, would be a separate directory in the repository. As
such, it would allow the release to follow semver and keep the dependencies
across all actors up-to-date. On the other hand, the code would still need to be
copied on new releases so it's shifting the problem rather than solving it. That
said, it's a valid approach and could be considered in the future.
