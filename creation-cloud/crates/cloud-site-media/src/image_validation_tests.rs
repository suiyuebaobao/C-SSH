//! 覆盖站点媒体真实格式探测、有界解码、尺寸规则和统一 PNG 重编码。

use image::{
    ExtendedColorType, ImageEncoder,
    codecs::{jpeg::JpegEncoder, png::PngEncoder},
};
use sha2::{Digest, Sha256};

use crate::image_validation::{MAX_UPLOAD_BYTES, validate_and_reencode};

fn png(width: u32, height: u32) -> Vec<u8> {
    let pixels = vec![192_u8; width as usize * height as usize * 4];
    let mut bytes = Vec::new();
    PngEncoder::new(&mut bytes)
        .write_image(&pixels, width, height, ExtendedColorType::Rgba8)
        .unwrap_or_else(|error| panic!("测试 PNG 编码失败: {error}"));
    bytes
}

fn jpeg(width: u32, height: u32) -> Vec<u8> {
    let pixels = vec![96_u8; width as usize * height as usize * 3];
    let mut bytes = Vec::new();
    JpegEncoder::new_with_quality(&mut bytes, 90)
        .write_image(&pixels, width, height, ExtendedColorType::Rgb8)
        .unwrap_or_else(|error| panic!("测试 JPEG 编码失败: {error}"));
    bytes
}

#[test]
fn accepts_png_and_jpeg_but_always_emits_verified_png() {
    for (mime, bytes) in [("image/png", png(128, 128)), ("image/jpeg", jpeg(128, 128))] {
        let image = validate_and_reencode(mime, &bytes).expect("合法正方形图片应通过校验");
        assert!(image.png.starts_with(b"\x89PNG\r\n\x1a\n"));
        assert_eq!(image.width, 128);
        assert_eq!(image.height, 128);
        assert_eq!(image.sha256, hex::encode(Sha256::digest(&image.png)));
    }
}

#[test]
fn rejects_mime_mismatch_and_non_png_jpeg_formats() {
    assert!(validate_and_reencode("image/jpeg", &png(128, 128)).is_err());
    assert!(validate_and_reencode("image/gif", b"GIF89a").is_err());
    assert!(
        validate_and_reencode(
            "image/svg+xml",
            br#"<svg xmlns="http://www.w3.org/2000/svg"><script/></svg>"#,
        )
        .is_err()
    );
}

#[test]
fn rejects_corrupt_non_square_and_out_of_range_images() {
    assert!(validate_and_reencode("image/png", b"\x89PNG\r\n\x1a\ncorrupt").is_err());
    assert!(validate_and_reencode("image/png", &png(128, 129)).is_err());
    assert!(validate_and_reencode("image/png", &png(127, 127)).is_err());
}

#[test]
fn rejects_empty_and_oversized_uploads_before_decoding() {
    assert!(validate_and_reencode("image/png", &[]).is_err());
    assert!(validate_and_reencode("image/png", &vec![0_u8; MAX_UPLOAD_BYTES + 1]).is_err());
}
