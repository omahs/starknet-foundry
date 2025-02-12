# Scarb

[Scarb](https://docs.swmansion.com/scarb) is the package manager and build toolchain for Starknet ecosystem.
Those coming from Rust ecosystem will find Scarb very similar to [Cargo](https://doc.rust-lang.org/cargo/).

Starknet Foundry uses [Scarb](https://docs.swmansion.com/scarb) to:
- [manage dependencies](https://docs.swmansion.com/scarb/docs/reference/specifying-dependencies)
- [build contracts](https://docs.swmansion.com/scarb/docs/starknet/contract-target)\

One of the core concepts of Scarb is its [manifest file](https://docs.swmansion.com/scarb/docs/reference/manifest) - `Scarb.toml`.
It can be also used to provide [configuration](../projects/configuration.md) for Starknet Foundry.

Last but no least, remember that in order to use Starknet Foundry, you must have Scarb 
[installed](https://docs.swmansion.com/scarb/download) and added to the `PATH` environment variable. 
