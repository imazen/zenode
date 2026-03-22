//! Tests for built-in shared nodes (Decode, QualityIntent).

extern crate alloc;

use zenode::nodes::*;
use zenode::*;

#[test]
fn decode_schema() {
    let schema = DECODE_NODE.schema();
    assert_eq!(schema.id, "zenode.decode");
    assert_eq!(schema.group, NodeGroup::Decode);
    assert_eq!(schema.phase, Phase::Decode);
    assert_eq!(schema.params.len(), 1);
    assert_eq!(schema.params[0].name, "io_id");
}

#[test]
fn decode_create_default() {
    let node = DECODE_NODE.create_default().unwrap();
    assert_eq!(node.get_param("io_id"), Some(ParamValue::I32(0)));
}

#[test]
fn decode_from_kv_no_keys() {
    // Decode has no KV keys, so from_kv should return None
    let mut kv = KvPairs::from_querystring("w=800");
    let result = DECODE_NODE.from_kv(&mut kv).unwrap();
    assert!(result.is_none());
}

#[test]
fn quality_intent_schema() {
    let schema = QUALITY_INTENT_NODE.schema();
    assert_eq!(schema.id, "zenode.quality_intent");
    assert_eq!(schema.group, NodeGroup::Encode);
    assert_eq!(schema.phase, Phase::Encode);
    assert!(schema.tags.contains(&"quality"));
    assert!(schema.tags.contains(&"auto"));

    // Check params exist
    let param_names: Vec<&str> = schema.params.iter().map(|p| p.name).collect();
    assert!(param_names.contains(&"profile"));
    assert!(param_names.contains(&"dpr"));
    assert!(param_names.contains(&"lossless"));
    assert!(param_names.contains(&"allow_webp"));
    assert!(param_names.contains(&"allow_avif"));
    assert!(param_names.contains(&"allow_jxl"));
    assert!(param_names.contains(&"allow_color_profiles"));
}

#[test]
fn quality_intent_defaults() {
    let node = QUALITY_INTENT_NODE.create_default().unwrap();
    assert_eq!(
        node.get_param("profile"),
        Some(ParamValue::Str("high".into()))
    );
    assert_eq!(node.get_param("dpr"), Some(ParamValue::F32(1.0)));
    assert_eq!(node.get_param("lossless"), Some(ParamValue::Bool(false)));
    assert_eq!(node.get_param("allow_webp"), Some(ParamValue::Bool(false)));
}

#[test]
fn quality_intent_from_kv_qp() {
    let mut kv = KvPairs::from_querystring("qp=medium&accept.webp=true&accept.avif=true");
    let node = QUALITY_INTENT_NODE.from_kv(&mut kv).unwrap().unwrap();
    assert_eq!(
        node.get_param("profile"),
        Some(ParamValue::Str("medium".into()))
    );
    assert_eq!(node.get_param("allow_webp"), Some(ParamValue::Bool(true)));
    assert_eq!(node.get_param("allow_avif"), Some(ParamValue::Bool(true)));
    assert_eq!(node.get_param("allow_jxl"), Some(ParamValue::Bool(false))); // not in qs
    assert_eq!(kv.unconsumed().count(), 0);
}

#[test]
fn quality_intent_from_kv_dpr() {
    let mut kv = KvPairs::from_querystring("qp=high&dpr=2.0");
    let node = QUALITY_INTENT_NODE.from_kv(&mut kv).unwrap().unwrap();
    assert_eq!(
        node.get_param("profile"),
        Some(ParamValue::Str("high".into()))
    );
    assert_eq!(node.get_param("dpr"), Some(ParamValue::F32(2.0)));
}

#[test]
fn quality_intent_from_kv_lossless() {
    let mut kv = KvPairs::from_querystring("qp=highest&lossless=true");
    let node = QUALITY_INTENT_NODE.from_kv(&mut kv).unwrap().unwrap();
    assert_eq!(node.get_param("lossless"), Some(ParamValue::Bool(true)));
}

#[test]
fn quality_intent_from_kv_no_match() {
    let mut kv = KvPairs::from_querystring("w=800&h=600");
    let result = QUALITY_INTENT_NODE.from_kv(&mut kv).unwrap();
    assert!(result.is_none());
}

#[test]
fn quality_intent_kv_keys_are_correct() {
    let schema = QUALITY_INTENT_NODE.schema();
    let profile_param = schema.params.iter().find(|p| p.name == "profile").unwrap();
    assert_eq!(profile_param.kv_keys, &["qp"]);

    let dpr_param = schema.params.iter().find(|p| p.name == "dpr").unwrap();
    assert!(dpr_param.kv_keys.contains(&"qp.dpr"));
    assert!(dpr_param.kv_keys.contains(&"dpr"));
}

#[test]
fn registry_with_shared_nodes() {
    let mut registry = NodeRegistry::new();
    registry.register(&DECODE_NODE);
    registry.register(&QUALITY_INTENT_NODE);

    // RIAPI querystring that uses quality intent
    let result = registry.from_querystring("qp=good&accept.webp=true&accept.jxl=true");
    assert_eq!(result.instances.len(), 1); // only QualityIntent matches
    let qi = &result.instances[0];
    assert_eq!(qi.schema().id, "zenode.quality_intent");
    assert_eq!(
        qi.get_param("profile"),
        Some(ParamValue::Str("good".into()))
    );
    assert_eq!(qi.get_param("allow_webp"), Some(ParamValue::Bool(true)));
    assert_eq!(qi.get_param("allow_jxl"), Some(ParamValue::Bool(true)));
}
