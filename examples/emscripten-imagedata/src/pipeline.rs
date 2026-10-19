//! Normalise an untrusted image into model input: sniff, bound, decode,
//! orient, downscale, re-encode as baseline JPEG with no metadata.

use std::io::Cursor;

use fast_image_resize as fir;
use image::metadata::Orientation;
use image::{DynamicImage, ImageDecoder, ImageFormat, ImageReader, Limits, RgbImage};
use serde::Serialize;
use zune_core::bytestream::ZCursor;
use zune_core::colorspace::ColorSpace;
use zune_core::options::DecoderOptions;

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
    // fir selects simd128 on every wasm32 build; keep a build without the
    // target feature genuinely scalar so the benchmark comparison holds.
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

/// Header-only dimensions, then the decoded RGB image with its EXIF
/// orientation. Each decoder is asked for dimensions before any pixel
/// allocation so the pixel budget is enforced up front.
struct Decoded {
    rgb: RgbImage,
    orientation: Orientation,
}

fn check_budget(width: u32, height: u32, opts: &Options) -> Result<(), Error> {
    if width == 0 || height == 0 || width as u64 * height as u64 > opts.max_pixels {
        return Err(Error::TooLarge { width, height });
    }
    Ok(())
}

fn decode_jpeg(input: &[u8], opts: &Options) -> Result<Decoded, Error> {
    let options = DecoderOptions::new_fast()
        .jpeg_set_out_colorspace(ColorSpace::RGB)
        .set_max_width(u16::MAX as usize)
        .set_max_height(u16::MAX as usize);
    let mut decoder = zune_jpeg::JpegDecoder::new_with_options(ZCursor::new(input), options);
    decoder
        .decode_headers()
        .map_err(|e| Error::Decode(e.to_string()))?;
    let info = decoder
        .info()
        .ok_or_else(|| Error::Decode("no header".into()))?;
    let (width, height) = (u32::from(info.width), u32::from(info.height));
    check_budget(width, height, opts)?;
    let orientation = decoder
        .exif()
        .and_then(|exif| Orientation::from_exif_chunk(exif))
        .unwrap_or(Orientation::NoTransforms);

    let pixels = decoder.decode().map_err(|e| Error::Decode(e.to_string()))?;
    let rgb = match decoder.output_colorspace() {
        Some(ColorSpace::RGB) => RgbImage::from_raw(width, height, pixels),
        Some(ColorSpace::Luma) => RgbImage::from_raw(
            width,
            height,
            pixels.iter().flat_map(|&l| [l, l, l]).collect(),
        ),
        Some(ColorSpace::RGBA) => Some(to_rgb(DynamicImage::ImageRgba8(
            image::RgbaImage::from_raw(width, height, pixels)
                .ok_or_else(|| Error::Decode("short buffer".into()))?,
        ))),
        other => return Err(Error::Decode(format!("colorspace {other:?}"))),
    }
    .ok_or_else(|| Error::Decode("short buffer".into()))?;
    Ok(Decoded { rgb, orientation })
}

fn decode_webp(input: &[u8], opts: &Options) -> Result<Decoded, Error> {
    use libwebp_sys as webp;
    let mut features = webp::WebPBitstreamFeatures {
        width: 0,
        height: 0,
        has_alpha: 0,
        has_animation: 0,
        format: 0,
        pad: [0; 5],
    };
    // SAFETY: `input` is a valid byte slice for the call duration, `features` is
    // a valid out-pointer sized for the pinned ABI version.
    let status = unsafe { webp::WebPGetFeatures(input.as_ptr(), input.len(), &mut features) };
    if status != webp::VP8StatusCode::VP8_STATUS_OK {
        return Err(Error::Decode(format!("webp header {status:?}")));
    }
    let (width, height) = (features.width as u32, features.height as u32);
    check_budget(width, height, opts)?;

    // The simple API decodes only the first frame of an animation, which is
    // the intended sanitisation behaviour.
    let (mut w, mut h) = (0, 0);
    let channels = if features.has_alpha != 0 { 4 } else { 3 };
    // SAFETY: as above; the returned buffer is `w * h * channels` bytes owned
    // by libwebp until WebPFree.
    let rgb = unsafe {
        let ptr = if channels == 4 {
            webp::WebPDecodeRGBA(input.as_ptr(), input.len(), &mut w, &mut h)
        } else {
            webp::WebPDecodeRGB(input.as_ptr(), input.len(), &mut w, &mut h)
        };
        if ptr.is_null() {
            return Err(Error::Decode("webp bitstream".into()));
        }
        let len = w as usize * h as usize * channels;
        let pixels = std::slice::from_raw_parts(ptr, len).to_vec();
        webp::WebPFree(ptr.cast());
        if channels == 4 {
            to_rgb(DynamicImage::ImageRgba8(
                image::RgbaImage::from_raw(w as u32, h as u32, pixels).unwrap(),
            ))
        } else {
            RgbImage::from_raw(w as u32, h as u32, pixels).unwrap()
        }
    };
    Ok(Decoded {
        rgb,
        orientation: Orientation::NoTransforms,
    })
}

