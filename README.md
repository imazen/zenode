# zennode [![CI](https://img.shields.io/github/actions/workflow/status/imazen/zennode/ci.yml?style=flat-square&label=CI)](https://github.com/imazen/zennode/actions/workflows/ci.yml) [![crates.io](https://img.shields.io/crates/v/zennode?style=flat-square)](https://crates.io/crates/zennode) [![lib.rs](https://img.shields.io/crates/v/zennode?style=flat-square&label=lib.rs&color=blue)](https://lib.rs/crates/zennode) [![docs.rs](https://img.shields.io/docsrs/zennode?style=flat-square)](https://docs.rs/zennode) [![MSRV](https://img.shields.io/badge/MSRV-1.85-blue?style=flat-square)](https://doc.rust-lang.org/cargo/reference/manifest.html#the-rust-version-field) [![license](https://img.shields.io/crates/l/zennode?style=flat-square)](#license)

zennode turns a plain Rust struct into a self-documenting pipeline node. Put `#[derive(Node)]` on a struct and you describe an operation's parameters *once* — ranges, defaults, units, slider mappings, UI sections, querystring keys, JSON field names, graph input ports — then get a zero-cost `&'static` schema, RIAPI-style querystring parsing, JSON (de)serialization, JSON Schema generation, and Markdown docs for free, all built for permanent backwards compatibility. zennode defines no nodes itself; nodes live in the crates that implement them. Pure Rust, `#![forbid(unsafe_code)]`, `no_std + alloc` (std optional).

## Quick start

```toml
[dependencies]
zennode = "0.1.1"
```

Derive `Node` on a struct and annotate its parameters:

```rust
use zennode::*;

#[derive(Node, Clone, Debug, Default)]
#[node(id = "filter.brightness", group = Tone, role = Filter)]
pub struct Brightness {
    /// Amount of brightness adjustment.
    #[param(range(-1.0..=1.0), default = 0.0, identity = 0.0, step = 0.05)]
    #[param(unit = "", section = "Main")]
    pub amount: f32,
}
```

`#[derive(Node)]` generates a `NodeDef` (the factory/schema holder), a `NodeInstance` (a live instance with parameter values), and a `&'static` singleton named `<STRUCT_NAME>_NODE` in screaming-snake-case — `Brightness` produces `pub static BRIGHTNESS_NODE: BrightnessNodeDef`. Doc comments on fields become parameter descriptions, and the full `NodeSchema` is reachable at zero cost through `&'static` references.

Each field's Rust type selects its `ParamKind`; `#[param(...)]` supplies the bounds and metadata:

| Field type | `ParamKind` | Typical `#[param(...)]` keys |
|---|---|---|
| `f32` | `Float` | `range(min..=max)`, `default`, `identity`, `step` |
| `i32` | `Int` | `range(min..=max)`, `default` |
| `u32` | `U32` | `range(min..=max)`, `default` |
| `bool` | `Bool` | `default` |
| `String` | `Str` | `default` |
| `[f32; N]` | `FloatArray` (or `Color` with `#[param(color)]`) | `default`, `labels(...)` |
| a struct/enum deriving `#[derive(Node)]` | `Object` / `TaggedUnion` | (auto-detected from the field type) |
| any type with `#[param(json_schema = "...")]` | `Json` | `json_default` |

