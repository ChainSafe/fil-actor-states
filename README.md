# State-only versions of the Filecoin Actors (smart-contracts)

[![GitHub Workflow Status](https://img.shields.io/github/actions/workflow/status/ChainSafe/fil-actor-states/tests.yml?style=for-the-badge)](https://github.com/ChainSafe/fil-actor-states/actions)

These crates may be used to inspect and validate the internal state of Filecoin Actors.

For the smart-contracts, see: https://github.com/filecoin-project/builtin-actors  
NOTE: The code in `actors/` and `fil_actors_shared/` is copy-pasted from the above repository. 
Therefore, it's advised to avoid refactoring such code in order to facilitate a smoother version 
upgrade process and potentially automate it. `fil_actor_interface/` can be modified according to
project's needs.
