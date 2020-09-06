use docker_command::*;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

fn new_path(s: &str) -> PathBuf {
    Path::new(s).into()
}

#[test]
fn test_build() {
    let mut build_args = BTreeMap::new();
    build_args.insert("barg1".into(), "bval1".into());
    build_args.insert("barg2".into(), "bval2".into());
    assert_eq!(
        Docker::new()
            .build(BuildOpt {
                build_args,
                context: new_path("/myContext"),
                dockerfile: Some(new_path("/myContext/myDockerfile")),
                no_cache: true,
                pull: true,
                quiet: true,
                tag: Some("myTag".into()),
            })
            .command_line_lossy(),
        "docker build --build-arg barg1=bval1 --build-arg barg2=bval2 --file /myContext/myDockerfile --no-cache --pull --quiet --tag myTag /myContext"
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

/// Test that tests/example.rs is faithfully reproduced in the readme.
#[test]
fn test_readme_example() {
    let source = include_str!("../tests/example.rs");

    // Extract the example code between two comments and de-indent
    let mut example = Vec::new();
    let mut copy = false;
    for line in source.lines() {
        if line.contains("Begin readme example") {
            copy = true;
        } else if line.contains("End readme example") {
            break;
        } else if copy {
            // De-indent
            let line = &line[4..];
            example.push(line);
        }
    }
    let example = example.join("\n");

    let readme = include_str!("../README.md");
    assert!(readme.contains(&example));
}
