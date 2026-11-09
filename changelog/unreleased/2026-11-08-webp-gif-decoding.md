- **WebP and GIF images decode on native.** `ImageCache::load`, `load_mime` and `file` take WebP
  (lossy or lossless) and GIF, showing an animated one's first frame. Adds the `image` crate's
  `webp` and `gif` decoders (pure Rust).
