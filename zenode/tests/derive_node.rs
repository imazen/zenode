//! Integration tests for `#[derive(Node)]`.

extern crate alloc;

use zenode::*;

/// Exposure adjustment in photographic stops.
/// Each stop doubles or halves brightness.
#[derive(Node, Clone, Debug, Default)]
#[node(id = "test.exposure", group = Tone, phase = DisplayAdjust)]
#[node(coalesce = "fused_adjust")]
#[node(format(preferred = OklabF32, alpha = Skip))]
#[node(tags("basic", "tone"))]
pub struct Exposure {
    /// Exposure compensation in stops
    #[param(range(-5.0..=5.0), default = 0.0, identity = 0.0, step = 0.1)]
    #[param(unit = "EV", section = "Main")]
    #[kv("exposure", "ev")]
    pub stops: f32,
}

/// Resize to specified dimensions.
#[derive(Node, Clone, Debug, Default)]
#[node(id = "test.resize", group = Geometry, phase = Resize)]
#[node(changes_dimensions)]
#[node(format(preferred = LinearF32, alpha = RequirePremul))]
pub struct Resize {
    /// Target width in pixels
    #[param(range(0..=65535), default = 0, step = 1)]
    #[param(unit = "px", section = "Main")]
    #[kv("w", "width")]
    pub width: u32,

    /// Target height in pixels
    #[param(range(0..=65535), default = 0, step = 1)]
    #[param(unit = "px", section = "Main")]
    #[kv("h", "height")]
    pub height: u32,

    /// Whether to use sharp downsampling
    #[param(default = false)]
    #[param(section = "Advanced")]
    #[kv("sharp")]
    pub sharp_yuv: bool,
}

#[test]
fn schema_metadata() {
    let schema = EXPOSURE_NODE.schema();
    assert_eq!(schema.id, "test.exposure");
    assert_eq!(schema.label, "Exposure");
    assert_eq!(schema.group, NodeGroup::Tone);
    assert_eq!(schema.phase, Phase::DisplayAdjust);
    assert_eq!(schema.version, 1);
    assert_eq!(schema.tags, &["basic", "tone"]);
    assert!(schema.coalesce.is_some());
    let coal = schema.coalesce.as_ref().unwrap();
    assert_eq!(coal.group, "fused_adjust");
    assert!(coal.fusable);
    assert!(!coal.is_target);
    assert_eq!(schema.format.preferred, PixelFormatPreference::OklabF32);
    assert_eq!(schema.format.alpha, AlphaHandling::Skip);
}

#[test]
fn param_metadata() {
    let schema = EXPOSURE_NODE.schema();
    assert_eq!(schema.params.len(), 1);
    let p = &schema.params[0];
    assert_eq!(p.name, "stops");
    assert_eq!(p.label, "Stops");
    assert_eq!(p.unit, "EV");
    assert_eq!(p.section, "Main");
    assert_eq!(p.kv_keys, &["exposure", "ev"]);
    match &p.kind {
        ParamKind::Float { min, max, default, identity, step } => {
            assert_eq!(*min, -5.0);
            assert_eq!(*max, 5.0);
            assert_eq!(*default, 0.0);
            assert_eq!(*identity, 0.0);
            assert_eq!(*step, 0.1);
        }
        _ => panic!("expected Float"),
    }
}

#[test]
fn create_default() {
    let instance = EXPOSURE_NODE.create_default().unwrap();
    assert_eq!(instance.schema().id, "test.exposure");
    assert_eq!(instance.get_param("stops"), Some(ParamValue::F32(0.0)));
    assert!(instance.is_identity());
}

#[test]
fn create_with_params() {
    let mut params = ParamMap::new();
    params.insert("stops".into(), ParamValue::F32(1.5));
    let instance = EXPOSURE_NODE.create(&params).unwrap();
    assert_eq!(instance.get_param("stops"), Some(ParamValue::F32(1.5)));
    assert!(!instance.is_identity());
}

#[test]
fn set_and_get_param() {
    let mut exp = Exposure::default();
    assert!(exp.set_param("stops", ParamValue::F32(2.0)));
    assert_eq!(exp.get_param("stops"), Some(ParamValue::F32(2.0)));
    assert!(!exp.set_param("nonexistent", ParamValue::F32(1.0)));
}

