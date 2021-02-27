# Jenkinsfile

[![Docs](https://docs.rs/jenkinsfile/badge.svg)](https://docs.rs/jenkinsfile)
[![Crates.io](https://img.shields.io/crates/d/jenkinsfile.svg)](https://crates.io/crates/jenkinsfile)
[![Crates.io](https://img.shields.io/crates/v/jenkinsfile.svg)](https://crates.io/crates/jenkinsfile)

> a tools to convert Jenkinsfile to data struct.

Usage:

```rust
let jenkinsfile = Jenkinsfile::from_str(code).unwrap();
```

## Struct

```rust
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Jenkinsfile {
    pub name: String,
    pub stages: Vec<JenkinsStage>,
    pub post: Vec<PostConfig>,       // todo
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct JenkinsStage {
    pub name: String,
    pub steps: Vec<String>,
    pub is_parallel: bool,
    pub sub_stages: Vec<JenkinsStage>,
}
```

## LICENSE

code based on [Jenkins Declarative Parser](https://github.com/rtyler/jdp) with LGPL 3.0

@ 2020~2021 This code is distributed under the LGPL 3.0 license. See `LICENSE` in this directory.
