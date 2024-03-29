# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [v0.2.1] - 2024-03-29
### :bug: Bug Fixes
- [`c4348a4`](https://github.com/JakeStanger/mpd-utils-rs/commit/c4348a4d7c3653b478e5be7fd3fe347ea0961f39) - **persistent client**: using new receiver for every recv call *(commit by [@JakeStanger](https://github.com/JakeStanger))*
- [`fd84f18`](https://github.com/JakeStanger/mpd-utils-rs/commit/fd84f1806fdda6a642f55710a6c0a0274e40f156) - **persistent client**: use much larger event channel buffer *(commit by [@JakeStanger](https://github.com/JakeStanger))*

### :recycle: Refactors
- [`bede833`](https://github.com/JakeStanger/mpd-utils-rs/commit/bede833f82547c52019c3ea10fbd3a5c86ac8edc) - **persistent client**: use rwlock to reduce chance of deadlock *(commit by [@JakeStanger](https://github.com/JakeStanger))*

### :memo: Documentation Changes
- [`c1b7421`](https://github.com/JakeStanger/mpd-utils-rs/commit/c1b74214d8794376fd830b21ac1aca6e66767cae) - update CHANGELOG.md for v0.2.0 [skip ci] *(commit by [@JakeStanger](https://github.com/JakeStanger))*


## [v0.2.0] - 2024-01-08
### :boom: BREAKING CHANGES
- due to [`c2fc743`](https://github.com/JakeStanger/mpd-utils-rs/commit/c2fc74327b1912c06a64e7d09bfea9061cd4843f) - change host to owned string *(commit by [@JakeStanger](https://github.com/JakeStanger))*:

  `host` input (and `hosts` for multi-host client) is now an owned `String` as this was causing issues with consumers, and as it clones the string internally anyway.


### :sparkles: New Features
- [`43bb2f2`](https://github.com/JakeStanger/mpd-utils-rs/commit/43bb2f2afed66cfefef234c261afa2cb44ba0abd) - add `mpd_client` re-export *(commit by [@JakeStanger](https://github.com/JakeStanger))*
- [`46aa5f4`](https://github.com/JakeStanger/mpd-utils-rs/commit/46aa5f4e6e81035f97aa35946b98549a1f7c564e) - **persistent client**: subscribe method *(commit by [@JakeStanger](https://github.com/JakeStanger))*
- [`7226c31`](https://github.com/JakeStanger/mpd-utils-rs/commit/7226c31b350064e16518636f385ace749f0ea115) - **persistent client**: `command` method *(commit by [@JakeStanger](https://github.com/JakeStanger))*

### :recycle: Refactors
- [`c2fc743`](https://github.com/JakeStanger/mpd-utils-rs/commit/c2fc74327b1912c06a64e7d09bfea9061cd4843f) - change host to owned string *(commit by [@JakeStanger](https://github.com/JakeStanger))*
- [`8d5d406`](https://github.com/JakeStanger/mpd-utils-rs/commit/8d5d40606c3d3f30d10e402d67d012421d779d6a) - replace channels with tokio broadcast channels *(commit by [@JakeStanger](https://github.com/JakeStanger))*

### :memo: Documentation Changes
- [`ba29890`](https://github.com/JakeStanger/mpd-utils-rs/commit/ba29890b2a955f9c7ea97976a5121b78ccb73a23) - **readme**: fix crate link *(commit by [@JakeStanger](https://github.com/JakeStanger))*


[v0.2.0]: https://github.com/JakeStanger/mpd-utils-rs/compare/v0.1.0...v0.2.0
[v0.2.1]: https://github.com/JakeStanger/mpd-utils-rs/compare/v0.2.0...v0.2.1