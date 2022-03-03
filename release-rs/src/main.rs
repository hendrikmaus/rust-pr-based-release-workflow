use anyhow::Context;
use cmd_lib::run_fun;
use serde::Deserialize;

// Name of the GitHub Actions output that following steps can read
static OUTPUT_RELEASE_DETECTED: &str = "release-detected";

// The name of the label to expect on release pull-requests
static PULL_REQUEST_LABEL: &str = "autorelease";

#[derive(Deserialize, Debug, Clone)]
struct PullRequest {
    number: u32,
}

#[derive(Deserialize, Debug)]
struct Labels {
    labels: Vec<Label>,
}

#[derive(Deserialize, Debug)]
struct Label {
    name: String,
}

struct Release;

impl Release {
    #[allow(dead_code)]
    fn hit() {
        set_output(OUTPUT_RELEASE_DETECTED, "true")
    }

    fn miss() {
        set_output(OUTPUT_RELEASE_DETECTED, "false")
    }
}

// Set an "output" in GitHub Actions
fn set_output(key: &str, value: &str) {
    println!("::set-output name={key}::{value}");
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
    let pr = find_pull_request_by(&sha)?;

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

// Shells out to GitHub's CLI `gh` to try and determine if the commit belongs to any pull-request
fn find_pull_request_by(sha: &str) -> anyhow::Result<Option<PullRequest>> {
    log::trace!("listing pull-requests that contain the commit:");
    let pulls = run_fun!(gh pr list --state merged --search ${sha} --limit 1 --json number)?;
    log::trace!("{pulls}");

    let pulls: Vec<PullRequest> = serde_json::from_str(&pulls)?;

    if pulls.is_empty() {
        log::trace!("the commit sha {sha} is not part of any pull-request");
        return Ok(None);
    }

    if pulls.len() != 1 {
        anyhow::bail!("the commit is part of more than pull-request; cannot parse at this time");
    }

    let pr = pulls.first().unwrap().clone();
    log::trace!("extracted {pr:?}");

    log::trace!("getting labels for the possibly qualified pull-request:");
    let labels = {
        let pr_number = pr.number;
        run_fun!(gh pr view ${pr_number} --json labels)?
    };
    log::trace!("{labels}");

    let labels: Labels = serde_json::from_str(&labels)?;

    if labels.labels.is_empty() {
        log::trace!(
            "the commit sha {sha} is not part of any pull-request with the {PULL_REQUEST_LABEL} label"
        );
        return Ok(None);
    }

    for label in labels.labels {
        if label.name == PULL_REQUEST_LABEL {
            return Ok(Some(pr));
        }
    }

    Ok(None)
}
