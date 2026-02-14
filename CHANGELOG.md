# Changelog

## [0.10.0] - 2025-02-14

This release comes with mainly bug fixes and some internal restructuring.

### CLI

- The `-d/--djb2-only` setting was removed from the `katsuba op` subcommand.

### ObjectProperty

- Remove `djb2_only` from `SerializerOptions` as it's not needed anymore after the restructuring.

### Types

- A `hash` field is now required when deserializing a `TypeDef` object.
  - This should not be a big issue in practice since `wiztype` and `arrtype` already include this out of the box.
- Translation of v1 type lists to the v2 format no longer hash type names unconditionally with `string_id`.

### Python

- `SerializerOptions` no longer expose the removed `djb2_only` property.

## [0.9.0] - 2025-02-08

This release comes with a lot of internal restructuring and bugfixes.

### General

- `katsuba-bits` and `katsuba-executor` crates have been removed from the workspace.
- Most code has been migrated to a strict crate-level `#![forbid(unsafe_code)]` policy with `katsuba-wad` being the only exception.
  - Changed code has been rewritten to a safer alternative with similar performance characteristics.
  - If you find Katsuba's performance regressed with this release, please reach out on Discord.
- Katsuba has an official Discord server for support and questions now: https://discord.gg/Anagxv6NBX
- All file format parsing has migrated from `binrw` to custom minimal helper functions.

### CLI

- `katsuba-executor` has been retired. I'm not aware of any independent users.
  - For CPU-bound processing of files we use `rayon` now.
  - On Windows `CloseHandle` is notoriously slow due to Defender hooking into it. We now use `blocking` to offload file closing.
  - We are able to achieve good performance without memory pooling through mimalloc v3 and careful restructuring to save on allocations in the hot loops.
- For invalid combinations of input/output sources, we now throw an error instead of panicking.
- ObjectProperty deserialization preserves the state configuration passed to `katsuba op` per input when batch-processing files.
  - With special serializer configurations the state can be altered through the input data and won't be properly reset for the next file.
  - This has rarely been observed in practice because most users use it on `BINd` files which had special handling to correctly restore the state before.
- `wad unpack` now rejects zip bombs and refuses to extract files that escape out of the designated output directory.
- The `op guess` command has been retired. It was unreliable and rarely used.

### ObjectProperty

- A bug was fixed where missing `DELTA_IGNORE` properties were not properly skipped.
- Improved logging of file hashes for unknown types.
- Improved support for Pirate101 by adding support for reading `class Point<unsigned char` and `class Point<unsigned int>`.
- A test suite with various binaries was added to catch regressions better.
- `katsuba-bits` has been replaced by `bitter` which implements the same algorithm.

### Python

- The minimum Python version was bumped to Python 3.11.
- A regression was fixed where Linux x86 and aarch64 targets had no pre-built binary wheels available.
- `katsuba` 0.3.0 has been published to PyPI.

### WAD

- OOB reads during journal and CRC verification do not panic anymore.
  - This has previously been documented but was poor behavior considering we explicitly deal with bad input too.
- Memory-mapped archives no longer keep the file they were created from open.

## [0.8.6] - 2025-03-05

This is a bugfix release for ObjectProperty deserialization.

- Bitflag properties encoded as an empty string now decode correctly to value `0`.
- Skipping unknown server types can result in faulty OOB reads sometimes. This was resolved.

This affects both the CLI and the `katsuba` Python package.

## [0.8.5] - 2025-03-03

This release bumps Rust dependencies, fixes some bugs, and adds new features to the Python scripting API.

### CLI

- A bug was fixed in `wad unpack` where files larger than the fixed bucket sizes could not be extracted.

### Python

-  Documented type stubs were added for the entire API.
- `LazyObject.items()` is now supported as a means for iterating over the properties of an object.
- `TypeList.open_many()` opens multiple type list files and merge them into one object.
- `TypeList.name_for()` translates a ObjectProperty type hash to the type name string.

## [0.8.4] - 2024-03-07

This is mostly a bugfix release for some issues with the CLI.

- A `-f` argument was added to `wad pack` to customize archive flags in the WAD header.
  - This specifically helps with packing `Root.wad`s that a client will accept.
- #34 was fixed by introducing a proper error message for the described case.
- A race condition in `wad unpack` was fixed where files may be written to output directories while these are still being created in the threadpool, resulting in an obscure error message.
  - Unpacking will now wait for all directories to be created before submitting any file writes to the executor.

## [0.8.3] - 2024-02-01

This release adds support for the `wad pack` CLI command which allows creating a KIWAD archive from a given directory and ObjectProperty compatibility with Pirate101.

### CLI

- Support for `wad pack` was added.
- The `op` commands now accept a `-d/--djb2-only` flag for Pirate101 compatibility.
  - The usage is otherwise identical to Wizard101.

### ObjectProperty

- `SerializerOptions` was extended with `djb2_only` for Pirate101 compatibility.
- `Value::Object` will contain a compatible hash for Wizard101/Pirate101, depending on
  the previously set configuration flag.

### Python

- Adds a property for `djb2_only` to `SerializerOptions`.

### Types

`katsuba-types` was extended with the newly discovered `PropertyFlags` from Pirate101.

## [0.8.1] - 2023-12-16

