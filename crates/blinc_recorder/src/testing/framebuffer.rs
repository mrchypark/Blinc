//! Framebuffer capture for testing and screenshots.
//!
//! Provides functionality to:
//! - Capture rendered frames to CPU memory
//! - Export screenshots as PNG
//! - Capture frame sequences for animation testing

use std::path::Path;

/// Raw captured framebuffer data.
#[derive(Clone, Debug)]
pub struct CapturedFrame {
    /// Raw pixel data (RGBA8)
    pub data: Vec<u8>,
    /// Width in pixels
    pub width: u32,
    /// Height in pixels
    pub height: u32,
    /// Frame number (if capturing a sequence)
    pub frame_number: u64,
}

impl CapturedFrame {
    /// Create a new captured frame.
    pub fn new(data: Vec<u8>, width: u32, height: u32) -> Self {
        Self {
            data,
            width,
            height,
            frame_number: 0,
        }
    }

    /// Create with frame number.
    pub fn with_frame_number(mut self, frame: u64) -> Self {
        self.frame_number = frame;
        self
    }

    /// Get the number of pixels.
    pub fn pixel_count(&self) -> usize {
        (self.width * self.height) as usize
    }

    /// Get expected data length for RGBA8.
    pub fn expected_size(&self) -> usize {
        self.pixel_count() * 4
    }

    /// Get a pixel at (x, y) as RGBA.
    pub fn get_pixel(&self, x: u32, y: u32) -> Option<[u8; 4]> {
        if x >= self.width || y >= self.height {
            return None;
        }
        let idx = ((y * self.width + x) * 4) as usize;
        if idx + 4 > self.data.len() {
            return None;
        }
        Some([
            self.data[idx],
            self.data[idx + 1],
            self.data[idx + 2],
            self.data[idx + 3],
        ])
    }

    /// Compare with another frame, returning the number of different pixels.
    pub fn diff_pixel_count(&self, other: &CapturedFrame) -> usize {
        if self.width != other.width || self.height != other.height {
            return self.pixel_count().max(other.pixel_count());
        }

        self.data
            .chunks(4)
            .zip(other.data.chunks(4))
            .filter(|(a, b)| a != b)
            .count()
    }

    /// Check if two frames are identical.
    pub fn is_identical_to(&self, other: &CapturedFrame) -> bool {
        self.width == other.width && self.height == other.height && self.data == other.data
    }

    /// Calculate the percentage of pixels that differ.
    pub fn diff_percentage(&self, other: &CapturedFrame) -> f32 {
        let total = self.pixel_count().max(1) as f32;
        let diff = self.diff_pixel_count(other) as f32;
        (diff / total) * 100.0
    }
}

/// Screenshot exporter for various formats.
pub struct ScreenshotExporter;

impl ScreenshotExporter {
    /// Export a captured frame as PNG.
    ///
    /// Note: Requires the `image` feature (via blinc_image crate).
    #[cfg(feature = "png")]
    pub fn save_png(frame: &CapturedFrame, path: impl AsRef<Path>) -> std::io::Result<()> {
        use std::fs::File;
        use std::io::BufWriter;

        let file = File::create(path)?;
        let writer = BufWriter::new(file);

        let mut encoder = png::Encoder::new(writer, frame.width, frame.height);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);

        let mut writer = encoder
            .write_header()
            .map_err(std::io::Error::other)?;

        writer
            .write_image_data(&frame.data)
            .map_err(std::io::Error::other)?;

