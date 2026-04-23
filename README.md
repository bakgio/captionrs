<p align="center">
  <h1 align="center">captionrs</h1>
  <p align="center">
    Subtitle conversion and cleanup for Rust - turn common caption formats into clean SubRip output.
  </p>
  <p align="center">
    <a href="https://crates.io/crates/captionrs"><img src="https://img.shields.io/crates/v/captionrs.svg" alt="Crates.io"></a>
    &nbsp;&nbsp;
    <a href="https://docs.rs/captionrs"><img src="https://img.shields.io/docsrs/captionrs" alt="docs.rs"></a>
    &nbsp;&nbsp;
    <a href="LICENSE-MIT"><img src="https://img.shields.io/crates/l/captionrs.svg" alt="License"></a>
    &nbsp;&nbsp;
    <img src="https://img.shields.io/badge/MSRV-1.88-blue.svg" alt="MSRV 1.88">
  </p>
</p>

---

- **One crate, many subtitle formats** - convert WebVTT, SAMI, SMPTE-TT/TTML, Bilibili JSON, MP4 ISMT, and MP4 WVTT into SubRip
- **Library and CLI** - use typed converters and processors in Rust, or run `captionrs convert` and `captionrs process` from the command line
- **Automatic format detection** - the CLI detects supported input formats and writes `.srt` output directly
- **Structured subtitle model** - work with `SubRipFile` and `Subtitle`, then export or save with configurable encoding and line endings
- **Cleanup pipeline** - fix common subtitle issues, normalize timing problems, remove short gaps, and strip SDH and speaker descriptions
- **Sync and async APIs** - converters and processors expose synchronous APIs, with Tokio-based async variants available behind the optional `async` feature
- **Language-aware processing** - optional language hints can trigger RTL-specific cleanup during subtitle fixing
- **Typed errors** - parsing, format, and I/O failures are surfaced through `SubtitleError`
- **Broad test coverage** - sync and async paths are covered across converters, processors, and end-to-end subtitle workflows

## Installation

```toml
[dependencies]
captionrs = "0.1.0"

# Enable the Tokio-based async library APIs:
# captionrs = { version = "0.1.0", features = ["async"] }
```

Install the CLI with:

```bash
cargo install captionrs
```

## Feature Flags

| Feature | Description |
|:---|:---|
| `async` | Enable Tokio-based async converters and processors for library integrations |

The CLI always runs synchronously. The `async` feature is only for library users who need to integrate `captionrs` into a Tokio-based application.

> See the [`examples/`](examples/) directory for format-specific conversions, sync and async workflows, processor chaining, and CLI-oriented examples.

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT license](LICENSE-MIT) at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in captionrs by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