This release contains stability improvements and fixes a regression in how WAD files are treated.

### CLI

* The `-c` flag was removed from `wad unpack` since the checks are now mandatory.
  * The performance impact from this is marginal thanks to SIMD-optimized CRC calculation.

### Executor

* Fixes OOM errors and overall slowness in batch-deserializing ObjectProperty state.
  * This issue was introduced with the switch to multi-threaded processing and is resolved now.

### WAD

* `File`s are now marked with a new `is_unpatched` attribute which is detected during CRC verification.

## [0.8.0] - 2023-11-02

This release combines months of work on improving existing components and adding requested features.

### General

- The project has been renamed to `katsuba`.
- The Python bindings are now available as `katsuba` on PyPI for all relevant x86, x86_64 and AArch64 platforms.
- All crates now return proper error types instead of `anyhow::Result`.

### Flake

- `nix build .#katsuba-py` can now be used to build the Python bindings.

### CLI

- A threadpool is now used for heavy I/O tasks. Performance gain has been measured on all tested platforms.
  - The environment variable `KATSUBA_WORKER_THREADS` can be used to tweak the amount of threads in the pool.
    - **WARNING:** More is not better and unless you know what you are doing just stick to the default configuration.
- Various subcommands now support logging at different verbosities using the `-v` option.
- Most commands now support glob patterns to process many input files at once.
- Reading input from stdin is now supported by using `-` in place of an input file path.
- A compact JSON representation will be emitted when `katsuba` output is piped into another application.

### ClientSig

- A new crate `katsuba-client-sig` was introduced.
- Support for reading and dumping `ClientSig.bin` files from the game was added.
  - This requires KingsIsle's private key which must be provided by the user.
- Appropriate CLI commands were added.

### BCD

- Parsing errors were fixed.
- Structure was changed to match the actual file representation closer.

### Python

- WAD archives now allow direct deserialization of ObjectProperty values without copying file contents between Rust and Python.
- ObjectProperty lists and objects now resolve their values lazily on access instead of converting the full object to a Python dict
  immediately after deserialization.
- All the compound leaf types like `Euler`, `Quaternion`, `Color` now have a Pythonic object representation and can be told apart
  from each other when encountered.
  - This has been possible before too but it required knowledge of the deserialized class layout since all these compounds were
    represented as tuples.
- `from katsuba.module import X` imports now work without raising an exception.
- The common KingsIsle hash functions from `katsuba-utils` are now accessible from Python.
- Support for NAV, POI, BCD has been preliminarily removed until we build a better object representation for them.
  - Demand for those has never been high so I don't think they will be missed at the time being.

### POI

- Parsing errors were fixed.
- Structure was changed to match the actual file representation closer.

### Types

- Support for [wiztype](https://github.com/wizspoil/wiztype) JSONs in all formats was added.
  - Katsuba automatically detects which one you supply.

### ObjectProperty

- Support for `CoreObject` deserialization has been removed because it is too inconsistent and gets outdated.
  - Users are encouraged to deserialize via library usage if this functionality is needed.
- `Value` has been shrunk to a size of 32 bytes.
- `Value::Object` now provides the type hash of the object so library users have access to this info.
- One reusable `Serializer` instance now provides all functionality.
- Support for guessing configuration based on an object's data was added.
  - This does not reliably detect every case but it's a good enough starting point for reversing unknown objects.
  -  An appropriate CLI command has been added.

### WAD

- Archives now report their UNIX file permissions, if available.
  - The extractor uses this info to create files with the same permissions as the archive they originate from.
- `GlobIter` has been added to iterate over a subset of an archive's files given a glob pattern.

## [0.7.1] - 2023-05-15

This release of Kobold mainly consists of bugfixes and UX improvements:

### ObjectProperty

- Fixes deserialization of enums that do not have any of the typical enum flag bits set
- Correctly reads the delta encode bit only in shallow mode
- Forbid skipping objects in shallow serialization mode (which doesn't make much sense anyway)

### WAD

- Adds support for UNIX glob patterns as file paths to unpack, i.e. `Data/GameData/*.wad`
- Allows multiple paths at once to be passed to the `unpack` command
- Properly ignores unpatched (all zeroes) files during unpacking

## [0.6.0] - 2023-05-13

This release of Kobold features features a better strategy for handling ignored types (-i flag) than the previous release.

## [0.5.0] - 2023-05-11

This release of Kobold features a lot of changes:

### Wad

- Fixes archive header deserialization
- Renders a progress bar (so users don't think it's stuck for big archives)

### ObjectProperty

- `DeserializerOptions::skip_unknown_types` field to ignore types with unknown hashes
- Fixes deserialization of negative `enum_options` integer values in the type list jsons
- Various bug fixes to overall ObjectProperty deserialization logic

### CLI

- Adds `bind` as a designated deserialization mode which configures correct `DeserializerOptions` for convenience
- `-i` flag for complementing the support for skipping unknown types

### Python bindings

- Define serializer flags integer constants

## [0.2.0] - 2022-12-21

This release of the CLI improves the JSON representation of std::string and std::wstring when deserializing ObjectProperty state.

## [0.1.0] - 2022-12-21

This marks the initial release of kobold v0.1.0 with support for deserializing and extracting various file formats from CLI.
