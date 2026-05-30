# 0.2.0 (May 30, 2026)

- Upgraded the `mp4forge` dependency from 0.4 to 0.8, pinned with `default-features = false` so only the
  core box, codec, fourcc, and walk APIs that the MP4 subtitle converters rely on are pulled in
- Added a `cargo-semver-checks` CI job and made the GitHub release depend on it to guard against
  unintended public API breakage

# 0.1.0 (April 23, 2026)

- Initial crate release
