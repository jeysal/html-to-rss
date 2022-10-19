use clap::Parser;

/// An opinionated CLI tool that reads HTML files and writes/updates an RSS feed file
#[derive(Parser, Debug)]
#[command(about, long_about = None, version)]
pub struct Args {
    /// The RSS file to read the current feed from and write the new feed to.
    #[arg(long, default_value = "index.rss")]
    pub feed: String,

    /// The title of the feed as a whole.
    #[arg(short, long)]
    pub title: Option<String>,

    /// The description of the feed as a whole.
    #[arg(short, long)]
    pub description: Option<String>,

    /// The public URL where the directory in which the RSS file is located will be served.
    #[arg(short, long)]
    pub base_url: Option<String>,

    /// The overall language of the feed.
    #[arg(long)]
    pub language: Option<String>,

    /// The path to the favicon file. Will be appended to the base URL.
    #[arg(long, default_value = "favicon.png")]
    pub favicon: String,

    /// The paths to pages to add to the feed where not already preset.
    #[arg(required = true)]
    pub pages: Vec<String>,
}
