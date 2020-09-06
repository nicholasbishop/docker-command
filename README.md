# docker-command

[![crates.io](https://img.shields.io/crates/v/docker-command.svg)](https://crates.io/crates/docker-command)
[![Documentation](https://docs.rs/docker-command/badge.svg)](https://docs.rs/docker-command)

Rust library for creating Docker commands.

Rather than speaking directly to the Docker daemon, this library
produces commands that can be run in a subprocess to invoke the Docker
client (or a compatible client such as Podman).

This crate depends on the [command-run] crate.

## Example

```rust
let output = Docker::new()
    .run(RunOpt {
        image: "alpine:latest".into(),
        command: Some(Path::new("echo").into()),
        args: vec!["hello".into(), "world".into()],
        ..Default::default()
    })
    .enable_capture()
    .run()?;
assert_eq!(output.stdout_string_lossy(), "hello world\n");
```

## TODO

Only a few Docker commands are currently implemented, and many of the
available options for those commands are not yet
implemented. Contributions welcome!

[command-run]: https://crates.io/crates/command-run
