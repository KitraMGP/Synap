#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildInfo {
    pub crate_version: &'static str,
    pub git_branch: &'static str,
    pub git_commit: &'static str,
    pub git_short_commit: &'static str,
    pub git_tag: Option<&'static str>,
}

impl BuildInfo {
    pub fn display_version(&self) -> String {
        match self.git_tag {
            Some(tag) => format!("{tag} ({})", self.git_short_commit),
            None => format!("{} ({})", self.git_branch, self.git_short_commit),
        }
    }
}

pub fn build_info() -> BuildInfo {
    let git_tag = env!("SYNAP_GIT_TAG");

    BuildInfo {
        crate_version: env!("CARGO_PKG_VERSION"),
        git_branch: env!("SYNAP_GIT_BRANCH"),
        git_commit: env!("SYNAP_GIT_COMMIT"),
        git_short_commit: env!("SYNAP_GIT_SHORT_COMMIT"),
        git_tag: if git_tag.is_empty() {
            None
        } else {
            Some(git_tag)
        },
    }
}

pub fn version_string() -> String {
    build_info().display_version()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_info_contains_commit_identity() {
        let info = build_info();
        assert!(!info.crate_version.is_empty());
        assert!(!info.git_branch.is_empty());
        assert!(!info.git_commit.is_empty());
        assert!(!info.git_short_commit.is_empty());
    }

    #[test]
    fn test_version_string_prefers_tag_when_available() {
        let info = build_info();
        let version = version_string();

        if let Some(tag) = info.git_tag {
            assert!(version.contains(tag));
        } else {
            assert!(version.contains(info.git_branch));
        }
        assert!(version.contains(info.git_short_commit));
    }
}
