use crate::{Actions, GitHub};

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
            log::info!("no release detected");
            self.miss();
            return Ok(());
        }
        log::info!("found possible release commit:");
        log::info!("  {commit}");

        // try and find the pull-request that the commit was part of to examine it
        // a release can only ever be triggered by a pull-request being merged
        let pr = GitHub::find_pull_request_by(sha, label)?;

        match pr {
            Some(_p) => {
                log::info!("detected release");
                // todo now we'd need to do additional parsing, e.g. get the tag to create
                // todo for example this simple regex can pull the version from the commit: (v?\d+.\d+.\d+-?.*)
                self.hit();
            }
            None => {
                log::info!("commit could not be found in a release pull-request");
                self.miss();
            }
        }

        Ok(())
    }

    fn hit(&self) {
        Actions::set_output("release-created", "true")
    }

    fn miss(&self) {
        Actions::set_output("release-created", "false")
    }
}
