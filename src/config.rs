use std::path::PathBuf;

use clap::Parser;
use derive_builder::Builder;
use fast_image_resize::{FilterType, ResizeAlg};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize, Parser)]
pub struct Config {
    #[clap(long, short, default_value = "3000")]
    pub port: u16,
    #[clap(flatten)]
    pub resize: ResizeConfig,
}

#[derive(Debug, Clone, Deserialize, Parser, Builder)]
#[builder(setter(into))]
pub struct ResizeConfig {

    #[clap(long="resize-path", default_value = "." )]
    #[builder(default = "\".\".into()")]
    pub path: PathBuf,

    /// `lanczos3`  
    /// `gaussian`  
    /// `catmull-rom`  
    /// `hamming`  
    /// `mitchell`  
    /// `bilinear`  
    /// `box`  
    #[clap(long="resize-filter-type", default_value = "bilinear" )]
    #[builder(default = "\"bilinear\".into()")]
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
    #[builder(default = "\"interpolation\".into()")]
    pub algorithm: String,
}

impl ResizeConfig {
    pub fn resize_algorithm(&self) -> ResizeAlg {
        let filter_type = match self.filter_type.as_str() {
            "lanczos3" => FilterType::Lanczos3,
            "gaussian" => FilterType::Gaussian,
            "catmull-rom" => FilterType::CatmullRom,
            "hamming" => FilterType::Hamming,
            "mitchell" => FilterType::Mitchell,
            "bilinear" => FilterType::Bilinear,
            "box" => FilterType::Box,
            _ => FilterType::Lanczos3,
        };

        match self.algorithm.as_str() {
            "super-sampling8x" => ResizeAlg::SuperSampling(filter_type, 8),
            "super-sampling4x" => ResizeAlg::SuperSampling(filter_type, 4),
            "super-sampling2x" => ResizeAlg::SuperSampling(filter_type, 2),
            "interpolation" => ResizeAlg::Interpolation(filter_type),
            "convolution" => ResizeAlg::Convolution(filter_type),
            "nearest" => ResizeAlg::Nearest,
            _ => ResizeAlg::Interpolation(filter_type),
        }
    }
}
