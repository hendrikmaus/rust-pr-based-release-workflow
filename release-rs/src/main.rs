mod github;

use crate::github::{Actions, GitHub};
use anyhow::Context;

// Name of the GitHub Actions output that following steps can read
// todo make command line option with default value
static OUTPUT_RELEASE_DETECTED: &str = "release-detected";

// The name of the label to expect on release pull-requests
// todo make command line option with default value
static PULL_REQUEST_LABEL: &str = "autorelease";

struct Release;

impl Release {
    fn hit() {
        Actions::set_output(OUTPUT_RELEASE_DETECTED, "true")
    }

    fn miss() {
        Actions::set_output(OUTPUT_RELEASE_DETECTED, "false")
    }
}

fn main() -> anyhow::Result<()> {
    env_logger::init();

    let start = std::time::Instant::now();
    log::trace!("starting release-rs process");

    let sha = std::env::var("GITHUB_SHA")
        .context("Could not fetch GITHUB_SHA from the environment, is it set?")?;
    log::trace!("got commit sha from GITHUB_SHA: {sha}");

    let commit: String = cmd_lib::run_fun!(git log --oneline -n 1 "${sha}")?;

    // assert, and exit early, if the current commit message does not carry the word 'release'
    // all release pull-requests will contain the word in their merge commit; however any commit
    // could and therefore this is just a very rough indicator.
    if !commit.contains("release") {
        log::info!("no release detected");
        Release::miss();
        return Ok(());
    }
    log::info!("found possible release commit:");
    log::info!("\t{commit}");

    // try and find the pull-request that the commit was part of to examine it
    // a release can only ever be triggered by a pull-request being merged
    let pr = GitHub::find_pull_request_by(&sha, PULL_REQUEST_LABEL)?;

    match pr {
        Some(_p) => {
            log::info!("detected release");
            // todo now we'd need to do additional parsing, e.g. get the tag to create
            // todo for example this simple regex can pull the version from the commit: (v?\d+.\d+.\d+-?.*)
            Release::hit();
        }
        None => {
            log::info!("commit could not be found in a release pull-request");
            Release::miss();
        }
    }

    log::trace!("finished release-rs process, took {:?}", start.elapsed());

    Ok(())
}
