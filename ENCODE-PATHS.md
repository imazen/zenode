# Encode Path Flowchart

## All the ways a user can specify encoding

### Path 1: Auto (QualityIntent — most common)
```
User: qp=high&accept.webp=true
  OR: { "preset": "auto", "quality_profile": "high", "allow": { "webp": true } }

  → QualityIntentNode
  → CodecIntent { profile: High, allowed: {jpeg,png,gif,webp} }
  → probe source → ImageFacts { has_alpha, pixel_count, is_hdr, ... }
  → select_format(intent, facts) → FormatDecision {
      format: WebP,
      quality: QualityIntent { generic: 82.0 },  // from calibration table
      lossless: false,
      hints: {},  // no per-codec overrides
      matte: None,
    }
  → EncoderConfig::default().with_generic_quality(82.0)
  → streaming encode
```

### Path 2: Format + Profile (explicit format, profile-derived quality)
```
User: format=jpeg&qp=medium
  OR: { "preset": "format", "format": "jpeg", "quality_profile": "medium" }

  → QualityIntentNode { format: "jpeg", profile: "medium" }
  → CodecIntent { format: Specific(Jpeg), profile: Medium }
  → FormatDecision {
      format: Jpeg,
      quality: QualityIntent { generic: 72.0 },
      ...
    }
  → JpegEncoderConfig::default().with_generic_quality(72.0)
  → streaming encode
```

### Path 3: Format + Profile + Per-Codec Hints (override specific params)
```
User: format=jpeg&qp=high&jpeg.progressive=true&jpeg.quality=92
  OR: { "preset": "format", "format": "jpeg", "quality_profile": "high",
        "encoder_hints": { "jpeg": { "quality": 92, "progressive": true } } }

  → QualityIntentNode + per-codec hints
  → CodecIntent { format: Specific(Jpeg), profile: High,
                   hints: { jpeg: { quality: "92", progressive: "true" } } }
  → FormatDecision {
      format: Jpeg,
      quality: QualityIntent { generic: 85.0 },  // from profile
      hints: { quality: "92", progressive: "true" },  // overrides
    }
  → JpegEncoderConfig::default()
      .with_generic_quality(85.0)  // from profile
      .with_quality(92.0)          // hint OVERRIDES generic
      .progressive(true)           // hint
  → streaming encode
```

### Path 4: Format + Generic Quality (no profile, explicit number)
```
User: format=webp&quality=75
  OR: { "preset": "format", "format": "webp", "quality_profile": "75" }

  → QualityIntentNode { format: "webp", profile: "75" }
  → CodecIntent { format: Specific(WebP), quality_fallback: 75.0 }
  → FormatDecision {
      format: WebP,
      quality: QualityIntent { generic: 75.0 },
    }
  → WebpEncoderConfig::default().with_generic_quality(75.0)
  → streaming encode
```

### Path 5: Direct Codec Config (JSON API only, full control)
```
User: { "encode": { "jpeg": { "quality": 92, "progressive": true,
                               "subsampling": "444", "trellis": true } } }

  → EncodeJpeg node (not QualityIntent)
  → node.to_encoder_config()
    → JpegEncoderConfig::ycbcr(92.0, S444).progressive(true).trellis(true)
  → streaming encode

  No format selection. No profile. No calibration tables.
  User specified everything explicitly.
```

### Path 6: Legacy (imageflow v2 compat — quality= only)
```
User: quality=85  (no qp=, no format=)

  → CodecEngine::Legacy
  → Keep source format
  → source_format.default_config().with_generic_quality(85.0)
  → streaming encode
```

### Path 7: Auto + Per-Codec Hints (format auto-selected, but with overrides for specific codecs)
```
User: qp=high&accept.webp=true&jpeg.progressive=true&webp.quality=70
  OR: { "preset": "auto", "quality_profile": "high",
        "allow": { "webp": true },
        "encoder_hints": {
          "jpeg": { "progressive": true },
          "webp": { "quality": 70 }
        } }

  → QualityIntentNode + PerCodecHints for jpeg AND webp
  → select_format picks WebP (auto)
  → FormatDecision {
      format: WebP,
      quality: QualityIntent { generic: 82.0 },
      hints: { quality: "70" },  // webp hints applied since WebP was selected
    }
  → WebpEncoderConfig::default()
      .with_generic_quality(82.0)  // from profile
      .with_quality(70.0)          // webp.quality OVERRIDES
  → streaming encode

  Note: jpeg.progressive was specified but ignored — JPEG wasn't selected.
  If JPEG had been selected, jpeg hints would apply instead.
```

## Resolution Priority (highest wins)

```
1. Per-codec hint value (jpeg.quality=92)     ← most specific
2. Generic quality from profile (qp=high → 85)
3. Generic quality fallback (quality=75)
4. Codec default
```

## Where Each Piece Lives

```
┌─────────────────────────────────────────────────────┐
│                    User Input                        │
│  RIAPI: qp=high&accept.webp=true&jpeg.quality=92    │
│  JSON:  { preset: "auto", quality_profile: "high",  │
│           allow: {webp:true}, hints: {jpeg:{q:92}} } │
│  Direct: { encode: { jpeg: { quality: 92, ... } } } │
└───────────────┬─────────────────────┬───────────────┘
                │                     │
    ┌───────────▼──────────┐   ┌──────▼──────────┐
    │  QualityIntentNode   │   │  EncodeJpeg /   │
    │  (zencodecs)         │   │  EncodePng /    │
    │                      │   │  etc.           │
    │  profile, format,    │   │  (codec crate)  │
    │  dpr, lossless,      │   │                 │
    │  allowed formats,    │   │  Explicit codec │
    │  per-codec hints     │   │  params only    │
    └───────────┬──────────┘   └──────┬──────────┘
                │                     │
    ┌───────────▼──────────┐          │
    │  zencodecs oracle    │          │
    │                      │          │
    │  select_format()     │          │
    │  calibration tables  │          │
    │  FormatDecision      │          │
    └───────────┬──────────┘          │
                │                     │
    ┌───────────▼─────────────────────▼──────────┐
    │              zenpipe encoder                │
    │                                             │
    │  1. Get EncoderConfig for selected format   │
    │  2. Apply generic quality (from decision    │
    │     or from node)                           │
    │  3. Apply per-codec hints (from decision)   │
    │     OR codec-specific params (from node)    │
    │  4. Build encoder, stream pixels            │
    └─────────────────────────────────────────────┘
```

## Key Design Points

- QualityIntent and EncodeJpeg are MUTUALLY EXCLUSIVE in a pipeline.
  If QualityIntent is present, it controls format + quality.
  If EncodeJpeg is present, it's explicit codec config.
  Both cannot appear (undefined behavior → error).

- Per-codec hints on QualityIntent are DIFFERENT from EncodeJpeg params.
  Hints are `BTreeMap<String, String>` — untyped, extensible.
  EncodeJpeg params are typed struct fields — compile-checked.
  Hints pass through zencodecs' FormatDecision.
  EncodeJpeg params go through `to_encoder_config()`.

- Generic quality (0-100) is the SAME SCALE for all codecs.
  Each codec maps it through `with_generic_quality()`.
  Profile names ("high", "medium") resolve to a generic quality number.
  Per-codec hint `quality` or codec-specific param OVERRIDES generic.

- Format auto-selection only happens with QualityIntent.
  EncodeJpeg means "I know I want JPEG." No selection needed.
