use std::{io::Cursor, path::PathBuf, sync::Arc};

use axum::{
    extract::{Path, Query, State},
    http::{
        header::{CACHE_CONTROL, CONTENT_TYPE, X_CONTENT_TYPE_OPTIONS},
        HeaderMap, HeaderValue, StatusCode,
    },
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use axum_extra::{headers::Range, TypedHeader};
use axum_range::{KnownSize, Ranged};
use bytes::Bytes;
use cached::{Cached, TimedSizedCache};
use fast_image_resize::{images::Image, IntoImageView, ResizeAlg, ResizeOptions, Resizer};
use image::{
    codecs::{jpeg::JpegEncoder, png::PngEncoder, webp::WebPEncoder},
    load_from_memory, ColorType, DynamicImage, ImageEncoder, ImageFormat,
};
use log::{debug, trace};
use mime_guess::MimeGuess;
use serde::Deserialize;
use tokio::{fs::File, io::AsyncReadExt, sync::Mutex};

pub mod config;

pub use config::*;

pub fn get_images_router(root: PathBuf, config: ResizeConfig) -> Router {
    const CACHE_LIFESPAN: u64 = 24 * 60 * 60; // 1 days in seconds
    let cache = TimedSizedCache::with_size_and_lifespan_and_refresh(
        config.cache_size,
        CACHE_LIFESPAN,
        true,
    );
    let cache = Arc::new(Mutex::new(cache));

    Router::new()
        .route("/{*path}", get(provide_images))
        .route(
            "/",
            get(|| async { (StatusCode::NOT_FOUND, "File not found".to_string()) }),
        )
        .with_state(ImageState {
            root,
            config,
            cache,
        })
}

type Error = (StatusCode, String);
type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Clone)]
struct ImageState {
    root: PathBuf,
    config: ResizeConfig,
    cache: Arc<Mutex<TimedSizedCache<(PathBuf, ImageQuery), Bytes>>>,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq, Hash)]
pub struct ImageQuery {
    pub output: Option<String>,
    pub dpr: Option<u32>,
    pub w: Option<u32>,
    pub h: Option<u32>,
}

impl ImageQuery {
    fn output(&self) -> Result<Option<ImageFormat>> {
        self.output
            .as_ref()
            .map(|ext| {
                find_image_mime(MimeGuess::from_ext(ext)).ok_or((
                    StatusCode::BAD_REQUEST,
                    format!("Unsupported output format: {ext}"),
                ))
            })
            .transpose()
    }

    fn size(&self) -> (Option<u32>, Option<u32>) {
        (self.w, self.h)
    }

    fn dpr(&self) -> u32 {
        self.dpr.unwrap_or(1).clamp(1, 3)
    }
}

async fn provide_images(
    State(ImageState {
        root,
        config,
        cache,
    }): State<ImageState>,
    Query(query): Query<ImageQuery>,
    Path(path): Path<PathBuf>,
    range: Option<TypedHeader<Range>>,
) -> Result<Response> {
    let (path, raw_mime) = get_path_and_mime(root, path)?;
    let dst_mime = query.output()?.unwrap_or(raw_mime);
    let (dst_width, dst_height) = query.size();
    let dpr = query.dpr();

    debug!(
        "Processing image: {path:?} to mime: {dst_mime:?}, size: {:?}x{:?}, dpr: {dpr}",
        dst_width.unwrap_or(0), dst_height.unwrap_or(0)
    );

    let range = range.map(|TypedHeader(range)| range);
    let headers = get_response_headers(&dst_mime);

    // If no resizing is needed, serve the original file directly
    let eq_raw = dst_width.is_none() && dst_height.is_none() && dpr == 1 && raw_mime == dst_mime;
    let exclude = matches!(raw_mime, image::ImageFormat::Ico | image::ImageFormat::Gif);
    if eq_raw || exclude {
        trace!("Serving original image: {path:?}");
        let file = load_file(&path).await?;
        let body = KnownSize::file(file).await.unwrap();
        let ranged = Ranged::new(range, body);
        return Ok((headers, ranged).into_response());
    }

    if let Some(cached) = cache.lock().await.cache_get(&(path.clone(), query.clone())) {
        trace!(
            "Serving cached image: {path:?} (mime: {dst_mime:?}, size {:?}x{:?}, dpr: {dpr})",
            dst_width.unwrap_or(0), dst_height.unwrap_or(0)
        );
        let body = KnownSize::seek(Cursor::new(cached.clone())).await.unwrap();
        return Ok((headers, Ranged::new(range, body)).into_response());
    }

    let file = load_file(&path).await?;
    let src_image = load_image(file).await?;

    let (dst_width, dst_height) = get_output_size(
        (src_image.width(), src_image.height()),
        (dst_width, dst_height),
        dpr,
    );

    let mut dst_image = Image::new(dst_width, dst_height, src_image.pixel_type().unwrap());
    resize_image(&config, &src_image, &mut dst_image)?;

    let bytes = encode_image(dst_mime, &dst_image, src_image.color())?;

    // Cache the processed image
    cache
        .lock()
        .await
        .cache_set((path.clone(), query), bytes.clone());

    let body = KnownSize::seek(Cursor::new(bytes)).await.unwrap();

    trace!(
        "Serving processed image: {path:?} (mime: {dst_mime:?}, size {dst_width:?}x{dst_height:?}, dpr: {dpr})"
    );
    Ok((headers, Ranged::new(range, body)).into_response())
}

