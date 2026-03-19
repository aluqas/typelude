use std::{fs, path::Path};

fn collect_pub_use_lines(root: &Path, out: &mut Vec<String>) {
    for entry in fs::read_dir(root).expect("read_dir failed") {
        let entry = entry.expect("dir entry failed");
        let path = entry.path();

        if path.is_dir() {
            collect_pub_use_lines(&path, out);
            continue;
        }

        if path.extension().and_then(|ext| ext.to_str()) != Some("rs") {
            continue;
        }

        let content = fs::read_to_string(&path).expect("read_to_string failed");
        for (index, line) in content.lines().enumerate() {
            if line.trim_start().starts_with("pub use ") {
                out.push(format!("{}:{}", path.display(), index + 1));
            }
        }
    }
}

#[test]
fn src_has_no_public_reexports() {
    let mut matches = Vec::new();
    let src_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    collect_pub_use_lines(&src_root, &mut matches);

    assert!(
        matches.is_empty(),
        "public re-exports remain:\n{}",
        matches.join("\n")
    );
}
