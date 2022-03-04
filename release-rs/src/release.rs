use crate::{Actions, GitHub};
use regex::Regex;

pub struct Release {
    sha: String,
    label: String,
}

impl Release {
    pub fn new(sha: String, label: String) -> Self {
        Self { sha, label }
    }

    pub fn detect(&self) -> anyhow::Result<()> {
        let sha = &self.sha;
        let label = &self.label;

        let commit: String = cmd_lib::run_fun!(git log --oneline -n 1 "${sha}")?;

        // assert, and exit early, if the current commit message does not carry the word 'release'
        // all release pull-requests will contain the word in their merge commit; however any commit
        // could and therefore this is just a very rough indicator.
        if !commit.contains("release") {
            return self.miss();
        }

        // release commits need to contain a semver version number
        let version = Regex::new(r"(v?\d+.\d+.\d+)")?
            .find_iter(&commit)
            .find(|m| semver::Version::parse(m.as_str()).is_ok());
        if version.is_none() {
            return self.miss();
        }
        let version = version.unwrap().as_str();

        log::debug!("found possible release commit:");
        log::debug!("  {commit}");

        // try and find the pull-request that the commit was part of to examine it
        // a release can only ever be triggered by a pull-request being merged
        if GitHub::find_pull_request_by(sha, label)?.is_none() {
            log::info!("could not match commit to a release pull-request");
            return self.miss();
        }

        self.hit(version)
    }

    fn hit(&self, tag: &str) -> anyhow::Result<()> {
        log::info!("detected release of {tag}");
        Actions::set_output("release-detected", "true");
        Actions::set_output("tag-name", tag);
        Ok(())
    }

    fn miss(&self) -> anyhow::Result<()> {
        log::info!("no release detected");
        Actions::set_output("release-created", "false");
        Ok(())
    }
}
