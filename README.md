# Reki

_Reki_ (礫) is a study project to explore general-purpose GPU computing
programming environments. It is a combination of a simplified C-like
kernel development language and a host platform providing rudimentary
correctness testing and benchmarking capabilities.

As this is very much a work in progress, the feature scope may change.
At the moment, I'm focusing on the following:

* translation of source code written in _Reki_ to AMD GCN assembly
(the final binary is produced by [HCC](https://github.com/RadeonOpenCompute/hcc/wiki))
* kernel execution with data-driven tests
(inspired by [mokt](https://github.com/band-of-four/master-of-kernel-testing),
which I collaborated on) and basic performance stats collection

The language will likely have major limitations in available
control flow constructs, data types, and intrinsic functions.

## Development

A Docker image with the Rust toolchain and ROCm development tools is available
on Docker Hub as [timlathy/reki](https://hub.docker.com/r/timlathy/reki/).

Run it with `docker run -it --rm -v \`pwd\`:/src timlathy/reki`, assuming
the current working directory is the root of this repository.
