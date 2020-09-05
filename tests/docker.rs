use docker_command::*;
use std::path::{Path, PathBuf};

fn new_path(s: &str) -> PathBuf {
    Path::new(s).into()
}

#[test]
fn test_build() {
    assert_eq!(
        Docker::new()
            .build(BuildOpt {
                context: new_path("/myContext"),
                dockerfile: Some(new_path("/myContext/myDockerfile")),
                tag: Some("myTag".into()),
            })
            .command_line_lossy(),
        "docker build --file /myContext/myDockerfile --tag myTag /myContext"
    );
}

#[test]
fn test_user() {
    let mut user = User {
        user: NameOrId::Name("myUser".into()),
        group: None,
    };
    assert_eq!(user.arg(), "myUser");

    user.group = Some(NameOrId::Name("myGroup".into()));
    assert_eq!(user.arg(), "myUser:myGroup");

    user.user = NameOrId::Id(1000);
    assert_eq!(user.arg(), "1000:myGroup");
}

#[test]
fn test_run() {
    assert_eq!(
        Docker::new()
            .run(RunOpt {
                image: "myImage".into(),
                detach: true,
                init: true,
                name: Some("myName".into()),
                network: Some("myNetwork".into()),
                read_only: true,
                remove: true,
                user: Some(User {
                    user: NameOrId::Name("myUser".into()),
                    group: Some(NameOrId::Name("myGroup".into())),
                }),
                volumes: vec![
                    // Read-write volume
                    Volume {
                        src: new_path("/mySrc"),
                        dst: new_path("/myDst"),
                        read_write: true,
                        ..Default::default()
                    },
                    // Read-only volume with extra options
                    Volume {
                        src: new_path("/mySrc"),
                        dst: new_path("/myDst"),
                        options: vec!["cached".into(), "z".into()],
                        ..Default::default()
                    }
                ],
                command: Some(Path::new("myCmd").into()),
                args: vec!["arg1".into(), "arg2".into()],
            })
            .command_line_lossy(),
        "docker run --detach --init --name myName --network myNetwork --read-only --rm --user myUser:myGroup --volume /mySrc:/myDst:rw --volume /mySrc:/myDst:ro,cached,z myImage myCmd arg1 arg2"
    );
}

/// Execute "docker run" to test an actual container.
#[test]
fn test_real() {
    let docker = Docker::new();
    let output = docker
        .run(RunOpt {
            image: "alpine:latest".into(),
            command: Some(new_path("echo")),
            args: vec!["hello".into(), "world".into()],
            ..Default::default()
        })
        .run()
        .unwrap();
    assert_eq!(output.stdout_string_lossy(), "hello world\n");
}
