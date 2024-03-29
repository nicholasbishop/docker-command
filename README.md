# docker-command

**This tool is no longer under active development. If you are interested in taking over or repurposing the name on crates.io, feel free to contact me: nbishop@nbishop.net**

[![crates.io](https://img.shields.io/crates/v/docker-command.svg)](https://crates.io/crates/docker-command)
[![Documentation](https://docs.rs/docker-command/badge.svg)](https://docs.rs/docker-command)

Rust library for creating Docker commands.

Rather than speaking directly to the Docker daemon, this library
produces commands that can be run in a subprocess to invoke the Docker
client (or a compatible client such as Podman).

This crate depends on the [command-run] crate. That crate's `logging`
feature (which controls whether the `log` crate is a dependency) can be
toggled with this crate's `logging` feature (enabled by default).

## Example

```rust
let output = Launcher::auto()
    .ok_or("container comand not found")?
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