        Ok(())
    }

    /// Export a captured frame as PNG without external dependencies.
    ///
    /// This is a minimal PNG encoder for basic screenshot needs.
    pub fn save_png_minimal(frame: &CapturedFrame, path: impl AsRef<Path>) -> std::io::Result<()> {
        use std::fs::File;
        use std::io::Write;

        let mut file = File::create(path)?;

        // PNG signature
        file.write_all(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A])?;

        // IHDR chunk
        let mut ihdr_data = Vec::with_capacity(13);
        ihdr_data.extend_from_slice(&frame.width.to_be_bytes());
        ihdr_data.extend_from_slice(&frame.height.to_be_bytes());
        ihdr_data.push(8); // Bit depth
        ihdr_data.push(6); // Color type (RGBA)
        ihdr_data.push(0); // Compression method
        ihdr_data.push(0); // Filter method
        ihdr_data.push(0); // Interlace method
        write_png_chunk(&mut file, b"IHDR", &ihdr_data)?;

        // IDAT chunk (image data) - uncompressed for simplicity
        // For a proper implementation, we'd use zlib compression
        let mut idat_data = Vec::new();

        // Build uncompressed deflate blocks
        let scanline_len = frame.width as usize * 4 + 1; // +1 for filter byte

        // Zlib header
        idat_data.push(0x78); // CM=8, CINFO=7
        idat_data.push(0x01); // FCHECK=1, no dict, fastest

        // Write scanlines as uncompressed deflate blocks
        for y in 0..frame.height {
            let is_last = y == frame.height - 1;

            // Build scanline with filter byte
            let mut scanline = Vec::with_capacity(scanline_len);
            scanline.push(0); // Filter: None
            let row_start = (y * frame.width * 4) as usize;
            let row_end = row_start + (frame.width * 4) as usize;
            scanline.extend_from_slice(&frame.data[row_start..row_end]);

            // Write as uncompressed deflate block
            let bfinal = if is_last { 1u8 } else { 0u8 };
            idat_data.push(bfinal); // BFINAL=0/1, BTYPE=00 (no compression)
            let len = scanline.len() as u16;
            idat_data.extend_from_slice(&len.to_le_bytes());
            idat_data.extend_from_slice(&(!len).to_le_bytes());
            idat_data.extend_from_slice(&scanline);
        }

        // Adler-32 checksum
        let adler = adler32(&frame.data, frame.width, frame.height);
        idat_data.extend_from_slice(&adler.to_be_bytes());

        write_png_chunk(&mut file, b"IDAT", &idat_data)?;

        // IEND chunk
        write_png_chunk(&mut file, b"IEND", &[])?;

        Ok(())
    }
}

/// Write a PNG chunk.
fn write_png_chunk(
    file: &mut std::fs::File,
    chunk_type: &[u8; 4],
    data: &[u8],
) -> std::io::Result<()> {
    use std::io::Write;

    let length = data.len() as u32;
    file.write_all(&length.to_be_bytes())?;
    file.write_all(chunk_type)?;
    file.write_all(data)?;

    // CRC32 of type + data
    let mut crc_data = Vec::with_capacity(4 + data.len());
    crc_data.extend_from_slice(chunk_type);
    crc_data.extend_from_slice(data);
    let crc = crc32(&crc_data);
    file.write_all(&crc.to_be_bytes())?;

    Ok(())
}

/// Simple CRC32 implementation for PNG.
fn crc32(data: &[u8]) -> u32 {
    const CRC_TABLE: [u32; 256] = {
        let mut table = [0u32; 256];
        let mut i = 0;
        while i < 256 {
            let mut c = i as u32;
            let mut k = 0;
            while k < 8 {
                if c & 1 != 0 {
                    c = 0xEDB88320 ^ (c >> 1);
                } else {
                    c >>= 1;
                }
                k += 1;
            }
            table[i] = c;
            i += 1;
        }
        table
    };

    let mut crc = 0xFFFFFFFF_u32;
    for &byte in data {
        crc = CRC_TABLE[((crc ^ byte as u32) & 0xFF) as usize] ^ (crc >> 8);
    }
    crc ^ 0xFFFFFFFF
}

/// Simple Adler-32 checksum for zlib.
fn adler32(data: &[u8], width: u32, height: u32) -> u32 {
    let mut a: u32 = 1;
    let mut b: u32 = 0;
    const MOD: u32 = 65521;

    // Process each scanline with filter byte prefix
    for y in 0..height {
        // Filter byte (always 0 for "None")
        a %= MOD;
        b = (b + a) % MOD;

        // Row data
        let row_start = (y * width * 4) as usize;
        let row_end = row_start + (width * 4) as usize;
        for &byte in &data[row_start..row_end] {
            a = (a + byte as u32) % MOD;
            b = (b + a) % MOD;
        }
    }

    (b << 16) | a
}

/// Frame sequence for capturing multiple frames.
pub struct FrameSequence {
    frames: Vec<CapturedFrame>,
    max_frames: usize,
}

impl FrameSequence {
    /// Create a new frame sequence with maximum capacity.
    pub fn new(max_frames: usize) -> Self {
        Self {
            frames: Vec::with_capacity(max_frames.min(1000)),
            max_frames,
        }
    }

    /// Add a frame to the sequence.
    pub fn push(&mut self, frame: CapturedFrame) {
        if self.frames.len() < self.max_frames {
            self.frames.push(frame);
        }
    }

    /// Get the number of captured frames.
    pub fn len(&self) -> usize {
        self.frames.len()
    }

    /// Check if the sequence is empty.
    pub fn is_empty(&self) -> bool {
        self.frames.is_empty()
    }

    /// Get a frame by index.
    pub fn get(&self, index: usize) -> Option<&CapturedFrame> {
        self.frames.get(index)
    }

    /// Iterate over all frames.
    pub fn iter(&self) -> impl Iterator<Item = &CapturedFrame> {
        self.frames.iter()
    }

