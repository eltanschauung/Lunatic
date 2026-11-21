fn bamboo_worldgen(overrides: &WorldgenOverrides, bundle: &str, base: &mut BTreeMap<String, String>) {
    if bundle != "features" || !overrides.mod_enabled("lunatic") { return; }
    for text in base.values_mut() {
        let Ok(mut feature) = serde_json::from_str::<serde_json::Value>(text) else { continue; };
        if feature.get("type").and_then(|v| v.as_str()) != Some("minecraft:bamboo") { continue; }
        let Some(feature) = feature.as_object_mut() else { continue; };
        // Respect a pack's explicit selector; otherwise use seeded 16-block patches.
        if feature.contains_key("patch") { continue; }
        feature.insert("patch".into(), serde_json::json!({
            "block":"lunatic:bushy_bamboo", "size":16, "chance":0.35
        }));
        *text = serde_json::to_string(feature).expect("serializable bamboo feature");
    }
}
