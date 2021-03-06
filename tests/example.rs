use docker_command::*;
use std::path::Path;

/// Execute "docker run" to test an actual container. This example is
/// used in the readme.
#[test]
fn test_example() -> Result<(), command_run::Error> {
    // Begin readme example
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
    // End readme example
    Ok(())
}
