use std::process::Command;

/// Custom build script
/// - generate `BUILD_GIT_VERSION` from `git describe --tags --always --dirty`
fn main() {
    // run when HEAD changes
    println!("cargo::rerun-if-changed=.git/HEAD");
    // run when new commit is done on the working branch
    println!("cargo::rerun-if-changed=.git/refs/heads");
    // run when new tag is added
    println!("cargo::rerun-if-changed=.git/refs/tags");
    // run if any change in the directory
    println!("cargo::rerun-if-changed=.");

    if let Some(version) = git_version() {
        println!("cargo::rustc-env=BUILD_GIT_VERSION={version}");
    }
}

/// Get git project version via `git describe --tags --always --dirty`
fn git_version() -> Option<String> {
    let output = Command::new("git")
        .args(["describe", "--tags", "--always", "--dirty"])
        .output()
        .ok()?;
    output.status.success().then_some(())?;
    String::from_utf8_lossy(&output.stdout)
        .strip_suffix('\n')
        .map(ToOwned::to_owned)
}
