use anyhow::Context;

fn main() -> anyhow::Result<()> {
    env_logger::init();

    let mut release_detected = false;

    let start = std::time::Instant::now();
    log::trace!("starting release-rs process");

    let sha = std::env::var("GITHUB_SHA")
        .context("Could not fetch GITHUB_SHA from the environment, is it set?")?;
    log::trace!("got commit sha from GITHUB_SHA: {sha}");

    let commit: String = cmd_lib::run_fun!(git log --oneline -n 1 "${sha}")?;
    log::info!("got commit: '{commit}'");

    if !commit.contains("release") {
        log::debug!("commit doesn't contain the word 'release'");
    } else {
        // todo set release detected for now without further investigation
        release_detected = true;
    }

    // todo use `gh` to get the pull-request (if any) of the current commit
    // gh pr list --state merged --search ${sha} --limit 1 --json number,url,id,title
    // the command returns an array of json object with the respective keys

    println!("::set-output name=release-detected::{release_detected}");

    log::trace!("finished release-rs process, took {:?}", start.elapsed());

    Ok(())
}
