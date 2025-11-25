use clap::Parser;

#[derive(Parser, Debug)]
pub struct PullOpts {
    /// Specify the image name
    #[arg(long, short)]
    pub image: String,
    /// Specify the tag of the image
    pub tag: String,
}

