//! `ImageCache::load_raster`: raster formats decode as `load` does; SVG and SVGZ are refused.
use super::*;

const SVG: &[u8] = br##"<svg xmlns="http://www.w3.org/2000/svg" width="16" height="8"><rect width="8" height="8" fill="#f00"/></svg>"##;

#[test]
fn raster_loads_decode_png_and_refuse_svg_and_svgz() {
    let cache = ImageCache::new(Arc::new(|| {}));
    let mut png = std::io::Cursor::new(Vec::new());
    image::RgbImage::from_pixel(8, 4, image::Rgb([20, 40, 60]))
        .write_to(&mut png, image::ImageFormat::Png)
        .unwrap();
    assert_eq!(
        cache.load_raster(png.into_inner()).wait(),
        Ok([8, 4]),
        "a PNG loads"
    );

    let mut bom = b"\xef\xbb\xbf \n".to_vec();
    bom.extend_from_slice(SVG);
    // Gzip's magic number is enough: no raster format starts with it.
    let svgz = vec![0x1f, 0x8b, 8, 0, 0, 0, 0, 0, 0, 0xff];
    for (what, bytes) in [
        ("svg", SVG.to_vec()),
        ("svg after a BOM and spaces", bom),
        ("svgz", svgz),
    ] {
        let error = cache
            .load_raster(bytes)
            .wait()
            .map(|_| ())
            .unwrap_err()
            .clone();
        assert!(error.0.contains("not a raster image"), "{what}: {error:?}");
    }
    #[cfg(feature = "svg")]
    {
        // The same bytes through `load` still rasterize: the refusal is its own cache entry.
        assert_eq!(cache.load(SVG).wait(), Ok([32, 16]));
        assert!(
            cache.load_raster(SVG).wait().is_err(),
            "a load of the same bytes doesn't let a raster load through"
        );
    }
    assert!(cache.load_raster(b"not an image".to_vec()).wait().is_err());
}