fn get_path_and_mime(root: PathBuf, rel_path: PathBuf) -> Result<(PathBuf, ImageFormat)> {
    let path = path_clean::clean(rel_path);
    let path = path.strip_prefix("/").unwrap_or(&path);
    let path = root.join(path);

    if !path.exists() || !path.is_file() {
        return Err((StatusCode::NOT_FOUND, "File not found".to_string()));
    }

    match find_image_mime(MimeGuess::from_path(&path)) {
        Some(mime) => Ok((path.clone(), mime)),
        None => Err((StatusCode::BAD_REQUEST, "Unsupported file type".to_string())),
    }
}

fn find_image_mime(mime: MimeGuess) -> Option<ImageFormat> {
    mime.into_iter()
        .flat_map(|m| ImageFormat::from_mime_type(&m))
        .next()
}

fn get_response_headers(image_format: &ImageFormat) -> HeaderMap {
    debug!("Setting response headers for format: {image_format:?}");
    let mut headers = HeaderMap::new();
    for (name, value) in [
        (CONTENT_TYPE, image_format.to_mime_type()),
        (CACHE_CONTROL, "public, max-age=31536000"),
        (X_CONTENT_TYPE_OPTIONS, "nosniff"),
    ] {
        debug!("Setting header: {name}: {value}");
        headers.insert(name, HeaderValue::from_static(value));
    }
    debug!("Response headers set: {headers:?}");
    headers
}

async fn load_file(path: &PathBuf) -> Result<File> {
    debug!("Loading file: {path:?}");
    File::open(&path).await.map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to read image".to_string(),
        )
    })
}

fn get_output_size(src: (u32, u32), dst: (Option<u32>, Option<u32>), dpr: u32) -> (u32, u32) {
    let (src_width, src_height) = src;
    let (dst_width, dst_height) = dst;
    let aspect_ratio = src_width as f32 / src_height as f32;

    let (mut width, mut height) = match (dst_width, dst_height) {
        (Some(w), Some(h)) => (w, h),
        (Some(w), None) => (w, (w as f32 / aspect_ratio).round() as u32),
        (None, Some(h)) => ((h as f32 * aspect_ratio).round() as u32, h),
        (None, None) => (src_width, src_height),
    };

    width *= dpr;
    height *= dpr;

    (width, height)
}

async fn load_image(mut file: File) -> Result<DynamicImage> {
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).await.unwrap();
    load_from_memory(&buffer).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to decode image: {e}"),
        )
    })
}

fn resize_image(
    config: &ResizeConfig,
    src_image: &DynamicImage,
    dst_image: &mut Image<'_>,
) -> Result<()> {
    let mut resizer = Resizer::new();

    let algorithm = if cfg!(debug_assertions) {
        ResizeAlg::Nearest
    } else {
        config.resize_algorithm()
    };
    let options = ResizeOptions::new()
        .resize_alg(algorithm)
        .fit_into_destination(Some((0.5, 0.5)));

    resizer
        .resize(src_image, dst_image, Some(&options))
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to resize image".to_string(),
            )
        })
}

fn encode_image(format: ImageFormat, image: &Image<'_>, color: ColorType) -> Result<Bytes> {
    macro_rules! match_format {
        ($format: expr , $( $target: pat => $encoder: expr, )+ ) => {
            match $format {$(
                $target => $encoder
                    .write_image(image.buffer(), image.width(), image.height(), color.into())
                    .map_err(|e| {
                        (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            format!("Failed to encode image: {e}"),
                        )
                    }),
            )+
                _ => Err((StatusCode::BAD_REQUEST, "Unsupported output format".to_string())),
            }
        };
    }

    let mut bytes = vec![];
    match_format! {
        format,
        ImageFormat::WebP => WebPEncoder::new_lossless(&mut bytes),
        ImageFormat::Png => PngEncoder::new(&mut bytes),
        ImageFormat::Jpeg => JpegEncoder::new(&mut bytes),
    }?;

    Ok(Bytes::from(bytes))
}
