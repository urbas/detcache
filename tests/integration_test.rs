use assert_cmd::cargo;
use std::fs::File;
use std::io::Write;
use std::process::{Command, Stdio};
use tempfile::TempDir;

#[test]
fn test_put_and_get_with_cli() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let cache_path = temp_dir.path().to_str().unwrap();

    // Create a config file with fs cache
    let config_path = temp_dir.path().join("config.toml");
    let mut config_file = File::create(&config_path).expect("Failed to create config file");
    writeln!(
        config_file,
        r#"[caches.fs]
type = "fs"
cache_dir = "{}""#,
        cache_path
    )
    .expect("Failed to write config");
    let config_path_str = config_path.to_str().unwrap();

    let hash = "b5bb9d8014a0f9b1d61e21e796d78dccdf1352f23cd32812f4850b878ae4944c";
    let value = "foo calculation result";

    let mut put_cmd = Command::new(cargo::cargo_bin!("detcache"))
        .args(["--config", config_path_str, "put", hash])
        .stdin(Stdio::piped())
        .spawn()
        .expect("Failed to spawn put command");

    {
        let mut stdin = put_cmd.stdin.take().expect("Failed to open stdin");
        stdin
            .write_all(value.as_bytes())
            .expect("Failed to write to stdin");
    }

    let put_status = put_cmd.wait().expect("Failed to wait for put command");
    assert!(put_status.success(), "Put command failed");

    let get_cmd = Command::new(cargo::cargo_bin!("detcache"))
        .args(["--config", config_path_str, "get", hash])
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to spawn get command");

    let output = get_cmd
        .wait_with_output()
        .expect("Failed to wait for get command");
    assert!(output.status.success(), "Get command failed");

    let retrieved_path = String::from_utf8(output.stdout).expect("Output is not valid UTF-8");
    assert_eq!(retrieved_path.trim(), value);
}
