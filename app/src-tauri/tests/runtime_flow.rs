use ielts_core::runtime;
#[test]
fn exported_room_preserves_skeleton_facial_morphs_and_animation() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../rooms/speaking/runtime/speaking-room.glb");
    let data = runtime::read(&path).unwrap();
    let doc = runtime::validate_glb(&data).unwrap();
    assert_eq!(doc["scenes"].as_array().unwrap().len(), 1);
    assert!(!doc["skins"].as_array().unwrap().is_empty());
    assert!(!doc["animations"].as_array().unwrap().is_empty());
    let names: Vec<&str> = doc["meshes"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|m| {
            m["extras"]["targetNames"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(|s| s.as_str())
        })
        .collect();
    assert!(names.contains(&"jawOpen"));
    assert!(names.contains(&"eyeBlinkLeft"));
    assert!(!doc["nodes"]
        .as_array()
        .unwrap()
        .iter()
        .any(|n| n["name"] == "Cube"));
}
#[test]
fn malformed_model_is_rejected() {
    assert!(runtime::validate_glb(b"not gltf").is_err());
}
#[test]
fn authored_eye_brow_and_hair_shader_effects_are_present_in_runtime_textures() {
    let path=std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../rooms/speaking/runtime/speaking-room.glb");
    let doc=runtime::validate_glb(&runtime::read(&path).unwrap()).unwrap();
    for material in doc["materials"].as_array().unwrap(){
      let name=material["name"].as_str().unwrap_or("");
      if name.contains("RebuiltHair"){
        assert!(material["pbrMetallicRoughness"]["baseColorTexture"].is_object(),"hair root/tip alpha and color lost");
        assert_eq!(material["alphaMode"],"MASK","hair must retain cutout edges");
        assert!(material["normalTexture"].is_object(),"authored hair detail lost");
      }
      if name.contains("eyebrow002"){
        assert_eq!(material["alphaMode"],"BLEND");
      }
    }
}
#[test]
fn card_text_inherits_the_animated_card_and_solid_skin_writes_depth() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../rooms/speaking/runtime/speaking-room.glb");
    let doc = runtime::validate_glb(&runtime::read(&path).unwrap()).unwrap();
    let nodes = doc["nodes"].as_array().unwrap();
    let card = nodes
        .iter()
        .position(|n| n["name"] == "P2_TopicCard")
        .unwrap();
    let mut descendants = vec![card];
    let mut index = 0;
    while index < descendants.len() {
        let node = descendants[index];
        descendants.extend(
            nodes[node]["children"]
                .as_array()
                .into_iter()
                .flatten()
                .map(|v| v.as_u64().unwrap() as usize),
        );
        index += 1;
    }
    for (i, node) in nodes.iter().enumerate() {
        if node["name"]
            .as_str()
            .is_some_and(|n| n.starts_with("P2_CardText"))
        {
            assert!(
                descendants.contains(&i),
                "Card text detached from moving card"
            );
        }
    }
    for material in doc["materials"].as_array().unwrap() {
        if material["name"].as_str().is_some_and(|n| {
            n.contains(".body.") || n.contains("female_suit")
        }) {
            assert!(material["alphaMode"].is_null() || material["alphaMode"] == "OPAQUE");
        }
    }
}
