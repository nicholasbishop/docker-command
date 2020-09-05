#![deny(missing_docs)]

//! Create [`Command`]s for running Docker or Docker-compatible clients.
//!
//! Rather than speaking directly to the Docker daemon, this library
//! produces commands that can be run in a subprocess to invoke the
//! Docker client (or a compatible client such as Podman).
//!
//! [`Command`]: https://docs.rs/command-run/latest/command_run/struct.Command.html

use command_run::Command;
use std::ffi::OsString;
use std::path::{Path, PathBuf};

/// Base container command used for building and running containers.
///
/// This allows variations such as "docker", "sudo docker", and
/// "podman".
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Docker {
    /// If true, run the command with `sudo`. The default is false.
    pub sudo: bool,

    /// The container command. The default is "docker", but can be
    /// changed to another compatible program such as "podman".
    pub program: PathBuf,
}

impl Docker {
    /// Create a new Docker instance with the default values set.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create the base `Command` for running Docker.
    pub fn command(&self) -> Command {
        if self.sudo {
            Command::with_args("sudo", &["docker"])
        } else {
            Command::new("docker")
        }
    }

    /// Create a `Command` for building a container.
    pub fn build(&self, opt: BuildOpt) -> Command {
        let mut cmd = self.command();
        cmd.add_arg("build");
        if let Some(dockerfile) = &opt.dockerfile {
            cmd.add_arg_pair("--file", dockerfile);
        }
        if let Some(tag) = &opt.tag {
            cmd.add_arg_pair("--tag", tag);
        }
        cmd.add_arg(opt.context);
        cmd
    }

    /// Create a `Command` for running a container.
    pub fn run(&self, opt: RunOpt) -> Command {
        let mut cmd = self.command();
        cmd.add_arg("run");
        if let Some(name) = &opt.name {
            cmd.add_arg_pair("--name", name);
        }

        // Add volumes
        for vol in &opt.volumes {
            cmd.add_arg_pair("--volume", vol.arg());
        }

        // Add image and command+args
        cmd.add_arg(opt.image);
        if let Some(command) = &opt.command {
            cmd.add_arg(command);
        }
        cmd.add_args(&opt.args);
        cmd
    }
}

impl Default for Docker {
    fn default() -> Self {
        Docker {
            sudo: false,
            program: Path::new("docker").into(),
        }
    }
}

/// Options for building a container.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct BuildOpt {
    /// Root directory containing files that can be pulled into the
    /// container.
    pub context: PathBuf,

    /// Dockerfile to build. This must be somewhere in the `context`
    /// directory. If not set (the default) then
    /// `<context>/Dockerfile` is used.
    pub dockerfile: Option<PathBuf>,

    /// If set, the image will be tagged with this name.
    pub tag: Option<String>,
}

/// Volume specification used when running a container.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Volume {
    /// Either a path on the host (if an absolute path) or the name of
    /// a volume.
    pub src: PathBuf,

    /// Absolute path in the container where the volume will be
    /// mounted.
    pub dst: PathBuf,

    /// If true, mount the volume read-write. Defaults to `false`.
    pub read_write: bool,

    /// Additional options to set on the volume.
    pub options: Vec<String>,
}

impl Volume {
    fn arg(&self) -> OsString {
        let mut out = OsString::new();
        out.push(&self.src);
        out.push(":");
        out.push(&self.dst);
        if self.read_write {
            out.push(":rw");
        } else {
            out.push(":ro");
        }
        for opt in &self.options {
            out.push(",");
            out.push(opt);
        }
        out
    }
}

/// Options for running a container.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct RunOpt {
    /// Container image to run.
    pub image: String,

    /// Optional name to give the container.
    pub name: Option<String>,

    /// Volumes to mount in the container.
    pub volumes: Vec<Volume>,

    /// Optional command to run.
    pub command: Option<PathBuf>,

    /// Optional arguments to pass to the command.
    pub args: Vec<OsString>,
}
