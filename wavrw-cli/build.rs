use anyhow::Result;
use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

fn main() -> Result<()> {
    let profile = env::var("PROFILE")?;
    if profile == "release" || profile == "dist" {
        println!("skipping licenses.txt update because PROFILE = {profile}");
        return Ok(());
    }

    println!("cargo:rerun-if-changed=Cargo.toml");
    let root_dir = env::var_os("CARGO_MANIFEST_DIR").unwrap();
    println!("root: {root_dir:?}");
    let dest_path = Path::new(&root_dir).join("generated/licenses.txt");
    println!("dest_path: {dest_path:?}");

    let cargo = env::var_os("CARGO").unwrap();
    let output = Command::new(cargo)
        .current_dir(root_dir)
        .arg("license")
        .arg("--avoid-dev-deps")
        // this seems to be too agressive, leaving out some used libraries
        // .arg("--avoid-build-deps")
        .arg("-d")
        .output()?;
    // println!("{}", String::from_utf8_lossy(&output.stdout));
    println!("{}", &output.status);
    println!("{}", String::from_utf8_lossy(&output.stderr));
    assert_eq!(
        output.status.code(),
        Some(0),
        "Failed to run cargo-license. Is it installed?"
    );
    fs::write(dest_path, output.stdout).expect("Unable to write file");
    Ok(())
}