/// PNG and GIF via `image`; animated GIFs yield their first frame.
fn decode_image(input: &[u8], format: ImageFormat, opts: &Options) -> Result<Decoded, Error> {
    let mut reader = ImageReader::with_format(Cursor::new(input), format);
    let mut limits = Limits::no_limits();
    limits.max_alloc = Some(opts.max_pixels * 4 + (16 << 20));
    reader.limits(limits);
    let mut decoder = reader
        .into_decoder()
        .map_err(|e| Error::Decode(e.to_string()))?;
    let (width, height) = decoder.dimensions();
    check_budget(width, height, opts)?;
    let orientation = decoder.orientation().unwrap_or(Orientation::NoTransforms);
    let img = DynamicImage::from_decoder(decoder).map_err(|e| Error::Decode(e.to_string()))?;
    Ok(Decoded {
        rgb: to_rgb(img),
        orientation,
    })
}

pub fn process(input: &[u8], opts: &Options) -> Result<Processed, Error> {
    let format = image::guess_format(input).map_err(|_| Error::Format)?;
    if !matches!(
        format,
        ImageFormat::Jpeg | ImageFormat::Png | ImageFormat::WebP | ImageFormat::Gif
    ) {
        return Err(Error::Format);
    }

    let mut info = Info {
        format: format_name(format),
        width: 0,
        height: 0,
        out_width: 0,
        out_height: 0,
        in_bytes: input.len(),
        out_bytes: 0,
        stage: opts.stage,
    };

    if opts.stage == Stage::Sniff {
        // Header parse only, through the same budget check as the real decode.
        let probe = Options {
            max_pixels: 0,
            ..*opts
        };
        match match format {
            ImageFormat::Jpeg => decode_jpeg(input, &probe),
            ImageFormat::WebP => decode_webp(input, &probe),
            other => decode_image(input, other, &probe),
        } {
            Err(Error::TooLarge { width, height }) => {
                check_budget(width, height, opts)?;
                info.width = width;
                info.height = height;
                return Ok(Processed {
                    info,
                    jpeg: Vec::new(),
                });
            }
            Err(e) => return Err(e),
            Ok(_) => unreachable!(),
        }
    }

    let Decoded { rgb, orientation } = match format {
        ImageFormat::Jpeg => decode_jpeg(input, opts)?,
        ImageFormat::WebP => decode_webp(input, opts)?,
        other => decode_image(input, other, opts)?,
    };
    info.width = rgb.width();
    info.height = rgb.height();
    let mut rgb = if orientation == Orientation::NoTransforms {
        rgb
    } else {
        let mut img = DynamicImage::ImageRgb8(rgb);
        img.apply_orientation(orientation);
        img.into_rgb8()
    };
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
    use image::{ImageEncoder, Rgba, RgbaImage};

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

    fn decode(jpeg: &[u8]) -> RgbImage {
        let out = process(
            jpeg,
            &Options {
                stage: Stage::Decode,
                max_edge: u32::MAX,
                ..Options::default()
            },
        )
        .unwrap();
        assert_eq!(out.info.format, "jpeg");
        // Decode-only leaves no JPEG, so re-run the real path at full size.
        let full = process(
            jpeg,
            &Options {
                max_edge: u32::MAX,
                quality: 100,
                ..Options::default()
            },
        )
        .unwrap();
        assert_eq!(
            (full.info.out_width, full.info.out_height),
            (out.info.width, out.info.height)
        );
        decode_jpeg(jpeg, &Options::default()).unwrap().rgb
    }

    #[test]
    fn downscales_and_reencodes() {
        let out = process(
            &png(4000, 2000),
            &Options {
                quality: 100,
                ..Options::default()
            },
        )
        .unwrap();
        assert_eq!(out.info.format, "png");
        assert_eq!((out.info.out_width, out.info.out_height), (1568, 784));
        let back = decode(&out.jpeg);
        assert_eq!(back.dimensions(), (1568, 784));
        // Transparent half flattened onto white.
        let px = back.get_pixel(1500, 400).0;
        assert!(px.iter().all(|c| *c > 240), "{px:?}");
        // Opaque half keeps its constant blue channel.
        let px = back.get_pixel(100, 100).0;
        assert!((px[2] as i32 - 128).abs() < 4, "{px:?}");
    }

    #[test]
    fn jpeg_in_jpeg_out() {
        let first = process(&png(2000, 1000), &Options::default()).unwrap();
        let again = process(
            &first.jpeg,
            &Options {
                max_edge: 800,
                ..Options::default()
            },
        )
        .unwrap();
        assert_eq!(again.info.format, "jpeg");
        assert_eq!((again.info.width, again.info.height), (1568, 784));
        assert_eq!((again.info.out_width, again.info.out_height), (800, 400));
        assert_eq!(decode(&again.jpeg).dimensions(), (800, 400));
    }

    #[test]
    fn webp_in() {
        let rgba = RgbaImage::from_fn(300, 200, |x, _| {
            Rgba([x as u8, 90, 200, if x < 150 { 255 } else { 0 }])
        });
        let mut out: *mut u8 = std::ptr::null_mut();
        let len = unsafe {
            libwebp_sys::WebPEncodeLosslessRGBA(rgba.as_raw().as_ptr(), 300, 200, 1200, &mut out)
        };
        let webp = unsafe { std::slice::from_raw_parts(out, len).to_vec() };
        unsafe { libwebp_sys::WebPFree(out.cast()) };

        let res = process(&webp, &Options::default()).unwrap();
        assert_eq!(res.info.format, "webp");
        assert_eq!((res.info.width, res.info.height), (300, 200));
        let back = decode(&res.jpeg);
        assert!(back.get_pixel(250, 100).0.iter().all(|c| *c > 240));
        assert!(back.get_pixel(50, 100).0[2] > 180);
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
