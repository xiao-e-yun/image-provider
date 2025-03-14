use std::path::PathBuf;

use clap::Parser;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize, Parser)]
pub struct Config {
    #[clap(default_value = ".")]
    pub path: PathBuf,
    #[clap(long, short, default_value = "3000")]
    pub port: u16,
    #[clap(flatten)]
    pub resize: ResizeConfig,
}

#[derive(Debug, Clone, Deserialize, Parser)]
pub struct ResizeConfig {

    /// the maximum cache size by number of images
    /// 0 to disable
    #[clap(long = "resize-cache", default_value = "200")]
    pub cache: usize,

    /// `lanczos3`  
    /// `gaussian`  
    /// `catmull-rom`  
    /// `hamming`  
    /// `mitchell`  
    /// `bilinear`  
    /// `box`  
    #[clap(long="resize-filter-type", default_value = "bilinear" )]
    pub filter_type: String,

    /// Slow <-
    /// `super-sampling8x`  
    /// `super-sampling4x`  
    /// `super-sampling2x`  
    /// `convolution`  
    /// `interpolation`  
    /// `nearest`  
    /// -> Fast
    /// (nearest will ignore filter_type)
    #[clap(long="resize-algorithm", default_value = "interpolation" )]
    pub algorithm: String,
}