#[test]
fn to_params_round_trip() {
    let exp = Exposure { stops: 1.5 };
    let params = exp.to_params();
    assert_eq!(params.get("stops"), Some(&ParamValue::F32(1.5)));

    let instance = EXPOSURE_NODE.create(&params).unwrap();
    assert_eq!(instance.get_param("stops"), Some(ParamValue::F32(1.5)));
}

#[test]
fn from_kv_basic() {
    let mut kv = KvPairs::from_querystring("exposure=1.5");
    let instance = EXPOSURE_NODE.from_kv(&mut kv).unwrap().unwrap();
    assert_eq!(instance.get_param("stops"), Some(ParamValue::F32(1.5)));
    assert_eq!(kv.unconsumed().count(), 0);
}

#[test]
fn from_kv_alias() {
    let mut kv = KvPairs::from_querystring("ev=2.0");
    let instance = EXPOSURE_NODE.from_kv(&mut kv).unwrap().unwrap();
    assert_eq!(instance.get_param("stops"), Some(ParamValue::F32(2.0)));
}

#[test]
fn from_kv_no_match() {
    let mut kv = KvPairs::from_querystring("w=800");
    let result = EXPOSURE_NODE.from_kv(&mut kv).unwrap();
    assert!(result.is_none());
}

#[test]
fn multi_field_node() {
    let schema = RESIZE_NODE.schema();
    assert_eq!(schema.params.len(), 3);
    assert_eq!(schema.params[0].name, "width");
    assert_eq!(schema.params[1].name, "height");
    assert_eq!(schema.params[2].name, "sharp_yuv");
    assert!(schema.format.changes_dimensions);
}

#[test]
fn multi_field_from_kv() {
    let mut kv = KvPairs::from_querystring("w=800&h=600&sharp=true");
    let instance = RESIZE_NODE.from_kv(&mut kv).unwrap().unwrap();
    assert_eq!(instance.get_param("width"), Some(ParamValue::U32(800)));
    assert_eq!(instance.get_param("height"), Some(ParamValue::U32(600)));
    assert_eq!(instance.get_param("sharp_yuv"), Some(ParamValue::Bool(true)));
    assert_eq!(kv.unconsumed().count(), 0);
}

#[test]
fn downcast() {
    let instance = EXPOSURE_NODE.create_default().unwrap();
    let exp = instance.as_any().downcast_ref::<Exposure>().unwrap();
    assert_eq!(exp.stops, 0.0);
}

#[test]
fn clone_boxed() {
    let instance = EXPOSURE_NODE.create_default().unwrap();
    let cloned = instance.clone_boxed();
    assert_eq!(cloned.get_param("stops"), instance.get_param("stops"));
}

#[test]
fn registry_basic() {
    let mut registry = NodeRegistry::new();
    registry.register(&EXPOSURE_NODE);
    registry.register(&RESIZE_NODE);

    assert!(registry.get("test.exposure").is_some());
    assert!(registry.get("test.resize").is_some());
    assert!(registry.get("test.nonexistent").is_none());
    assert_eq!(registry.all().len(), 2);
}

#[test]
fn registry_from_querystring() {
    let mut registry = NodeRegistry::new();
    registry.register(&EXPOSURE_NODE);
    registry.register(&RESIZE_NODE);

    let result = registry.from_querystring("exposure=1.5&w=800&h=600");
    assert_eq!(result.instances.len(), 2);
    // Exposure should have stops=1.5
    let exp = &result.instances[0];
    assert_eq!(exp.schema().id, "test.exposure");
    assert_eq!(exp.get_param("stops"), Some(ParamValue::F32(1.5)));
    // Resize should have width=800, height=600
    let rsz = &result.instances[1];
    assert_eq!(rsz.schema().id, "test.resize");
    assert_eq!(rsz.get_param("width"), Some(ParamValue::U32(800)));
}

#[test]
fn registry_warns_unrecognized_keys() {
    let mut registry = NodeRegistry::new();
    registry.register(&EXPOSURE_NODE);

    let result = registry.from_querystring("exposure=1.5&unknown=foo");
    assert_eq!(result.instances.len(), 1);
    assert!(result
        .warnings
        .iter()
        .any(|w| w.kind == KvWarningKind::UnrecognizedKey && w.key == "unknown"));
}
