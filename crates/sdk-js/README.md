# @trysoma/sdk-core

Core native bindings for the [Soma](https://github.com/trysoma/soma) JavaScript SDK.

## Overview

This package provides the low-level native bindings (built with Rust and NAPI-RS) that power the Soma JavaScript SDK. It handles performance-critical operations like schema parsing, validation, and code generation.

**Most users should install `@trysoma/sdk` instead**, which provides a high-level TypeScript API built on top of this package.

## Installation

```bash
npm install @trysoma/sdk
```

If you need the core bindings directly:

```bash
npm install @trysoma/sdk-core
```

## Supported Platforms

- macOS (x86_64, ARM64)
- Linux (x86_64, ARM64)

## Documentation

For comprehensive documentation and guides, visit [https://docs.trysoma.ai/](https://docs.trysoma.ai/)

## Repository

[https://github.com/trysoma/soma](https://github.com/trysoma/soma)
