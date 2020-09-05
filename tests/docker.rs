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
fn test_run() {
    assert_eq!(
        Docker::new()
            .run(RunOpt {
                image: "myImage".into(),
                detach: true,
                name: Some("myName".into()),
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
        "docker run --detach --name myName --volume /mySrc:/myDst:rw --volume /mySrc:/myDst:ro,cached,z myImage myCmd arg1 arg2"
    );
}
