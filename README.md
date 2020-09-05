# docker-command

[![crates.io](https://img.shields.io/crates/v/docker-command.svg)](https://crates.io/crates/docker-command)
[![Documentation](https://docs.rs/docker-command/badge.svg)](https://docs.rs/docker-command)

Rust library for creating Docker commands.

Rather than speaking directly to the Docker daemon, this library
produces commands that can be run in a subprocess to invoke the Docker
client (or a compatible client such as Podman).
