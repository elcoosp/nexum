# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).
## [0.3.0] - 2026-11-11

### Bug Fixes
- gate unix paths and test handlers to target_os unix by @elcoosp ([3c0d135](https://github.com/elcoosp/nexum/commit/3c0d13520be8ccd21b0d6ab43b6175468638648b))
- add formatting separators to hexadecimal constants by @elcoosp ([5af9279](https://github.com/elcoosp/nexum/commit/5af92799c63003571536c7e3243e616f18500ded))

### Documentation
- replace absolute paths with short local crate macros by @elcoosp ([0206ff0](https://github.com/elcoosp/nexum/commit/0206ff06cc3002e94510f2ad848f6d92108f66de))
- document single-instance, plist, mobile, and bounded hubs by @elcoosp ([8274673](https://github.com/elcoosp/nexum/commit/827467390103f1b13ac96796a284c562712e4ce4))
- add nexum-platform readme by @elcoosp ([295b1a5](https://github.com/elcoosp/nexum/commit/295b1a5bc2e76a0ca99f6bd258df103988d2f8fd))

### Features
- add single-instance hand-off over a local socket by @elcoosp ([419585f](https://github.com/elcoosp/nexum/commit/419585f0116f81ae5086fd041281dce05dc7acd6))
- add Info.plist URL type generator and validator by @elcoosp ([1fde50b](https://github.com/elcoosp/nexum/commit/1fde50b0bb93a717d5b20027819c60ccc6d95287))
- add Android and iOS deep-link intake and generators by @elcoosp ([0d4cfe9](https://github.com/elcoosp/nexum/commit/0d4cfe9e73e74a248a65eee62e49e1609499bf17))
- validate the bundle Info.plist on macOS by @elcoosp ([187f00d](https://github.com/elcoosp/nexum/commit/187f00d96cc48b4e215e8c9c5ae61fd81b95f59d))
- add init_with, single-instance, plist, and mobile modules by @elcoosp ([f10e89f](https://github.com/elcoosp/nexum/commit/f10e89ff2a695336f0e59a3a1a26d1e8c32385e6))
- implement windows registry registration by @elcoosp ([a4dd0b8](https://github.com/elcoosp/nexum/commit/a4dd0b8f84094b0dc782dabc73f7e4044a78aebe))
- implement macos apple event handler by @elcoosp ([ca660d3](https://github.com/elcoosp/nexum/commit/ca660d3dc627b9b949483f56b583bea7ecdb1abf))
- implement linux desktop entry and xdg-mime registration by @elcoosp ([0081ea0](https://github.com/elcoosp/nexum/commit/0081ea075da78a3ae93730304d2cecf90c247a69))
- implement global hub and registration bootstrap by @elcoosp ([50db21f](https://github.com/elcoosp/nexum/commit/50db21f8a221581c6a03df4fb798b7ce1831542c))
- create platform crate to reuse code by @elcoosp ([875244d](https://github.com/elcoosp/nexum/commit/875244d5aaab178c4a95f571f4cd13e556249fb2))

### Miscellaneous
- edition 2024 by @elcoosp ([c7c4da3](https://github.com/elcoosp/nexum/commit/c7c4da38c3facd687a79bb39eaa217a03223db95))

### Refactor
- clean up macos targets and fix test gate bounds by @elcoosp ([55db095](https://github.com/elcoosp/nexum/commit/55db09571f78f2a2b0d2e54c3a56063d05c5cb2e))

### build
- bump to 0.3.0 and add single-instance feature by @elcoosp ([2b65cdd](https://github.com/elcoosp/nexum/commit/2b65cdd8f9beb94380c9fa63a6bfde4d9999da75))
- update nexum-platform dependencies by @elcoosp ([e13b442](https://github.com/elcoosp/nexum/commit/e13b442ee8cf61a687725cad3b3f46f6d513cdb2))
**Full Changelog**: https://github.com/elcoosp/nexum/releases/tag/v0.3.0
