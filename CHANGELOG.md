# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed

- Split the `data` crate into pure dump/wire types: client-side domain logic
  (scoring engine, reveal driver, panel views, client helpers) moved to
  `client-model`; server-only dump items (contest state, secret config,
  server-side run transforms) moved to `service`. No wire-format changes.
