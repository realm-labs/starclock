use std::path::Path;
use std::process::Command;

#[test]
fn workspace_dependency_boundaries_hold() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .and_then(Path::parent)
        .expect("CLI crate must live under crates/ at the workspace root");
    let status = Command::new("node")
        .arg("tools/workspace/verify-dependencies.mjs")
        .current_dir(workspace_root)
        .status()
        .expect("Node.js must run the workspace dependency verifier");
    assert!(status.success(), "workspace dependency verifier failed");
}