Metadata keys allowed on any `#[param(...)]`: `unit`, `section`, `label`, `slider`, `since`, `visible_when`, `json_name`, `json_alias` — plus `#[kv("w", "width")]` for the RIAPI querystring keys that map to a field. Struct-level `#[node(...)]` takes `id` (required), `group`, `role` (a.k.a. `phase`), and optional `label`, `version` / `compat_version`, `coalesce` / `fusable` / `coalesce_target`, `neighborhood`, `changes_dimensions`, `format(preferred = …, alpha = …)`, `tags(...)`, `json_key`, `deny_unknown_fields`, and `inputs(...)` (see [Graph topology](#graph-topology)).

Parameter enums get their own derive. Variant labels are set with `#[variant(label = "…")]`, descriptions come from doc comments, and `#[variant(alias = "…")]` adds extra `FromStr` spellings:

```rust
#[derive(NodeEnum, Clone, Debug, Default)]
pub enum FitMode {
    /// Fit entirely within the bounds, preserving aspect ratio.
    #[default]
    #[variant(label = "Max")]
    Max,
    /// Fill the bounds, cropping any excess.
    #[variant(label = "Crop", alias = "cover")]
    Crop,
}
```

`#[derive(NodeEnum)]` generates `Display`, `FromStr`, and a `&'static [EnumVariant]` table (`FitMode::zennode_variants()`) for the snake_case variant names. To use an enum as a *structured* node parameter, derive `Node` on it instead — that yields a `ParamKind::TaggedUnion`.

## Using the registry

`NodeRegistry` aggregates node definitions from across the ecosystem, then parses querystrings, instantiates nodes, and emits schemas and docs:

```rust
use zennode::{NodeRegistry, ParamMap, ParamValue, NodeGroup};

let mut registry = NodeRegistry::new();
registry.register(&BRIGHTNESS_NODE);          // or register_all(&[&A_NODE, &B_NODE])

// Parse a RIAPI-style querystring into node instances + non-fatal warnings.
let parsed = registry.from_querystring("brightness.amount=0.2");
for warning in &parsed.warnings {
    eprintln!("querystring: {warning:?}");    // unknown / out-of-range keys degrade gracefully
}

// Or build an instance from explicit params (ParamMap = BTreeMap<String, ParamValue>).
let mut params = ParamMap::new();
params.insert("amount".into(), ParamValue::F32(0.2));
let node = registry.create("filter.brightness", &params).unwrap();

// Discover nodes by group, or render Markdown docs for the whole registry.
let tone_nodes = registry.by_group(NodeGroup::Tone);
let docs = registry.to_markdown();
```

Querystring parsing is lenient: it reports problems via `KvWarning` instead of failing hard, so an unknown or out-of-range key degrades gracefully rather than rejecting the whole request — the right default for a public image-URL API.

## What it provides

**Schema introspection.** Every node carries a `NodeSchema` with 14 fields (id, label, description, group, role, params, tags, coalesce info, format hints, version, compat_version, json_key, deny_unknown_fields, inputs). Every parameter carries a `ParamDesc` with 13 fields (name, label, description, kind, unit, section, slider mapping, kv_keys, since_version, visible_when, optional, json_name, json_aliases). Parameter types span 11 `ParamKind` variants (Float, Int, U32, Bool, Str, Enum, FloatArray, Color, Json, Object, TaggedUnion), and nodes file under 18 `NodeGroup` categories (Decode through Other).

**Node registry.** `NodeRegistry` looks up nodes by id (`get`), group (`by_group`), or tag (`by_tag`); creates instances from a `ParamMap` (`create`); parses RIAPI querystrings against every registered node (`from_querystring`, with consumption tracking and warnings for unrecognized keys); and renders Markdown docs for one node or the whole registry (`to_markdown`).

**RIAPI querystring parsing.** `KvPairs` is a consumption-tracking parser — several node definitions each claim their keys from one querystring, and unconsumed keys surface as warnings. Typed accessors (`take_f32`, `take_i32`, `take_u32`, `take_bool`, `take`) handle parsing and reporting, and `snapshot()` traces which consumer took each key.

**Operation coalescing.** `NodeRole` (9 roles: Decode, Geometry, Orient, Resize, Filter, Composite, Analysis, Quantize, Encode; aliased as `Phase`) tells the pipeline bridge which planner a node feeds into. `CoalesceInfo` marks adjacent compatible nodes for fusion into a single operation — always equivalent to sequential execution, never reordering.

**JSON round-trip.** With the `serde` feature, the registry serializes and deserializes individual nodes and whole pipelines. Nodes serialize as `{"json_key": {...params...}}`, pipelines as arrays. Unknown-field rejection is opt-in per node via `#[node(deny_unknown_fields)]`; field renames and back-compat aliases come from `#[param(json_name = …)]` / `#[param(json_alias = …)]`.

**JSON Schema generation.** With the `json-schema` feature, emit JSON Schema 2020-12 documents with `x-zennode-*` extensions for slider mappings, units, sections, and identity values: `node_to_json_schema` / `registry_to_json_schema` for per-node and full-pipeline schemas (`registry_to_openapi_schemas` packages them for OpenAPI 3.1), and `querystring_to_json_schema` / `registry_querystring_keys` for the flat RIAPI querystring surface.

## Graph topology

Most nodes have a single implicit input — the previous node's output. Multi-input nodes (compositing, montage, watermarking) declare their ports with `#[node(inputs(...))]`, which the derive turns into the `NodeSchema::inputs` slice of `InputPort` descriptors:

```rust
#[derive(Node, Clone, Debug, Default)]
#[node(id = "compose.over", group = Composite, role = Composite)]
#[node(inputs(canvas("Background"), input("Foreground")))]
pub struct Over {
    #[param(range(0.0..=1.0), default = 1.0, identity = 1.0)]
    pub opacity: f32,
}
```

Port kinds: `canvas("…")` (the background/canvas edge), `input("…")` (a normal data edge), `variadic("…")` (N-way fan-in), and `from_io("…")` (a source referenced by io_id rather than a graph edge — e.g. a watermark loaded from a separate buffer). Each `InputPort` records a name, label, `EdgeKind` (`Input` or `Canvas`), and the `required` / `variadic` / `from_io_id` flags, so a downstream code generator can emit correct call signatures (`DrawImage(other, x, y)` vs `Exposure(stops)`).

## Key types

| Type | Purpose |
|------|---------|
| `#[derive(Node)]` / `#[derive(NodeEnum)]` | generate the schema + `NodeDef` from a struct / enum |
| `NodeDef` / `NodeInstance` | the static definition vs. a constructed, parameterized node |
| `NodeSchema` / `ParamDesc` / `ParamKind` | the schema a node exposes |
| `NodeRegistry` | register nodes; `from_querystring`, `create`, `get`, `by_group`, `by_tag`, `to_markdown` |
| `ParamMap` / `ParamValue` | runtime parameter values |
| `NodeRole` (= `Phase`) / `NodeGroup` | pipeline + UI classification |
| `InputPort` / `EdgeKind` | graph input ports |
| `VersionSet` | per-id schema versioning for backwards compatibility |
| `NodeError` / `KvPairs` / `KvWarning` | typed errors and non-fatal parse warnings |

## Feature flags

| Feature | Default | Description |
|---------|---------|-------------|
| `std` | yes | `std::error::Error` impl on `NodeError` |
| `derive` | yes | `#[derive(Node)]` and `#[derive(NodeEnum)]` macros |
| `serde` | no | `Serialize`/`Deserialize` on param + schema types; JSON node/pipeline round-trip (implies `std`) |
| `json-schema` | no | JSON Schema 2020-12 generation (implies `serde`) |

The library is `no_std + alloc` compatible when both `std` and `derive` are disabled.

## Integration pattern

Each sibling crate in the zen ecosystem wires its nodes in the same way:

1. Depend on zennode behind a feature flag:
   ```toml
   [dependencies]
   zennode = { version = "0.1.1", optional = true }
   ```
2. Define nodes in a feature-gated module:
   ```rust
   #[cfg(feature = "zennode")]
   pub mod zennode_defs;
   ```
3. That module declares node structs with `#[derive(Node)]` and exposes a `register()` function:
   ```rust
   pub fn register(registry: &mut zennode::NodeRegistry) {
       registry.register(&ENCODE_JPEG_NODE);
       registry.register(&DECODE_JPEG_NODE);
   }
   ```
4. An aggregator crate (zenpipe) calls each crate's `register()` behind feature flags to build one registry of every available node.

This pattern is used across the zen codecs and processing crates — zenjpeg, zenpng, zenwebp, zengif, zenavif, zenjxl, zentiff, zenbitmaps, zenfilters, zenresize, zenlayout, zenquant, zencodecs, and more.

## Workspace structure

```
zennode/          # workspace root + GitHub README
  zennode/        # library crate (traits, schema, registry, KV parsing)
  zennode-derive/ # proc-macro crate (#[derive(Node)], #[derive(NodeEnum)])
```

## License

Licensed under either of [Apache-2.0](https://github.com/imazen/zennode/blob/main/LICENSE-APACHE) or [MIT](https://github.com/imazen/zennode/blob/main/LICENSE-MIT), at your option.

## Image tech I maintain

| | |
|:--|:--|
| **Codecs** ¹ | [zenjpeg] · [zenpng] · [zenwebp] · [zengif] · [zenavif] · [zenjxl] · [zenjxl-decoder] · [jxl-encoder] · [zenbitmaps] · [heic] · [zentiff] · [zenpdf] · [zensvg] · [zenjp2] · [zenraw] · [ultrahdr] |
| Codec internals | [zenrav1e] · [rav1d-safe] · [zenravif] · [zenavif-parse] · [zenavif-serialize] |
| Compression | [zenflate] · [zenzop] · [zenzstd] |
| Processing | [zenresize] · [zenquant] · [zenblend] · [zenfilters] · [zensally] · [zentone] |
| Pixels & color | [zenpixels] · [zenpixels-convert] · [linear-srgb] · [garb] · [zenyuv] |
| Pipeline & framework | [zenpipe] · [zencodec] · [zencodecs] · [zenlayout] · **zennode** · [zenwasm] · [zentract] |
| Metrics | [zensim] · [fast-ssim2] · [butteraugli] · [zenmetrics] · [resamplescope-rs] |
| Pickers & ML | [zenanalyze] · [zenpredict] · [zenpicker] · [zenanalyze-api] |
| Test corpora | [codec-corpus] · [imazen-26] |
| Products | [Imageflow] image engine ([.NET][imageflow-dotnet] · [Node][imageflow-node] · [Go][imageflow-go]) · [Imageflow Server] · [ImageResizer] (C#) |

<sub>¹ pure-Rust, `#![forbid(unsafe_code)]` codecs, as of 2026</sub>

### General Rust awesomeness

[zenbench] · [archmage] · [magetypes] · [enough] · [whereat] · [cargo-copter] · [zenutils]

[Open source](https://www.imazen.io/open-source) · [@imazen](https://github.com/imazen) · [@lilith](https://github.com/lilith) · [lib.rs/~lilith](https://lib.rs/~lilith)

[zenjpeg]: https://github.com/imazen/zenjpeg
[zenpng]: https://github.com/imazen/zenpng
[zenwebp]: https://github.com/imazen/zenwebp
[zengif]: https://github.com/imazen/zengif
[zenavif]: https://github.com/imazen/zenavif
[zenjxl]: https://github.com/imazen/zenjxl
[zenjxl-decoder]: https://github.com/imazen/zenjxl-decoder
[jxl-encoder]: https://github.com/imazen/jxl-encoder
[zenbitmaps]: https://github.com/imazen/zenbitmaps
[heic]: https://github.com/imazen/heic
[zentiff]: https://github.com/imazen/zenextras
[zenpdf]: https://github.com/imazen/zenextras
[zensvg]: https://github.com/imazen/zenextras
[zenjp2]: https://github.com/imazen/zenextras
[zenraw]: https://github.com/imazen/zenraw
[ultrahdr]: https://github.com/imazen/ultrahdr
[zenrav1e]: https://github.com/imazen/zenrav1e
[rav1d-safe]: https://github.com/imazen/rav1d-safe
[zenravif]: https://github.com/imazen/cavif-rs
[zenavif-parse]: https://github.com/imazen/zenavif
[zenavif-serialize]: https://github.com/imazen/zenavif
[zenflate]: https://github.com/imazen/zenflate
[zenzop]: https://github.com/imazen/zenzop
[zenzstd]: https://github.com/imazen/zenzstd
[zenresize]: https://github.com/imazen/zenresize
[zenquant]: https://github.com/imazen/zenquant
[zenblend]: https://github.com/imazen/zenblend
[zenfilters]: https://github.com/imazen/zenpipe
[zensally]: https://github.com/imazen/zensally
[zentone]: https://github.com/imazen/zentone
[zenpixels]: https://github.com/imazen/zenpixels
[zenpixels-convert]: https://github.com/imazen/zenpixels
[linear-srgb]: https://github.com/imazen/linear-srgb
[garb]: https://github.com/imazen/garb
[zenyuv]: https://github.com/imazen/zenjpeg
[zenpipe]: https://github.com/imazen/zenpipe
[zencodec]: https://github.com/imazen/zencodec
[zencodecs]: https://github.com/imazen/zenpipe
[zenlayout]: https://github.com/imazen/zenpipe
[zenwasm]: https://github.com/imazen/zenwasm
[zentract]: https://github.com/imazen/zentract
[zensim]: https://github.com/imazen/zensim
[fast-ssim2]: https://github.com/imazen/fast-ssim2
[butteraugli]: https://github.com/imazen/butteraugli
[zenmetrics]: https://github.com/imazen/zenmetrics
[resamplescope-rs]: https://github.com/imazen/resamplescope-rs
[zenanalyze]: https://github.com/imazen/zenanalyze
[zenpredict]: https://github.com/imazen/zenanalyze
[zenpicker]: https://github.com/imazen/zenanalyze
[zenanalyze-api]: https://github.com/imazen/zenanalyze
[codec-corpus]: https://github.com/imazen/codec-corpus
[imazen-26]: https://github.com/imazen/imazen-26
[zenbench]: https://github.com/imazen/zenbench
[archmage]: https://github.com/imazen/archmage
[magetypes]: https://github.com/imazen/archmage
[enough]: https://github.com/imazen/enough
[whereat]: https://github.com/lilith/whereat
[cargo-copter]: https://github.com/imazen/cargo-copter
[zenutils]: https://github.com/imazen/zenutils
[Imageflow]: https://github.com/imazen/imageflow
[Imageflow Server]: https://github.com/imazen/imageflow-dotnet-server
[ImageResizer]: https://github.com/imazen/resizer
[imageflow-dotnet]: https://github.com/imazen/imageflow-dotnet
[imageflow-node]: https://github.com/imazen/imageflow-node
[imageflow-go]: https://github.com/imazen/imageflow-go
