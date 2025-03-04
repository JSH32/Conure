use std::{env, fs, path::PathBuf};

use fs_extra::file::{CopyOptions, move_file};
use walkdir::WalkDir;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let target_dir = out_dir.join("capnp");

    if target_dir.exists() {
        fs::remove_dir_all(&target_dir)?;
        fs::create_dir_all(&target_dir)?;
    } else {
        fs::create_dir_all(&target_dir)?;
    }

    capnpc::CompilerCommand::new()
        .file("../protocol/conure_rpc.capnp")
        .output_path(&target_dir)
        .run()?;

    for entry in WalkDir::new(&target_dir).into_iter().filter_map(Result::ok) {
        if entry.file_type().is_file() {
            let new_path = target_dir.join(entry.file_name());
            move_file(entry.path(), new_path, &CopyOptions::new())?;
        }
    }

    println!(
        "cargo:rustc-env=CAP_DIR={}",
        fs::canonicalize(&target_dir).unwrap().to_str().unwrap()
    );
    Ok(())
}
