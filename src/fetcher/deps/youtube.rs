//! Fetch the latest release of 'yt-dlp' from a GitHub repository.

use crate::error::{Error, Result};
use crate::fetcher::model::{Asset, Release, WantedRelease};
use crate::fetcher::platform::Architecture;
use crate::fetcher::platform::Platform;
use crate::fetcher::Fetcher;
use derive_more::Display;

/// The GitHub fetcher is responsible for fetching the latest release of 'yt-dlp' from a GitHub repository.
/// It can also select the correct asset for the current platform and architecture.
///
/// # Example
///
/// ```rust, no_run
/// # use std::path::PathBuf;
/// # use yt_dlp::fetcher::deps::youtube::GitHubFetcher;
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let fetcher = GitHubFetcher::new("yt-dlp", "yt-dlp");
/// let release = fetcher.fetch_release(None).await?;
///
/// let destination = PathBuf::from("yt-dlp");
/// release.download(destination).await?;
/// # Ok(())
/// # }
#[derive(Debug, Display)]
#[display("Github Fetcher: {}/{}", owner, repo)]
pub struct GitHubFetcher {
    /// The owner or organization of the GitHub repository.
    owner: String,
    /// The name of the GitHub repository.
    repo: String,
}

impl GitHubFetcher {
    /// Create a new fetcher for the given GitHub repository.
    ///
    /// # Arguments
    ///
    /// * `owner` - The owner of the GitHub repository.
    /// * `repo` - The name of the GitHub repository.
    pub fn new(owner: &str, repo: &str) -> Self {
        Self {
            owner: owner.to_string(),
            repo: repo.to_string(),
        }
    }

    /// Fetch the latest release of the GitHub repository, and select the correct asset for the current platform and architecture.
    ///
    /// # Arguments
    ///
    /// * `auth_token` - An optional GitHub personal access token to authenticate the request.
    #[cfg_attr(feature = "tracing", tracing::instrument(level = "debug", skip(self)))]
    pub async fn fetch_release(&self, auth_token: Option<String>) -> Result<WantedRelease> {
        #[cfg(feature = "tracing")]
        tracing::debug!("Fetching latest release for {}/{}", self.owner, self.repo);

        let platform = Platform::detect();
        let architecture = Architecture::detect();

        self.fetch_release_for_platform(platform, architecture, auth_token)
            .await
    }

    /// Fetch the latest release of the GitHub repository, and select the correct asset for the given platform and architecture.
    ///
    /// # Arguments
    ///
    /// * `platform` - The platform to fetch the release for.
    /// * `architecture` - The architecture to fetch the release for.
    /// * `auth_token` - An optional GitHub personal access token to authenticate the request.
    #[cfg_attr(feature = "tracing", tracing::instrument(level = "debug", skip(self)))]
    pub async fn fetch_release_for_platform(
        &self,
        platform: Platform,
        architecture: Architecture,
        auth_token: Option<String>,
    ) -> Result<WantedRelease> {
        #[cfg(feature = "tracing")]
        tracing::debug!(
            "Fetching latest release for {}/{} for platform: {:?}, architecture: {:?}",
            self.owner,
            self.repo,
            platform,
            architecture
        );

        let release = self.fetch_latest_release(auth_token).await?;

        let asset = Self::select_asset(&platform, &architecture, &release)
            .ok_or(Error::Github(platform, architecture))?;

        Ok(WantedRelease {
            asset_name: asset.name.clone(),
            asset_url: asset.download_url.clone(),
        })
    }

    /// Fetch the latest release of the GitHub repository.
    ///
    /// # Arguments
    ///
    /// * `auth_token` - An optional GitHub personal access token to authenticate the request.
    #[cfg_attr(feature = "tracing", tracing::instrument(level = "debug", skip(self)))]
    pub async fn fetch_latest_release(&self, auth_token: Option<String>) -> Result<Release> {
        #[cfg(feature = "tracing")]
        tracing::debug!("Fetching latest release for {}/{}", self.owner, self.repo);

        let url = format!(
            "https://api.github.com/repos/{}/{}/releases/latest",
            self.owner, self.repo
        );

        let fetcher = Fetcher::new(&url);
        let response = fetcher.fetch_json(auth_token).await?;

        let release: Release = serde_json::from_value(response)?;
        Ok(release)
    }

    /// Select the correct asset from the release for the given platform and architecture.
    ///
    /// # Arguments
    ///
    /// * `platform` - The platform to select the asset for.
    /// * `architecture` - The architecture to select the asset for.
    /// * `release` - The release to select the asset from.
    #[cfg_attr(feature = "tracing", tracing::instrument(level = "debug"))]
    pub fn select_asset<'a>(
        platform: &Platform,
        architecture: &Architecture,
        release: &'a Release,
    ) -> Option<&'a Asset> {
        const BASE_NAME: &str = "yt-dlp";

        #[cfg(feature = "tracing")]
        tracing::debug!(
            "Selecting asset for platform: {:?}, architecture: {:?}",
            platform,
            architecture
        );

        let assets = &release.assets;
        let asset = assets.iter().find(|asset| {
            let name = &asset.name;

            match (platform, architecture) {
                (Platform::Windows, Architecture::X64) => {
                    name.contains(&format!("{}.exe", BASE_NAME))
                }
                (Platform::Windows, Architecture::X86) => {
                    name.contains(&format!("{}_x86.exe", BASE_NAME))
                }

                (Platform::Linux, Architecture::X64) => {
                    name.contains(&format!("{}_linux", BASE_NAME))
                }
                (Platform::Linux, Architecture::Armv7l) => {
                    name.contains(&format!("{}_linux_armv7l", BASE_NAME))
                }
                (Platform::Linux, Architecture::Aarch64) => {
                    name.contains(&format!("{}_linux_aarch64", BASE_NAME))
                }

                (Platform::Mac, _) => name.contains(&format!("{}_macos", BASE_NAME)),

                _ => false,
            }
        });

        asset
    }
}
