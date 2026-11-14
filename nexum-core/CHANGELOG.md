# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).
## [0.3.0] - 2026-11-14

### Bug Fixes
- clean dioxus example by @elcoosp ([8e703e5](https://github.com/elcoosp/nexum/commit/8e703e522a39c313677c7bb9271a940294005732))
- compiling by @elcoosp ([d2e105e](https://github.com/elcoosp/nexum/commit/d2e105e03df550351797c160063721a957d591e6))
- core compile by @elcoosp ([e4a2a62](https://github.com/elcoosp/nexum/commit/e4a2a6232bdc5aee891dbbc2511a3a713b33edbd))

### Documentation
- add Vercel documentation link to Cargo.toml by @elcoosp ([05ecc54](https://github.com/elcoosp/nexum/commit/05ecc54120b1632b904cc0710752473473c1481d))
- enforce SKILL.md rules (jsDelivr logos, absolute links, remove license sections) by @elcoosp ([94cb6bd](https://github.com/elcoosp/nexum/commit/94cb6bd393a00105a94d226844e1e94b9a1037ab))
- convert Stream backlink to code span format by @elcoosp ([209acfd](https://github.com/elcoosp/nexum/commit/209acfd418b91914a1a8a5cc7ffaec6e77e7e73d))
- add absolute path to Error::MalformedUrl link reference by @elcoosp ([a9d8708](https://github.com/elcoosp/nexum/commit/a9d870896b2ec9c8068639619abc505a1993f8d7))
- fix internal item link path for bounded hub queue by @elcoosp ([816f2c7](https://github.com/elcoosp/nexum/commit/816f2c7021f110d747c00122677b30b765c28af8))
- add nexum-core readme by @elcoosp ([8bf9f16](https://github.com/elcoosp/nexum/commit/8bf9f16ca07ebc74fdceb4a56b7a5801c41d6723))

### Features
- support bounded hub queues and non-blocking pushes by @elcoosp ([c5b6844](https://github.com/elcoosp/nexum/commit/c5b6844561fbb530d424c9e32eb7099592120ad8))
- add QueueFull error for bounded hubs by @elcoosp ([c4ef52e](https://github.com/elcoosp/nexum/commit/c4ef52e40e49500d5b59e7f84689a46b96279869))
- add deep link producer hub by @elcoosp ([7e54f2c](https://github.com/elcoosp/nexum/commit/7e54f2ce35b04c8081989d2ca3f7dcd1f5bfecf0))
- add deep link consumer handle by @elcoosp ([6bea174](https://github.com/elcoosp/nexum/commit/6bea174c936a1299cee2b20e319e063a6b8e06fa))
- add deep link event and source types by @elcoosp ([cf5f9e6](https://github.com/elcoosp/nexum/commit/cf5f9e603f890f677080a828bd9dfea7fc8b3e86))
- add unified error types by @elcoosp ([51e6549](https://github.com/elcoosp/nexum/commit/51e6549a8e0b48e6b6f64ac0bbeff799a9fb4471))
- add configuration and validation logic by @elcoosp ([cc9f438](https://github.com/elcoosp/nexum/commit/cc9f4387c2055beb68f094ec11ce7dd631cb53bf))
- create platform crate to reuse code by @elcoosp ([875244d](https://github.com/elcoosp/nexum/commit/875244d5aaab178c4a95f571f4cd13e556249fb2))
- working example with unified better api by @elcoosp ([d926b06](https://github.com/elcoosp/nexum/commit/d926b0650b4d1e5f297258439b0d00eab4ba21cb))
- working xilem-basic example by @elcoosp ([678594d](https://github.com/elcoosp/nexum/commit/678594d58bc1ed647e4a4e3070ba97b73e83d5ea))
- receiving deeplink xilem example by @elcoosp ([da91a99](https://github.com/elcoosp/nexum/commit/da91a992f7e58a2fe24a82255df349b57d8dc7c0))
- working gpui-basic with lean core by @elcoosp ([213cfcd](https://github.com/elcoosp/nexum/commit/213cfcdbe1f4fdcf7ea9044db3402639b5a927cf))
- compiling macos by @elcoosp ([baf5f3c](https://github.com/elcoosp/nexum/commit/baf5f3c27c404a1d48ab53982f87e1711424161f))
- gpui example with cli arg by @elcoosp ([711d0ce](https://github.com/elcoosp/nexum/commit/711d0cea7428c609d751d94c711bbde8fb454e30))

### Miscellaneous
- fix workspace inheritance and inject curated keywords by @elcoosp ([c636a12](https://github.com/elcoosp/nexum/commit/c636a12140ce49c2b3c423837bc7abf139abf0c3))
- edition 2024 by @elcoosp ([c7c4da3](https://github.com/elcoosp/nexum/commit/c7c4da38c3facd687a79bb39eaa217a03223db95))
- initial workspace and core crate with platform implementations by @elcoosp ([3eb93a2](https://github.com/elcoosp/nexum/commit/3eb93a23d2f120cf57dfcfbf19c33442bf060a5f))

### Refactor
- make bounded event push fallback to non-blocking on wasm by @elcoosp ([53de30d](https://github.com/elcoosp/nexum/commit/53de30d8255084cc788e2b43bff3617c73bc0edf))
- gate blocking receiver primitives away from wasm by @elcoosp ([580b5e8](https://github.com/elcoosp/nexum/commit/580b5e8fbdb933f8dabe81d1200714702e72e5a3))
- restructure module exports and documentation by @elcoosp ([f7eb272](https://github.com/elcoosp/nexum/commit/f7eb272a995a289d2fbed6b587c0c4f33fcf0f0b))

### build
- update nexum-core dependencies and features by @elcoosp ([974bc05](https://github.com/elcoosp/nexum/commit/974bc05d14dbf1ced0dab4e0bd485b9c55981c18))
**Full Changelog**: https://github.com/elcoosp/nexum/releases/tag/v0.3.0
