# zennode ![CI](https://img.shields.io/github/actions/workflow/status/imazen/zennode/ci.yml?style=for-the-badge) ![MSRV](https://img.shields.io/badge/MSRV-1.85-blue?style=for-the-badge) ![License](https://img.shields.io/badge/license-Apache--2.0%20OR%20MIT-blue?style=for-the-badge)

A self-describing node definition system for image processing pipelines.

zennode provides the trait infrastructure, derive macros, parameter schemas, RIAPI querystring parsing, JSON Schema generation, and node registry that the zen ecosystem uses to define pipeline operations. It defines no nodes itself -- nodes live in the crates that implement them.

## Quick Start

```rust
use zennode::*;

#[derive(Node, Clone, Debug, Default)]
#[node(id = "myfilter.brightness", group = Tone, role = Filter)]
pub struct Brightness {
    /// Amount of brightness adjustment
    #[param(range(-1.0..=1.0), default = 0.0, identity = 0.0, step = 0.05)]
    #[param(unit = "", section = "Main")]
    pub amount: f32,
}
```

The `#[derive(Node)]` macro generates a `NodeDef` implementation (the factory/schema holder), a `NodeInstance` implementation (a live instance with parameter values), and a static singleton -- all from the struct definition. Doc comments on fields become parameter descriptions. The full `NodeSchema` is available at zero cost through `&'static` references.

Parameter enums get their own derive:

```rust
#[derive(NodeEnum, Clone, Debug, Default)]
pub enum FitMode {
    #[default]
    #[variant(label = "Max", description = "Fit within bounds")]
    Max,
    #[variant(label = "Crop", description = "Fill bounds, crop excess")]
    Crop,
}
```

## What It Provides

**Schema introspection.** Every node carries a `NodeSchema` with 13 fields (id, label, description, group, role, params, tags, coalesce info, format hints, version, compat_version, json_key, deny_unknown_fields). Every parameter carries a `ParamDesc` with 14 fields (name, label, description, kind, unit, section, slider mapping, kv_keys, since_version, visible_when, optional, json_name, json_aliases). Parameter types span 11 `ParamKind` variants: Float, Int, U32, Bool, Str, Enum, FloatArray, Color, Json, Object, and TaggedUnion.

**Node registry.** `NodeRegistry` aggregates node definitions from across the ecosystem. Look up nodes by id, group, or tag. Create instances from parameter maps. Parse RIAPI querystrings against all registered nodes (with consumption tracking and warnings for unrecognized keys). Generate Markdown documentation for every registered node.

**RIAPI querystring parsing.** `KvPairs` is a consumption-tracking parser -- multiple node definitions each claim their relevant keys from the same querystring, and unconsumed keys generate warnings. Typed accessors (`take_f32`, `take_i32`, `take_bool`) handle parsing and error reporting.

**Operation coalescing.** `NodeRole` (9 roles: Decode, Geometry, Orient, Resize, Filter, Composite, Analysis, Quantize, Encode) tells the pipeline bridge what kind of planner a node feeds into. `CoalesceInfo` marks adjacent compatible nodes for fusion into a single operation -- always equivalent to sequential execution, never reordering.

**JSON round-trip.** With the `serde` feature, the registry can serialize/deserialize individual nodes and full pipelines as JSON. Nodes serialize as `{"json_key": {...params...}}`, pipelines as arrays. Unknown field rejection is opt-in per node via `#[node(deny_unknown_fields)]`.

**JSON Schema generation.** With the `json-schema` feature, generate JSON Schema 2020-12 documents with `x-zennode-*` extensions for slider mappings, units, sections, and identity values. Produces both per-node schemas and full pipeline schemas suitable for OpenAPI 3.1.

## Feature Flags

| Feature | Default | Description |
|---------|---------|-------------|
| `std` | yes | `std::error::Error` impl |
| `derive` | yes | `#[derive(Node)]` and `#[derive(NodeEnum)]` macros |
| `serde` | no | Serialize/Deserialize on param types, JSON pipeline round-trip (implies `std`) |
| `json-schema` | no | JSON Schema 2020-12 generation (implies `serde`) |

The library is `no_std + alloc` compatible when both `std` and `derive` are disabled.

## Integration Pattern

Each sibling crate in the zen ecosystem follows the same pattern:

1. Add zennode as an optional dependency behind a feature flag:
   ```toml
   [dependencies]
   zennode = { version = "0.1", path = "../zennode/zennode", optional = true }
   ```

2. Define nodes in a `zennode_defs` module, feature-gated:
   ```rust
   #[cfg(feature = "zennode")]
   pub mod zennode_defs;
   ```

3. That module defines node structs with `#[derive(Node)]` and provides a `register()` function:
   ```rust
   pub fn register(registry: &mut zennode::NodeRegistry) {
       registry.register(&ENCODE_JPEG_DEF);
       registry.register(&DECODE_JPEG_DEF);
   }
   ```

4. The aggregator crate (zenpipe) calls each crate's `register()` behind feature flags to build a `full_registry()` containing every available node.

This pattern is used by zenjpeg, zenpng, zenwebp, zengif, zenavif, zenjxl, zentiff, zenbitmaps, zenfilters, zenresize, zenlayout, zenquant, and zencodecs.

## Workspace Structure

```
zennode/          # Workspace root
  zennode/        # Library crate (traits, schema, registry, KV parsing)
  zennode-derive/ # Proc macro crate (#[derive(Node)], #[derive(NodeEnum)])
```

## License

Apache-2.0 OR MIT
