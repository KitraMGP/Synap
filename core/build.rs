use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

fn main() {
    let manifest_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let repo_root = manifest_dir
        .parent()
        .expect("core crate should live under the workspace root")
        .to_path_buf();

    let git_dir = resolve_git_dir(&repo_root);
    emit_git_rerun_hints(git_dir.as_deref());

    let metadata = git_metadata(&repo_root, git_dir.as_deref());
    println!("cargo:rustc-env=SYNAP_GIT_BRANCH={}", metadata.branch);
    println!("cargo:rustc-env=SYNAP_GIT_COMMIT={}", metadata.commit);
    println!(
        "cargo:rustc-env=SYNAP_GIT_SHORT_COMMIT={}",
        metadata.short_commit
    );
    println!(
        "cargo:rustc-env=SYNAP_GIT_TAG={}",
        metadata.tag.unwrap_or_default()
    );
}

#[derive(Debug)]
struct GitMetadata {
    branch: String,
    commit: String,
    short_commit: String,
    tag: Option<String>,
}

fn emit_git_rerun_hints(git_dir: Option<&Path>) {
    println!("cargo:rerun-if-changed=build.rs");

    let Some(git_dir) = git_dir else {
        return;
    };

    let head = git_dir.join("HEAD");
    println!("cargo:rerun-if-changed={}", head.display());

    let refs = git_dir.join("refs");
    if refs.exists() {
        println!("cargo:rerun-if-changed={}", refs.display());
    }

    let packed_refs = git_dir.join("packed-refs");
    if packed_refs.exists() {
        println!("cargo:rerun-if-changed={}", packed_refs.display());
    }

    if let Some(reference) = current_head_reference(git_dir) {
        let reference_file = git_dir.join(reference);
        if reference_file.exists() {
            println!("cargo:rerun-if-changed={}", reference_file.display());
        }
    }
}

fn git_metadata(repo_root: &Path, git_dir: Option<&Path>) -> GitMetadata {
    let branch = run_git(repo_root, &["rev-parse", "--abbrev-ref", "HEAD"])
        .filter(|value| !value.is_empty())
        .map(|value| {
            if value == "HEAD" {
                "detached".to_string()
            } else {
                value
            }
        })
        .or_else(|| {
            git_dir
                .and_then(current_head_reference)
                .map(|reference| reference.trim_start_matches("refs/heads/").to_string())
        })
        .unwrap_or_else(|| "unknown".to_string());

    let commit = run_git(repo_root, &["rev-parse", "HEAD"])
        .filter(|value| !value.is_empty())
        .or_else(|| git_dir.and_then(read_head_commit))
        .unwrap_or_else(|| "unknown".to_string());

    let short_commit = run_git(repo_root, &["rev-parse", "--short=12", "HEAD"])
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| short_commit_from(&commit));

    let tag = run_git(repo_root, &["describe", "--tags", "--exact-match", "HEAD"])
        .filter(|value| !value.is_empty());

    GitMetadata {
        branch,
        commit,
        short_commit,
        tag,
    }
}

fn resolve_git_dir(repo_root: &Path) -> Option<PathBuf> {
    let dot_git = repo_root.join(".git");
    if dot_git.is_dir() {
        return Some(dot_git);
    }

    let contents = fs::read_to_string(&dot_git).ok()?;
    let path = contents.strip_prefix("gitdir:")?.trim();
    let git_dir = Path::new(path);
    Some(if git_dir.is_absolute() {
        git_dir.to_path_buf()
    } else {
        repo_root.join(git_dir)
    })
}

fn current_head_reference(git_dir: &Path) -> Option<String> {
    let head = fs::read_to_string(git_dir.join("HEAD")).ok()?;
    head.strip_prefix("ref:")
        .map(str::trim)
        .map(ToOwned::to_owned)
}

fn read_head_commit(git_dir: &Path) -> Option<String> {
    if let Some(reference) = current_head_reference(git_dir) {
        let reference_file = git_dir.join(&reference);
        if let Ok(commit) = fs::read_to_string(&reference_file) {
            return Some(commit.trim().to_string());
        }

        let packed_refs = fs::read_to_string(git_dir.join("packed-refs")).ok()?;
        for line in packed_refs.lines() {
            if line.starts_with('#') || line.starts_with('^') || line.trim().is_empty() {
                continue;
            }

            let mut parts = line.split_whitespace();
            let commit = parts.next()?;
            let name = parts.next()?;
            if name == reference {
                return Some(commit.to_string());
            }
        }
        return None;
    }

    fs::read_to_string(git_dir.join("HEAD"))
        .ok()
        .map(|commit| commit.trim().to_string())
}

fn run_git(repo_root: &Path, args: &[&str]) -> Option<String> {
    let output = Command::new("git")
        .args(args)
        .current_dir(repo_root)
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let value = String::from_utf8(output.stdout).ok()?;
    Some(value.trim().to_string())
}

fn short_commit_from(commit: &str) -> String {
    if commit == "unknown" {
        return commit.to_string();
    }

    commit.chars().take(12).collect()
}
