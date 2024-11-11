//! Tools for fetching video streams from YouTube.

use crate::error::Error;
use crate::executor::Executor;
use crate::fetcher::Fetcher;
use crate::model::format::Format;
use crate::model::Video;
use crate::utils::file_system;
use crate::{utils, Youtube};
use std::path::PathBuf;
use std::time::Duration;

impl Youtube {
    /// Fetch the video information from the given URL.
    ///
    /// # Arguments
    ///
    /// * `url` - The URL of the video to fetch.
    ///
    /// # Errors
    ///
    /// This function will return an error if the video information could not be fetched.
    ///
    /// # Examples
    ///
    /// ```rust, no_run
    /// # use yt_dlp::Youtube;
    /// # use std::path::PathBuf;
    /// # use yt_dlp::fetcher::deps::Libraries;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let libraries_dir = PathBuf::from("libs");
    /// # let output_dir = PathBuf::from("output");
    /// # let youtube = libraries_dir.join("yt-dlp");
    /// # let ffmpeg = libraries_dir.join("ffmpeg");
    /// # let libraries = Libraries::new(youtube, ffmpeg);
    /// let fetcher = Youtube::new(libraries, output_dir)?;
    ///
    /// let url = String::from("https://www.youtube.com/watch?v=dQw4w9WgXcQ");
    /// let video = fetcher.fetch_video_infos(url).await?;
    /// # Ok(())
    /// # }
    /// ```
    #[cfg_attr(feature = "tracing", tracing::instrument(level = "debug"))]
    pub async fn fetch_video_infos(&self, url: String) -> crate::error::Result<Video> {
        #[cfg(feature = "tracing")]
        tracing::debug!("Fetching video information for {}", url);

        let download_args = vec!["--no-progress", "--dump-json", &url];

        let mut final_args = self.args.clone();
        final_args.append(&mut utils::to_owned(download_args));

        let executor = Executor {
            executable_path: self.libraries.youtube.clone(),
            timeout: Duration::from_secs(30),
            args: final_args,
        };

        let output = executor.execute().await?;
        let video: Video = serde_json::from_str(&output.stdout).map_err(Error::Serde)?;

        Ok(video)
    }

    /// Fetch the video from the given URL, download it (video with audio) and returns its path.
    /// Be careful, this function may take a while to execute.
    ///
    /// # Arguments
    ///
    /// * `url` - The URL of the video to download.
    /// * `output` - The name of the file to save the video to.
    ///
    /// # Errors
    ///
    /// This function will return an error if the video could not be fetched or downloaded.
    ///
    /// # Examples
    ///
    /// ```rust, no_run
    /// # use yt_dlp::Youtube;
    /// # use std::path::PathBuf;
    /// # use yt_dlp::fetcher::deps::Libraries;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let libraries_dir = PathBuf::from("libs");
    /// # let output_dir = PathBuf::from("output");
    /// # let youtube = libraries_dir.join("yt-dlp");
    /// # let ffmpeg = libraries_dir.join("ffmpeg");
    /// # let libraries = Libraries::new(youtube, ffmpeg);
    /// let fetcher = Youtube::new(libraries, output_dir)?;
    ///
    /// let url = String::from("https://www.youtube.com/watch?v=dQw4w9WgXcQ");
    /// let video_path = fetcher.download_video_from_url(url, "my-video.mp4").await?;
    /// # Ok(())
    /// # }
    /// ```
    #[cfg_attr(feature = "tracing", tracing::instrument(level = "debug"))]
    pub async fn download_video_from_url(
        &self,
        url: String,
        output: impl AsRef<str>,
    ) -> crate::error::Result<PathBuf> {
        #[cfg(feature = "tracing")]
        tracing::debug!("Downloading video from {}", url);

        let video = self.fetch_video_infos(url.clone()).await?;

        self.download_video(&video, output).await
    }

    /// Downloads the video (with its audio), and returns its path.
    /// Be careful, this function may take a while to execute.
    ///
    /// # Arguments
    ///
    /// * `video` - The video to download.
    /// * `output` - The name of the file to save the video to.
    ///
    /// # Errors
    ///
    /// This function will return an error if the video could not be fetched or downloaded.
    ///
    /// # Examples
    ///
    /// ```rust, no_run
    /// # use yt_dlp::Youtube;
    /// # use std::path::PathBuf;
    /// # use yt_dlp::fetcher::deps::Libraries;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let libraries_dir = PathBuf::from("libs");
    /// # let output_dir = PathBuf::from("output");
    /// # let youtube = libraries_dir.join("yt-dlp");
    /// # let ffmpeg = libraries_dir.join("ffmpeg");
    /// # let libraries = Libraries::new(youtube, ffmpeg);
    /// let fetcher = Youtube::new(libraries, output_dir)?;
    ///
    /// let url = String::from("https://www.youtube.com/watch?v=dQw4w9WgXcQ");
    /// let video = fetcher.fetch_video_infos(url).await?;
    ///
    /// let video_path = fetcher.download_video(&video, "my-video.mp4").await?;
    /// # Ok(())
    /// # }
    /// ```
    #[cfg_attr(feature = "tracing", tracing::instrument(level = "debug"))]
    pub async fn download_video(
        &self,
        video: &Video,
        output: impl AsRef<str>,
    ) -> crate::error::Result<PathBuf> {
        #[cfg(feature = "tracing")]
        tracing::debug!("Downloading video {}", video.title);

        let output_path = self.output_dir.join(output.as_ref());
        let file_name = file_system::try_without_extension(output_path.clone())?;

        let audio_name = format!("audio-{}.mp3", file_name.clone());
        self.download_audio_stream(video, &audio_name).await?;

        let video_name = format!("video-{}.mp4", file_name.clone());
        self.download_video_stream(video, &video_name).await?;

        self.combine_audio_and_video(&audio_name, &video_name, output)
            .await
    }

