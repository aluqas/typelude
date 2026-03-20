use std::{fs, path::Path};

const LEGACY_ALGEBRA: &str = concat!("vm::", "algebra");
const LEGACY_SHARED: &str = concat!("shared", "::");

fn collect_legacy_path_uses(root: &Path, out: &mut Vec<String>) {
    for entry in fs::read_dir(root).expect("read_dir failed") {
        let entry = entry.expect("dir entry failed");
        let path = entry.path();

        if path.is_dir() {
            collect_legacy_path_uses(&path, out);
            continue;
        }

        if path.extension().and_then(|ext| ext.to_str()) != Some("rs") {
            continue;
        }

        let content = fs::read_to_string(&path).expect("read_to_string failed");
        for (index, line) in content.lines().enumerate() {
            if line.contains(LEGACY_ALGEBRA) || line.contains(LEGACY_SHARED) {
                out.push(format!("{}:{}", path.display(), index + 1));
            }
        }
    }
}

#[test]
fn src_and_tests_have_no_legacy_vm_paths() {
    let manifest_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut matches = Vec::new();

    collect_legacy_path_uses(&manifest_root.join("src"), &mut matches);
    collect_legacy_path_uses(&manifest_root.join("tests"), &mut matches);

    assert!(
        matches.is_empty(),
        "legacy vm paths remain:\n{}",
        matches.join("\n")
    );
}
