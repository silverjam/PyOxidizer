// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Resolve details about the PyOxidizer execution environment.

use {
    crate::project_layout::PyembedLocation,
    anyhow::{anyhow, Result},
    git2::{Commit, Repository},
    lazy_static::lazy_static,
    std::env,
    std::path::{Path, PathBuf},
};

/// Canonical Git repository for PyOxidizer.
const CANONICAL_GIT_REPO_URL: &str = "https://github.com/silverjam/PyOxidizer.git";

/// Root Git commit for PyOxidizer.
const ROOT_COMMIT: &str = "66e694ca1466a9056047a359396b79466d3051b7";

/// Git commit this build of PyOxidizer was produced with.
pub const BUILD_GIT_COMMIT: &str = env!("GIT_COMMIT");

/// Semantic version for this build of PyOxidizer. Can correspond to a Git
/// tag or version string from Cargo.toml.
pub const BUILD_SEMVER: &str = env!("VERGEN_SEMVER");

lazy_static! {
    /// Version string of PyOxidizer.
    pub static ref PYOXIDIZER_VERSION: String = {
        if env!("CARGO_PKG_VERSION").ends_with("-pre") {
            format!("{}-{}", env!("CARGO_PKG_VERSION"), BUILD_GIT_COMMIT)
        } else {
            env!("CARGO_PKG_VERSION").to_string()
        }
    };

    /// Minimum version of Rust required to build PyOxidizer applications.
    ///
    // Remember to update the CI configuration in ci/azure-pipelines-template.yml
    // when this changes.
    pub static ref MINIMUM_RUST_VERSION: semver::Version = semver::Version::new(1, 40, 0);

    /// Target triples for Linux.
    pub static ref LINUX_TARGET_TRIPLES: Vec<&'static str> = vec![
        "x86_64-unknown-linux-gnu",
        "x86_64-unknown-linux-musl",
    ];

    /// Target triples for macOS.
    pub static ref MACOS_TARGET_TRIPLES: Vec<&'static str> = vec![
        "x86_64-apple-darwin",
    ];

    /// Target triples for Windows.
    pub static ref WINDOWS_TARGET_TRIPLES: Vec<&'static str> = vec![
        "i686-pc-windows-gnu",
        "i686-pc-windows-msvc",
        "x86_64-pc-windows-gnu",
        "x86_64-pc-windows-msvc",
    ];
}

/// Find the root Git commit given a starting Git commit.
///
/// This just walks parents until it gets to a commit without any.
fn find_root_git_commit(commit: Commit) -> Commit {
    let mut current = commit;

    while current.parent_count() != 0 {
        current = current.parents().next().unwrap();
    }

    current
}

pub fn canonicalize_path(path: &Path) -> Result<PathBuf, std::io::Error> {
    let mut p = path.canonicalize()?;

    // Strip \\?\ prefix on Windows and replace \ with /, which is valid.
    if cfg!(windows) {
        let mut s = p.display().to_string().replace("\\", "/");
        if s.starts_with("//?/") {
            s = s[4..].to_string();
        }

        p = PathBuf::from(s);
    }

    Ok(p)
}

/// Describes the location of the PyOxidizer source files.
pub enum PyOxidizerSource {
    /// A local filesystem path.
    LocalPath { path: PathBuf },

    /// A Git repository somewhere. Defined by a Git remote URL and a commit string.
    GitUrl {
        url: String,
        commit: Option<String>,
        tag: Option<String>,
    },
}

/// Describes the PyOxidizer run-time environment.
pub struct Environment {
    /// Where a copy of PyOxidizer can be obtained from.
    pub pyoxidizer_source: PyOxidizerSource,
}

impl Environment {
    /// Determine the location of the pyembed crate given a run-time environment.
    ///
    /// If running from a PyOxidizer Git repository, we reference the pyembed
    /// crate within the PyOxidizer Git repository. Otherwise we use the pyembed
    /// crate from the package registry.
    ///
    /// There is room to reference a Git repository+commit. But this isn't implemented
    /// yet.
    pub fn as_pyembed_location(&self) -> PyembedLocation {
        match &self.pyoxidizer_source {
            PyOxidizerSource::LocalPath { path } => {
                PyembedLocation::Path(canonicalize_path(&path.join("pyembed")).unwrap())
            }
            PyOxidizerSource::GitUrl { url, commit, .. } => match commit {
                Some(commit) => PyembedLocation::Git(url.clone(), commit.clone()),
                None => PyembedLocation::Version(env!("CARGO_PKG_VERSION").to_string()),
            },
        }
    }

    /// Obtain a string to be used as the long form version info for the executable.
    pub fn version_long(&self) -> String {
        format!(
            "{}\ncommit: {}\nsource: {}\npyembed crate location: {}",
            env!("CARGO_PKG_VERSION"),
            BUILD_GIT_COMMIT,
            match &self.pyoxidizer_source {
                PyOxidizerSource::LocalPath { path } => {
                    format!("{}", path.display())
                }
                PyOxidizerSource::GitUrl { url, .. } => {
                    url.clone()
                }
            },
            self.as_pyembed_location().cargo_manifest_fields(),
        )
    }
}

/// Obtain a PyOxidizerSource pointing to the GitUrl this binary was built with.
pub fn built_git_url() -> PyOxidizerSource {
    let commit = match BUILD_GIT_COMMIT {
        // Can happen when not run from a Git checkout (such as installing
        // from a crate).
        "" => None,
        // Can happen if build script could not find Git repository.
        "UNKNOWN" => None,
        value => Some(value.to_string()),
    };

    // Commit and tag should be mutually exclusive. BUILD_SEMVER could be
    // derived by a Git tag in some circumstances. More commonly it is
    // derived from Cargo.toml. The Git tags have ``v`` prefixes.
    let tag = if commit.is_some() {
        None
    } else if !BUILD_SEMVER.starts_with('v') {
        Some("v".to_string() + BUILD_SEMVER)
    } else {
        Some(BUILD_SEMVER.to_string())
    };

    PyOxidizerSource::GitUrl {
        url: CANONICAL_GIT_REPO_URL.to_owned(),
        commit,
        tag,
    }
}

pub fn resolve_environment() -> Result<Environment> {
    let exe_path = PathBuf::from(
        env::current_exe()?
            .parent()
            .ok_or_else(|| anyhow!("could not resolve parent of current exe"))?,
    );

    let pyoxidizer_source = match Repository::discover(&exe_path) {
        Ok(repo) => {
            let head = repo.head().unwrap();
            let commit = head.peel_to_commit().unwrap();
            let root = find_root_git_commit(commit.clone());

            if root.id().to_string() == ROOT_COMMIT {
                PyOxidizerSource::LocalPath {
                    path: canonicalize_path(
                        repo.workdir()
                            .ok_or_else(|| anyhow!("unable to resolve Git workdir"))?,
                    )?,
                }
            } else {
                // The pyoxidizer binary is in a directory that is in a Git repo that isn't
                // pyoxidizer's. This could happen if running `pyoxidizer` from another
                // project's Git repository. This commonly happens when running
                // pyoxidizer as a library from a build script. Fall back to
                // returning info embedded in the build.
                built_git_url()
            }
        }
        Err(_) => {
            // We're not running from a Git repo. Point to the canonical repo for the Git commit
            // baked into the binary.
            // TODO detect builds from forks via build.rs environment variable.
            built_git_url()
        }
    };

    Ok(Environment { pyoxidizer_source })
}