    /// Fetch the video from the given URL, download it and returns its path.
    /// Be careful, this function may take a while to execute.
    ///
    /// # Arguments
    ///
    /// * `url` - The URL of the video to download.
    /// * `output` - The name of the file to save the video to.
    ///
    /// # Errors
    ///
    /// This function will return an error if the video could not be fetched or downloaded.
    ///
    /// # Examples
    ///
    /// ```rust, no_run
    /// # use yt_dlp::Youtube;
    /// # use std::path::PathBuf;
    /// # use yt_dlp::fetcher::deps::Libraries;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let libraries_dir = PathBuf::from("libs");
    /// # let output_dir = PathBuf::from("output");
    /// # let youtube = libraries_dir.join("yt-dlp");
    /// # let ffmpeg = libraries_dir.join("ffmpeg");
    /// # let libraries = Libraries::new(youtube, ffmpeg);
    /// let fetcher = Youtube::new(libraries, output_dir)?;
    ///
    /// let url = String::from("https://www.youtube.com/watch?v=dQw4w9WgXcQ");
    /// let video_path = fetcher.download_video_stream_from_url(url, "my-video-stream.mp4").await?;
    /// # Ok(())
    /// # }
    /// ```
    #[cfg_attr(feature = "tracing", tracing::instrument(level = "debug"))]
    pub async fn download_video_stream_from_url(
        &self,
        url: String,
        output: impl AsRef<str>,
    ) -> crate::error::Result<PathBuf> {
        #[cfg(feature = "tracing")]
        tracing::debug!("Downloading video stream from {}", url);

        let video = self.fetch_video_infos(url).await?;

        self.download_video_stream(&video, output).await
    }

    /// Download the video only, and returns its path.
    /// Be careful, this function may take a while to execute.
    ///
    /// # Arguments
    ///
    /// * `video` - The video to download.
    /// * `output` - The name of the file to save the video to.
    ///
    /// # Errors
    ///
    /// This function will return an error if the video could not be fetched or downloaded.
    ///
    /// # Examples
    ///
    /// ```rust, no_run
    /// # use yt_dlp::Youtube;
    /// # use std::path::PathBuf;
    /// # use yt_dlp::fetcher::deps::Libraries;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let libraries_dir = PathBuf::from("libs");
    /// # let output_dir = PathBuf::from("output");
    /// # let youtube = libraries_dir.join("yt-dlp");
    /// # let ffmpeg = libraries_dir.join("ffmpeg");
    /// # let libraries = Libraries::new(youtube, ffmpeg);
    /// let fetcher = Youtube::new(libraries, output_dir)?;
    ///
    /// let url = String::from("https://www.youtube.com/watch?v=dQw4w9WgXcQ");
    /// let video = fetcher.fetch_video_infos(url).await?;
    ///
    /// let video_path = fetcher.download_video_stream(&video, "my-video-stream.mp4").await?;
    /// # Ok(())
    /// # }
    /// ```
    #[cfg_attr(feature = "tracing", tracing::instrument(level = "debug"))]
    pub async fn download_video_stream(
        &self,
        video: &Video,
        output: impl AsRef<str>,
    ) -> crate::error::Result<PathBuf> {
        #[cfg(feature = "tracing")]
        tracing::debug!("Downloading video stream {}", video.title);

        let best_video = video
            .best_video_format()
            .ok_or(Error::Video("No video format available".to_string()))?;

        self.download_format(best_video, output).await
    }

