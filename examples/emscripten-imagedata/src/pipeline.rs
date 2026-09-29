//! Normalise an untrusted image into model input: sniff, bound, decode,
//! orient, downscale, re-encode as baseline JPEG with no metadata.

use std::io::Cursor;

use fast_image_resize as fir;
use image::metadata::Orientation;
use image::{DynamicImage, ImageDecoder, ImageFormat, ImageReader, Limits, RgbImage};
use serde::Serialize;

#[derive(Debug, Clone, Copy)]
pub struct Options {
    /// Longest output edge; larger inputs are downscaled to fit.
    pub max_edge: u32,
    /// Inputs with more pixels than this are rejected before decoding.
    pub max_pixels: u64,
    pub quality: u8,
    /// Where to stop; earlier stages exist for benchmarking.
    pub stage: Stage,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            max_edge: 1568,
            max_pixels: 40_000_000,
            quality: 85,
            stage: Stage::Encode,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Stage {
    Sniff,
    Decode,
    Resize,
    Encode,
}

#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
impl Stage {
    pub fn parse(s: &str) -> Option<Self> {
        Some(match s {
            "sniff" => Self::Sniff,
            "decode" => Self::Decode,
            "resize" => Self::Resize,
            "encode" => Self::Encode,
            _ => return None,
        })
    }
}

#[derive(Debug, Serialize)]
pub struct Info {
    pub format: &'static str,
    pub width: u32,
    pub height: u32,
    pub out_width: u32,
    pub out_height: u32,
    pub in_bytes: usize,
    pub out_bytes: usize,
    pub stage: Stage,
}

pub struct Processed {
    pub info: Info,
    /// Baseline JPEG, empty for stages before `Encode`.
    pub jpeg: Vec<u8>,
}

#[derive(Debug)]
pub enum Error {
    Base64,
    Format,
    TooLarge { width: u32, height: u32 },
    Decode(String),
    Resize(String),
    Encode(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Base64 => write!(f, "invalid base64"),
            Error::Format => write!(f, "unsupported or unrecognised image format"),
            Error::TooLarge { width, height } => {
                write!(
                    f,
                    "image dimensions {width}x{height} exceed the pixel budget"
                )
            }
            Error::Decode(e) => write!(f, "decode: {e}"),
            Error::Resize(e) => write!(f, "resize: {e}"),
            Error::Encode(e) => write!(f, "encode: {e}"),
        }
    }
}

impl std::error::Error for Error {}

/// Strip a `data:` URL prefix if present and decode the base64 payload.
/// The declared media type is ignored: the bytes are sniffed.
pub fn decode_base64(input: &str) -> Result<Vec<u8>, Error> {
    let payload = match input.strip_prefix("data:") {
        Some(rest) => rest.split_once(',').map(|(_, p)| p).ok_or(Error::Base64)?,
        None => input,
    };
    base64_simd::forgiving_decode_to_vec(payload.trim().as_bytes()).map_err(|_| Error::Base64)
}

pub fn encode_base64(bytes: &[u8]) -> String {
    base64_simd::STANDARD.encode_to_string(bytes)
}

fn format_name(f: ImageFormat) -> &'static str {
    match f {
        ImageFormat::Jpeg => "jpeg",
        ImageFormat::Png => "png",
        ImageFormat::WebP => "webp",
        ImageFormat::Gif => "gif",
        _ => "other",
    }
}

fn fit(width: u32, height: u32, max_edge: u32) -> (u32, u32) {
    let long = width.max(height);
    if long <= max_edge {
        return (width, height);
    }
    let scale = max_edge as f64 / long as f64;
    (
        ((width as f64 * scale).round() as u32).max(1),
        ((height as f64 * scale).round() as u32).max(1),
    )
}

/// Alpha is flattened onto white rather than dropped so transparent regions
/// do not leak arbitrary hidden colour data into the model.
fn to_rgb(img: DynamicImage) -> RgbImage {
    match img {
        DynamicImage::ImageRgb8(rgb) => rgb,
        DynamicImage::ImageRgba8(rgba) => {
            let (w, h) = rgba.dimensions();
            let src = rgba.into_raw();
            let mut out = Vec::with_capacity(w as usize * h as usize * 3);
            for px in src.chunks_exact(4) {
                let a = px[3] as u32;
                let inv = 255 - a;
                for c in &px[..3] {
                    out.push(((*c as u32 * a + 255 * inv + 127) / 255) as u8);
                }
            }
            RgbImage::from_raw(w, h, out).unwrap()
        }
        other => match other.color().has_alpha() {
            true => to_rgb(DynamicImage::ImageRgba8(other.into_rgba8())),
            false => other.into_rgb8(),
        },
    }
}

fn resize(rgb: RgbImage, out_w: u32, out_h: u32) -> Result<RgbImage, Error> {
    let (w, h) = rgb.dimensions();
    let src = fir::images::Image::from_vec_u8(w, h, rgb.into_raw(), fir::PixelType::U8x3)
        .map_err(|e| Error::Resize(e.to_string()))?;
    let mut dst = fir::images::Image::new(out_w, out_h, fir::PixelType::U8x3);
    let opts = fir::ResizeOptions::new()
        .resize_alg(fir::ResizeAlg::Convolution(fir::FilterType::Lanczos3));
    let mut resizer = fir::Resizer::new();
    // fir assumes simd128 on every wasm32 target; keep a build without the
    // target feature genuinely scalar so the comparison is meaningful.
    if cfg!(target_arch = "wasm32") && !cfg!(target_feature = "simd128") {
        unsafe { resizer.set_cpu_extensions(fir::CpuExtensions::None) };
    }
    resizer
        .resize(&src, &mut dst, &opts)
        .map_err(|e| Error::Resize(e.to_string()))?;
    Ok(RgbImage::from_raw(out_w, out_h, dst.into_vec()).unwrap())
}

