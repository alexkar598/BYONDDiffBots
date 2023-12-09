use actix_web::rt::Runtime;
use diffbot_lib::{github::github_types::ChangeType, job::types::Job};
use dmm_tools::dmi::IconFile;
use eyre::{Context, Result};
use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

use crate::downloading::download_url;

#[derive(Debug)]
pub struct IconFileWithName {
    pub full_name: String,
    pub sha: String,
    pub hash: u64,
    pub icon: IconFile,
}

pub fn status_to_sha<'a>(job: &'a Job, status: &ChangeType) -> (Option<&'a str>, Option<&'a str>) {
    match status {
        ChangeType::Added => (None, Some(&job.head.sha)),
        ChangeType::Deleted => (Some(&job.base.sha), None),
        ChangeType::Modified => (Some(&job.base.sha), Some(&job.head.sha)),
        _ => (None, None),
    }
}

pub fn sha_to_iconfile(
    job: &Job,
    filename: &str,
    sha: (Option<&str>, Option<&str>),
    client: reqwest::Client,
) -> Result<(Result<Option<IconFileWithName>>, Option<IconFileWithName>)> {
    Ok((
        get_if_exists(job, filename, sha.0, client.clone()),
        get_if_exists(job, filename, sha.1, client)?,
    ))
}

#[tracing::instrument]
fn get_if_exists(
    job: &Job,
    filename: &str,
    sha: Option<&str>,
    client: reqwest::Client,
) -> Result<Option<IconFileWithName>> {
    let Some(sha) = sha else { return Ok(None) };
    let rt = Runtime::new()?;
    let raw = rt.block_on(async {
        download_url(&job.installation, &job.repo, filename, sha, client)
            .await
            .wrap_err_with(|| format!("Failed to download file {filename:?}"))
    })?;

    let mut hasher = DefaultHasher::new();
    raw.hash(&mut hasher);
    let hash = hasher.finish();

    Ok(Some(IconFileWithName {
        full_name: filename.to_string(),
        sha: sha.to_string(),
        hash,
        icon: IconFile::from_bytes(&raw)
            .wrap_err_with(|| format!("IconFile::from_bytes failed for {filename:?}"))?,
    }))
}
