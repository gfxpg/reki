# Reki

_Reki_ (礫) is a study project to explore the low-level foundations
of general-purpose GPU computing. It is a basic _decompiler_ for
AMD GCN kernels, using LLVM to disassemble machine code and leveraging
various metadata to translate assembly into a higher-level language.

As this is very much a work in progress, the feature scope may change.
At the moment, I have yet to decide on the output format:
producing _valid_ OpenCL code (with inline assembly for complex paths)
would be nice, but I'm not sure how feasible it is given my limited
knowledge and time constraints.

## Development

A Docker image with the Rust toolchain and ROCm development tools is available
on Docker Hub as [timlathy/reki](https://hub.docker.com/r/timlathy/reki/).

Run it with `docker run -it --rm -v $(pwd):/src timlathy/reki`, assuming
the current working directory is the root of this repository.