fn encode(rgb: &RgbImage, quality: u8) -> Result<Vec<u8>, Error> {
    let (w, h) = rgb.dimensions();
    let (w, h) = (
        u16::try_from(w).map_err(|_| Error::Encode("width".into()))?,
        u16::try_from(h).map_err(|_| Error::Encode("height".into()))?,
    );
    let mut out = Vec::with_capacity(rgb.len() / 8);
    jpeg_encoder::Encoder::new(&mut out, quality)
        .encode(rgb.as_raw(), w, h, jpeg_encoder::ColorType::Rgb)
        .map_err(|e| Error::Encode(e.to_string()))?;
    Ok(out)
}

pub fn process(input: &[u8], opts: &Options) -> Result<Processed, Error> {
    let mut reader = ImageReader::new(Cursor::new(input))
        .with_guessed_format()
        .map_err(|_| Error::Format)?;
    let format = reader.format().ok_or(Error::Format)?;
    if !matches!(
        format,
        ImageFormat::Jpeg | ImageFormat::Png | ImageFormat::WebP | ImageFormat::Gif
    ) {
        return Err(Error::Format);
    }

    let mut limits = Limits::no_limits();
    limits.max_alloc = Some(opts.max_pixels * 4 + (16 << 20));
    reader.limits(limits);

    // Header only: reject before any pixel allocation happens.
    let mut decoder = reader
        .into_decoder()
        .map_err(|e| Error::Decode(e.to_string()))?;
    let (width, height) = decoder.dimensions();
    if width as u64 * height as u64 > opts.max_pixels || width == 0 || height == 0 {
        return Err(Error::TooLarge { width, height });
    }
    let orientation = decoder.orientation().unwrap_or(Orientation::NoTransforms);

    let mut info = Info {
        format: format_name(format),
        width,
        height,
        out_width: width,
        out_height: height,
        in_bytes: input.len(),
        out_bytes: 0,
        stage: opts.stage,
    };
    if opts.stage == Stage::Sniff {
        return Ok(Processed {
            info,
            jpeg: Vec::new(),
        });
    }

    let mut img = DynamicImage::from_decoder(decoder).map_err(|e| Error::Decode(e.to_string()))?;
    img.apply_orientation(orientation);
    let mut rgb = to_rgb(img);
    if opts.stage == Stage::Decode {
        return Ok(Processed {
            info,
            jpeg: Vec::new(),
        });
    }

    let (w, h) = rgb.dimensions();
    let (out_w, out_h) = fit(w, h, opts.max_edge);
    if (out_w, out_h) != (w, h) {
        rgb = resize(rgb, out_w, out_h)?;
    }
    info.out_width = out_w;
    info.out_height = out_h;
    if opts.stage == Stage::Resize {
        return Ok(Processed {
            info,
            jpeg: Vec::new(),
        });
    }

    let jpeg = encode(&rgb, opts.quality)?;
    info.out_bytes = jpeg.len();
    Ok(Processed { info, jpeg })
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{GenericImageView, ImageEncoder, Rgba, RgbaImage};

    fn png(w: u32, h: u32) -> Vec<u8> {
        let img = RgbaImage::from_fn(w, h, |x, y| {
            Rgba([
                (x % 256) as u8,
                (y % 256) as u8,
                128,
                if x < w / 2 { 255 } else { 0 },
            ])
        });
        let mut out = Vec::new();
        image::codecs::png::PngEncoder::new(&mut out)
            .write_image(img.as_raw(), w, h, image::ExtendedColorType::Rgba8)
            .unwrap();
        out
    }

    #[test]
    fn downscales_and_reencodes() {
        let out = process(&png(4000, 2000), &Options::default()).unwrap();
        assert_eq!(out.info.format, "png");
        assert_eq!((out.info.out_width, out.info.out_height), (1568, 784));
        let back = image::load_from_memory(&out.jpeg).unwrap();
        assert_eq!(back.dimensions(), (1568, 784));
        // Transparent half flattened onto white.
        let px = back.to_rgb8().get_pixel(1500, 400).0;
        assert!(px.iter().all(|c| *c > 240), "{px:?}");
    }

    #[test]
    fn keeps_small_images() {
        let out = process(&png(300, 200), &Options::default()).unwrap();
        assert_eq!((out.info.out_width, out.info.out_height), (300, 200));
    }

    #[test]
    fn rejects_oversized_before_decoding() {
        let opts = Options {
            max_pixels: 1_000_000,
            ..Options::default()
        };
        match process(&png(2000, 1000), &opts) {
            Err(Error::TooLarge {
                width: 2000,
                height: 1000,
            }) => {}
            other => panic!("{:?}", other.map(|p| p.info)),
        }
    }

    #[test]
    fn rejects_non_images() {
        assert!(matches!(
            process(b"hello world", &Options::default()),
            Err(Error::Format)
        ));
        assert!(matches!(
            process(b"", &Options::default()),
            Err(Error::Format)
        ));
    }

    #[test]
    fn data_url_prefix_is_ignored_for_sniffing() {
        let b64 = encode_base64(&png(10, 10));
        let bytes = decode_base64(&format!("data:image/jpeg;base64,{b64}")).unwrap();
        assert_eq!(
            process(&bytes, &Options::default()).unwrap().info.format,
            "png"
        );
        assert!(matches!(
            decode_base64("data:image/png;base64,!!!"),
            Err(Error::Base64)
        ));
    }
}