    /// Fetch the audio from the given URL, download it and returns its path.
    /// Be careful, this function may take a while to execute.
    ///
    /// # Arguments
    ///
    /// * `url` - The URL of the video to download.
    /// * `output` - The name of the file to save the audio to.
    ///
    /// # Errors
    ///
    /// This function will return an error if the video could not be fetched or downloaded.
    ///
    /// # Examples
    ///
    /// ```rust, no_run
    /// # use yt_dlp::Youtube;
    /// # use std::path::PathBuf;
    /// # use yt_dlp::fetcher::deps::Libraries;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let libraries_dir = PathBuf::from("libs");
    /// # let output_dir = PathBuf::from("output");
    /// # let youtube = libraries_dir.join("yt-dlp");
    /// # let ffmpeg = libraries_dir.join("ffmpeg");
    /// # let libraries = Libraries::new(youtube, ffmpeg);
    /// let fetcher = Youtube::new(libraries, output_dir)?;
    ///
    /// let url = String::from("https://www.youtube.com/watch?v=dQw4w9WgXcQ");
    /// let audio_path = fetcher.download_audio_stream_from_url(url, "my-audio-stream.mp3").await?;
    /// # Ok(())
    /// # }
    /// ```
    #[cfg_attr(feature = "tracing", tracing::instrument(level = "debug"))]
    pub async fn download_audio_stream_from_url(
        &self,
        url: String,
        output: impl AsRef<str>,
    ) -> crate::error::Result<PathBuf> {
        #[cfg(feature = "tracing")]
        tracing::debug!("Downloading audio stream from {}", url);

        let video = self.fetch_video_infos(url).await?;

        self.download_audio_stream(&video, output).await
    }

    /// Downloads the audio, and returns its path.
    /// Be careful, this function may take a while to execute.
    ///
    /// # Arguments
    ///
    /// * `video` - The video to download.
    /// * `output` - The name of the file to save the audio to.
    ///
    /// # Errors
    ///
    /// This function will return an error if the video could not be fetched or downloaded.
    ///
    /// # Examples
    ///
    /// ```rust, no_run
    /// # use yt_dlp::Youtube;
    /// # use std::path::PathBuf;
    /// # use yt_dlp::fetcher::deps::Libraries;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let libraries_dir = PathBuf::from("libs");
    /// # let output_dir = PathBuf::from("output");
    /// # let youtube = libraries_dir.join("yt-dlp");
    /// # let ffmpeg = libraries_dir.join("ffmpeg");
    /// # let libraries = Libraries::new(youtube, ffmpeg);
    /// let fetcher = Youtube::new(libraries, output_dir)?;
    ///
    /// let url = String::from("https://www.youtube.com/watch?v=dQw4w9WgXcQ");
    /// let video = fetcher.fetch_video_infos(url).await?;
    ///
    /// let audio_path = fetcher.download_audio_stream(&video, "my-audio-stream.mp3").await?;
    /// # Ok(())
    /// # }
    /// ```
    #[cfg_attr(feature = "tracing", tracing::instrument(level = "debug"))]
    pub async fn download_audio_stream(
        &self,
        video: &Video,
        output: impl AsRef<str>,
    ) -> crate::error::Result<PathBuf> {
        #[cfg(feature = "tracing")]
        tracing::debug!("Downloading audio stream {}", video.title);

        let best_audio = video
            .best_audio_format()
            .ok_or(Error::Video("No audio format available".to_string()))?;

        self.download_format(best_audio, output).await
    }

    /// Downloads a specific format, and returns its path.
    /// Be careful, this function may take a while to execute.
    ///
    /// # Arguments
    ///
    /// * `format` - The format to download.
    /// * `output` - The name of the file to save the format to.
    ///
    /// # Errors
    ///
    /// This function will return an error if the video could not be downloaded.
    ///
    /// # Examples
    ///
    /// ```rust, no_run
    /// # use yt_dlp::Youtube;
    /// # use std::path::PathBuf;
    /// # use yt_dlp::fetcher::deps::Libraries;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let libraries_dir = PathBuf::from("libs");
    /// # let output_dir = PathBuf::from("output");
    /// # let youtube = libraries_dir.join("yt-dlp");
    /// # let ffmpeg = libraries_dir.join("ffmpeg");
    /// # let libraries = Libraries::new(youtube, ffmpeg);
    /// let fetcher = Youtube::new(libraries, output_dir)?;
    ///
    /// let url = String::from("https://www.youtube.com/watch?v=dQw4w9WgXcQ");
    /// let video = fetcher.fetch_video_infos(url).await?;
    ///
    /// let video_format = video.best_video_format().unwrap();
    /// let format_path = fetcher.download_format(&video_format, "my-video-stream.mp4").await?;
    ///
    /// let audio_format = video.worst_audio_format().unwrap();
    /// let audio_path = fetcher.download_format(&audio_format, "my-audio-stream.mp3").await?;
    /// # Ok(())
    /// # }
    /// ```
    #[cfg_attr(feature = "tracing", tracing::instrument(level = "debug"))]
    pub async fn download_format(
        &self,
        format: &Format,
        output: impl AsRef<str>,
    ) -> crate::error::Result<PathBuf> {
        #[cfg(feature = "tracing")]
        tracing::debug!("Downloading format {}", format.download_info.url);

        let path = self.output_dir.join(output.as_ref());
        let url = format.download_info.url.clone();

        let fetcher = Fetcher::new(&url);
        fetcher.fetch_asset(path.clone()).await?;

        Ok(path)
    }
}