    /// Export all frames as numbered PNGs.
    pub fn export_frames(&self, base_path: impl AsRef<Path>) -> std::io::Result<()> {
        let base = base_path.as_ref();
        let parent = base.parent().unwrap_or(Path::new("."));
        let stem = base.file_stem().and_then(|s| s.to_str()).unwrap_or("frame");

        std::fs::create_dir_all(parent)?;

        for (i, frame) in self.frames.iter().enumerate() {
            let path = parent.join(format!("{stem}_{i:04}.png"));
            ScreenshotExporter::save_png_minimal(frame, path)?;
        }

        Ok(())
    }

    /// Clear all frames.
    pub fn clear(&mut self) {
        self.frames.clear();
    }
}

/// Visual regression test result.
#[derive(Clone, Debug)]
pub struct RegressionResult {
    /// Whether the test passed.
    pub passed: bool,
    /// Number of pixels that differ.
    pub diff_pixels: usize,
    /// Percentage of pixels that differ.
    pub diff_percentage: f32,
    /// Tolerance threshold used.
    pub tolerance: f32,
}

impl RegressionResult {
    /// Check if within tolerance.
    pub fn is_within_tolerance(&self, tolerance: f32) -> bool {
        self.diff_percentage <= tolerance
    }
}

/// Compare two frames for visual regression testing.
pub fn compare_frames(
    actual: &CapturedFrame,
    expected: &CapturedFrame,
    tolerance_percent: f32,
) -> RegressionResult {
    let diff_pixels = actual.diff_pixel_count(expected);
    let diff_percentage = actual.diff_percentage(expected);
    let passed = diff_percentage <= tolerance_percent;

    RegressionResult {
        passed,
        diff_pixels,
        diff_percentage,
        tolerance: tolerance_percent,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_frame(width: u32, height: u32, color: [u8; 4]) -> CapturedFrame {
        let pixel_count = (width * height) as usize;
        let mut data = Vec::with_capacity(pixel_count * 4);
        for _ in 0..pixel_count {
            data.extend_from_slice(&color);
        }
        CapturedFrame::new(data, width, height)
    }

    #[test]
    fn test_frame_creation() {
        let frame = create_test_frame(100, 100, [255, 0, 0, 255]);
        assert_eq!(frame.width, 100);
        assert_eq!(frame.height, 100);
        assert_eq!(frame.pixel_count(), 10000);
        assert_eq!(frame.data.len(), 40000);
    }

    #[test]
    fn test_get_pixel() {
        let frame = create_test_frame(10, 10, [255, 128, 64, 255]);
        let pixel = frame.get_pixel(5, 5).unwrap();
        assert_eq!(pixel, [255, 128, 64, 255]);

        // Out of bounds
        assert!(frame.get_pixel(100, 100).is_none());
    }

    #[test]
    fn test_frame_comparison_identical() {
        let frame1 = create_test_frame(100, 100, [255, 0, 0, 255]);
        let frame2 = create_test_frame(100, 100, [255, 0, 0, 255]);

        assert!(frame1.is_identical_to(&frame2));
        assert_eq!(frame1.diff_pixel_count(&frame2), 0);
        assert_eq!(frame1.diff_percentage(&frame2), 0.0);
    }

    #[test]
    fn test_frame_comparison_different() {
        let frame1 = create_test_frame(100, 100, [255, 0, 0, 255]);
        let frame2 = create_test_frame(100, 100, [0, 255, 0, 255]);

        assert!(!frame1.is_identical_to(&frame2));
        assert_eq!(frame1.diff_pixel_count(&frame2), 10000);
        assert_eq!(frame1.diff_percentage(&frame2), 100.0);
    }

    #[test]
    fn test_regression_comparison() {
        let actual = create_test_frame(100, 100, [255, 0, 0, 255]);
        let expected = create_test_frame(100, 100, [255, 0, 0, 255]);

        let result = compare_frames(&actual, &expected, 1.0);
        assert!(result.passed);
        assert_eq!(result.diff_pixels, 0);
    }

    #[test]
    fn test_frame_sequence() {
        let mut seq = FrameSequence::new(10);
        assert!(seq.is_empty());

        seq.push(create_test_frame(10, 10, [255, 0, 0, 255]));
        seq.push(create_test_frame(10, 10, [0, 255, 0, 255]));

        assert_eq!(seq.len(), 2);
        assert!(!seq.is_empty());
    }

    #[test]
    fn test_crc32() {
        // Test vector: "IEND" chunk type should produce known CRC
        let data = b"IEND";
        let crc = crc32(data);
        assert_eq!(crc, 0xAE426082);
    }
}
