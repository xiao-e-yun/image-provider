use clap::Parser;
use derive_builder::Builder;
use fast_image_resize::{FilterType, ResizeAlg};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize, Parser, Builder)]
#[builder(pattern = "owned")]
pub struct ResizeConfig {

    /// `lanczos3`  
    /// `gaussian`  
    /// `catmull-rom`  
    /// `hamming`  
    /// `mitchell`  
    /// `bilinear`  
    /// `box`  
    #[clap(name="resize-images-filter-type", long, default_value = "lanczos3" )]
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
    #[clap(name="resize-images-algorithm", long, default_value = "interpolation" )]
    pub algorithm: String,
}

impl ResizeConfig {
    pub fn builder() -> ResizeConfigBuilder {
        ResizeConfigBuilder {
            filter_type: Some("lanczos3".into()),
            algorithm: Some("interpolation".into()),
        }
    }

    pub fn resize_algorithm(&self) -> ResizeAlg {
        let filter_type = match self.filter_type.as_str() {
            "lanczos3" => FilterType::Lanczos3,
            "gaussian" => FilterType::Gaussian,
            "catmull-rom" => FilterType::CatmullRom,
            "hamming" => FilterType::Hamming,
            "mitchell" => FilterType::Mitchell,
            "bilinear" => FilterType::Bilinear,
            "box" => FilterType::Box,
            _ => panic!("Unsupported filter type"),
        };

        match self.algorithm.as_str() {
            "super-sampling8x" => ResizeAlg::SuperSampling(filter_type, 8),
            "super-sampling4x" => ResizeAlg::SuperSampling(filter_type, 4),
            "super-sampling2x" => ResizeAlg::SuperSampling(filter_type, 2),
            "interpolation" => ResizeAlg::Interpolation(filter_type),
            "convolution" => ResizeAlg::Convolution(filter_type),
            "nearest" => ResizeAlg::Nearest,
            _ => panic!("Unsupported resize algorithm"),
        }
    }
}
