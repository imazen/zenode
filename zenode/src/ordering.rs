//! Pipeline ordering and coalescing types.

/// Pipeline processing phase.
///
/// Nodes in earlier phases run before nodes in later phases.
/// Within the same phase, nodes can be reordered by the pipeline optimizer.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Phase {
    /// Source decoding.
    Decode = 0,
    /// Raw development (demosaic, white balance, color matrix).
    RawDevelop = 1,
    /// EXIF orientation, initial crop.
    Orient = 2,
    /// Pre-tonemap adjustments on scene-referred linear data.
    SceneLinear = 3,
    /// Scene-to-display tone mapping (sigmoid, basecurve, filmic).
    ToneMap = 4,
    /// Display-referred adjustments (exposure, contrast, color).
    DisplayAdjust = 5,
    /// Full-resolution spatial operations (denoise, sharpen, clarity).
    PreResize = 6,
    /// Geometric transforms (resize, crop, rotate).
    Resize = 7,
    /// Output-resolution spatial operations (grain, vignette, bloom).
    PostResize = 8,
    /// Palette reduction.
    Quantize = 9,
    /// Encoding / output.
    Encode = 10,
}

/// How a node can be fused/coalesced with adjacent operations.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CoalesceInfo {
    /// Named group (e.g., `"fused_adjust"`).
    ///
    /// Nodes in the same group with `fusable = true` can be merged
    /// into a single SIMD pass.
    pub group: &'static str,

    /// This node contributes parameters to the coalesced operation.
    pub fusable: bool,

    /// This node IS the coalesced target (e.g., FusedAdjust itself).
    pub is_target: bool,
}
