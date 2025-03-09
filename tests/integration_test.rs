use assert_cmd::prelude::*;
use std::io::Write;
use std::process::{Command, Stdio};
use tempfile::TempDir;

#[test]
fn test_put_and_get_with_cli() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let cache_path = temp_dir.path().to_str().unwrap();

    let test_data = "foo";
    let store_path = "/nix/store/abcd-foo";

    let mut put_cmd = Command::cargo_bin("nr")
        .unwrap()
        .args(["--cache-dir", cache_path, "put", store_path])
        .stdin(Stdio::piped())
        .spawn()
        .expect("Failed to spawn put command");

    {
        let mut stdin = put_cmd.stdin.take().expect("Failed to open stdin");
        stdin
            .write_all(test_data.as_bytes())
            .expect("Failed to write to stdin");
    }

    let put_status = put_cmd.wait().expect("Failed to wait for put command");
    assert!(put_status.success(), "Put command failed");

    let mut get_cmd = Command::cargo_bin("nr")
        .unwrap()
        .args(["--cache-dir", cache_path, "get"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to spawn get command");

    {
        let mut stdin = get_cmd.stdin.take().expect("Failed to open stdin");
        stdin
            .write_all(test_data.as_bytes())
            .expect("Failed to write to stdin");
    }

    let output = get_cmd
        .wait_with_output()
        .expect("Failed to wait for get command");
    assert!(output.status.success(), "Get command failed");

    let retrieved_path = String::from_utf8(output.stdout).expect("Output is not valid UTF-8");
    assert_eq!(retrieved_path.trim(), store_path);
}
