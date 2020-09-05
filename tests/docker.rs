use docker_command::*;
use std::path::Path;

#[test]
fn test_build() {
    assert_eq!(
        Docker::new()
            .build(BuildOpt {
                context: Path::new("/myContext").into(),
                dockerfile: Some(Path::new("/myContext/myDockerfile").into()),
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
                name: Some("myName".into()),
                image: "myImage".into(),
                command: Some(Path::new("myCmd").into()),
                args: vec!["arg1".into(), "arg2".into()],
            })
            .command_line_lossy(),
        "docker run --name myName myImage myCmd arg1 arg2"
    );
}
