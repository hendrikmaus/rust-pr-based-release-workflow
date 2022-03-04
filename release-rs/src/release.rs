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
            .map(|m| semver::Version::parse(m.as_str()).unwrap())
            .next();

        if version.is_none() {
            return self.miss();
        }

        let version = version.unwrap();
        log::info!("found possible release commit:");
        log::info!("  {commit}");

        // try and find the pull-request that the commit was part of to examine it
        // a release can only ever be triggered by a pull-request being merged
        let pr = GitHub::find_pull_request_by(sha, label)?;

        if pr.is_none() {
            return self.miss();
        }

        log::info!("detected release of {version}");
        self.hit()
    }

    fn hit(&self) -> anyhow::Result<()> {
        Actions::set_output("release-created", "true");
        Ok(())
    }

    fn miss(&self) -> anyhow::Result<()> {
        log::info!("no release detected");
        Actions::set_output("release-created", "false");
        Ok(())
    }
}
