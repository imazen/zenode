//! Built-in shared node definitions.
//!
//! These nodes don't belong to any specific codec or processing crate.
//! They represent format-independent concepts used across the pipeline.

extern crate alloc;
use alloc::string::String;

use crate::*;

// ─── Decode ───

/// Decode an image from an I/O source.
///
/// The I/O binding (bytes, file, buffer) is handled by the pipeline
/// runtime, not by this node. The `io_id` identifies which I/O slot
/// to read from.
#[derive(Node, Clone, Debug, Default)]
#[node(id = "zenode.decode", group = Decode, phase = Decode)]
#[node(tags("io", "decode"))]
pub struct Decode {
    /// I/O slot identifier (assigned by the job builder).
    #[param(range(0..=255), default = 0, step = 1)]
    #[param(section = "Main")]
    pub io_id: i32,
}

// ─── QualityIntent ───

/// Format auto-selection and quality profile.
///
/// Controls which output format is chosen (when `format=auto`) and
/// at what quality level. Maps to imageflow's `qp` (quality profile)
/// system and the `accept.*` format negotiation flags.
///
/// When this node is present, the pipeline uses auto-selection.
/// When absent, an explicit encode node (e.g., `EncodeJpeg`) must
/// be provided.
#[derive(Node, Clone, Debug)]
#[node(id = "zenode.quality_intent", group = Encode, phase = Encode)]
#[node(tags("quality", "auto", "format", "encode"))]
pub struct QualityIntent {
    /// Quality profile: named preset or numeric 0-100.
    ///
    /// Named: "lowest", "low", "medium_low", "medium", "good", "high", "highest", "lossless".
    /// Numeric: "0" to "100" (maps to codec-specific quality).
    #[param(default = "high")]
    #[param(section = "Main", label = "Quality Profile")]
    #[kv("qp")]
    pub profile: String,

    /// Device pixel ratio for quality adjustment.
    ///
    /// Higher DPR screens can tolerate lower quality since pixels are
    /// smaller. A DPR of 2.0 at "high" quality is roughly equivalent
    /// to "medium" at DPR 1.0.
    #[param(range(0.5..=10.0), default = 1.0, identity = 1.0, step = 0.5)]
    #[param(unit = "×", section = "Main", label = "Device Pixel Ratio")]
    #[kv("qp.dpr", "qp.dppx", "dpr", "dppx")]
    pub dpr: f32,

    /// Global lossless preference.
    ///
    /// When true, selects lossless encoding if the chosen format supports it.
    #[param(default = false)]
    #[param(section = "Main")]
    #[kv("lossless")]
    pub lossless: bool,

    /// Allow WebP as an auto-selected output format.
    #[param(default = false)]
    #[param(section = "Allowed Formats")]
    #[kv("accept.webp")]
    pub allow_webp: bool,

    /// Allow AVIF as an auto-selected output format.
    #[param(default = false)]
    #[param(section = "Allowed Formats")]
    #[kv("accept.avif")]
    pub allow_avif: bool,

    /// Allow JPEG XL as an auto-selected output format.
    #[param(default = false)]
    #[param(section = "Allowed Formats")]
    #[kv("accept.jxl")]
    pub allow_jxl: bool,

    /// Allow non-sRGB color profiles in the output.
    #[param(default = false)]
    #[param(section = "Allowed Formats")]
    #[kv("accept.color_profiles")]
    pub allow_color_profiles: bool,
}

impl Default for QualityIntent {
    fn default() -> Self {
        Self {
            profile: String::from("high"),
            dpr: 1.0,
            lossless: false,
            allow_webp: false,
            allow_avif: false,
            allow_jxl: false,
            allow_color_profiles: false,
        }
    }
}
