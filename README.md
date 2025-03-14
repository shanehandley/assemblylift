<p align="center">
  <img width="600" src="./docs/AssemblyLift_logo_with_text.png">
</p>

![AssemblyLift CI](https://github.com/akkoro/assemblylift/workflows/AssemblyLift%20CI/badge.svg)
![Crates.io](https://img.shields.io/crates/v/assemblylift-cli)

AssemblyLift is a framework for building serverless applications powered by WebAssembly (WASM).

Highlight reel:

- ["IO Modules"](https://dev.to/dotxlem/assemblylift-v0-2-preview-rpc-based-io-modules-2d38) provide a plugin interface for both the host and WASM guest,
  allowing guests to **safely** make calls to the outside world without needing elevated access.
- IOmods are implemented on top of [Cap'n Proto RPC](https://capnproto.org), and guests written using Rust fully support **async/await**.
- Currently focusing on support for guests written in **Rust**, but other languages targeting WASM are possible. PR's welcome!
- Planned support for multiple backends, but the focus is currently on [AWS Lambda](https://aws.amazon.com/lambda/)
- Built using the [Wasmer](https://wasmer.io) interpreter

**Examples** can be found [here](https://github.com/akkoro/assemblylift-examples).

# Overview

The three primary aims of this project, are to provide you with an _ergonomic_ development framework for building serverless applications
which are both _efficient_, and _safe_.

## Efficiency

WebAssembly modules [are smaller and faster](https://medium.com/@OPTASY.com/webassembly-vs-javascript-is-wasm-faster-than-js-when-does-javascript-perform-better-db86d2ecf2cc)
than their NodeJS counterparts. Combined with the IOmod framework, most of the heavy lifting (such as a call to an AWS
service) is handled by the host runtime (which is native code, written in Rust).

## Safety

WebAssembly modules are isolated -- they are sandboxed with their own memory, and have no access to the outside world
(such as by opening a socket connection). This allows your serverless guest code to be _untrusted_.

A side-effect of this with respect to an IOmod, is that the guest code has to ask the host to execute
any third-party dependency code which needs network access. Ideally this will help you prevent unwanted version changes that
have a habit of sneaking into function code, keeping your entire project in sync and giving you tighter control over
your dependency supply chain.

## Ergonomics

It's still early days, so there's nothing in this repo right now which I would characterize as ergonomic. In terms of
plans in this area, I intend for the tooling to abstract away as much of the underlying backend as possible (ie AWS vs Azure).

# Roadmap

## 0.3 - The Observability Release
[ ] TBD

## 0.4 - The Reliability Release
[ ] TBD

# Contributing

I'd like to figure this part out collaboratively. Just in terms of getting code merged though,
I'm a big fan of [forking workflow](https://www.atlassian.com/git/tutorials/comparing-workflows/forking-workflow),
so let's start there 🙂.

# License

The AssemblyLift source code is licensed under [Hippocratic License 2.1](/LICENSE.md).  
The AssemblyLift CLI delegates some tasks to [HashiCorp Terraform](https://terraform.io), which is licensed under [Mozilla Public License 2.0](https://www.mozilla.org/en-US/MPL/2.0/).
