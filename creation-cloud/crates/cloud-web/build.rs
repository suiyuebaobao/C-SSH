use std::{fs, path::Path};

fn main() {
    for root in ["templates", "static"] {
        emit_rerun_paths(Path::new(root));
    }
}

fn emit_rerun_paths(root: &Path) {
    println!("cargo:rerun-if-changed={}", root.display());

    let mut pending = vec![root.to_path_buf()];
    while let Some(directory) = pending.pop() {
        let mut entries = fs::read_dir(&directory)
            .unwrap_or_else(|error| {
                panic!(
                    "failed to inspect embedded web asset directory {}: {error}",
                    directory.display()
                )
            })
            .map(|entry| {
                entry.unwrap_or_else(|error| {
                    panic!(
                        "failed to inspect embedded web asset entry under {}: {error}",
                        directory.display()
                    )
                })
            })
            .collect::<Vec<_>>();
        entries.sort_by_key(|entry| entry.path());

        for entry in entries {
            let path = entry.path();
            let file_type = entry.file_type().unwrap_or_else(|error| {
                panic!(
                    "failed to inspect embedded web asset type {}: {error}",
                    path.display()
                )
            });
            if file_type.is_dir() {
                pending.push(path);
            } else if file_type.is_file() {
                println!("cargo:rerun-if-changed={}", path.display());
            } else {
                panic!(
                    "embedded web asset path must be a regular file or directory: {}",
                    path.display()
                );
            }
        }
    }
}
