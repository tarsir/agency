use std::process::Command;
use std::process::Stdio;

pub fn run(args: &[&str]) -> String {
    let mut cmd = Command::new("cargo");
    cmd.args(["run", "--"]);

    for arg in args {
        cmd.arg(arg);
    }

    let output = cmd
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap()
        .wait_with_output()
        .unwrap();

    if !output.status.success() {
        let err_msg = String::from_utf8(output.stderr).unwrap();
        panic!("{err_msg}");
    }

    String::from_utf8(output.stdout).unwrap().trim().to_string()
}
