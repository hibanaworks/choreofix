use super::*;

// These fixtures isolate static wire/actor checks. Their source rows declare
// the full inventory; the complete EPF format is tested in dynamic.rs.
fn declare_fixture_roles(programs: &mut [Value]) {
    let source: Vec<_> = programs
        .iter()
        .enumerate()
        .map(|(index, program)| {
            let role = program["role"].as_u64().unwrap();
            json!([
                index,
                hibana::runtime::tap::PROJECTED_SOURCE,
                0,
                role << 24 | role << 16,
                0
            ])
        })
        .collect();
    for program in programs {
        program["rows"]
            .as_array_mut()
            .unwrap()
            .extend(source.clone());
    }
}

fn artifact() -> Value {
    let mut programs = Vec::new();
    for role in [0, 1, 2, 5, 6, 7, 8, 9, 10, 13] {
        let actor = if role == 13 { 7 } else { role };
        let mut rows = Vec::new();
        let actor_word = actor << 8 | role;
        if role == 0 {
            rows.push(json!([0, SEND, actor_word, 2 << 8 | 7, 1 << 16]));
            rows.push(json!([
                1,
                RECV,
                actor_word,
                256 << 16 | 3 << 8 | 10,
                7 << 24 | 1 << 8 | 13
            ]));
        } else if role == 1 {
            rows.push(json!([0, RECV, actor_word, 2 << 8 | 7, 1 << 16]));
        } else if role == 7 || role == 13 {
            rows.push(json!([
                0,
                SEND,
                actor_word,
                256 << 16 | 3 << 8 | 10,
                7 << 24 | 1 << 8 | 13
            ]));
        }
        programs.push(json!({"role":role,"rows":rows}));
    }
    declare_fixture_roles(&mut programs);
    json!({"format_version":1,"runtime_binary_sha256":"a".repeat(64),
        "compiled":{"type":"compiled_projection","format_version":1,"task":"6E",
        "roles":{"a":0,"b":1,"review":2,"review_a":5,"review_b":6,"repo":7,
            "a_io":8,"b_io":9,"review_io":10,"repository":13},"programs":programs}})
}

fn rows(artifact: &Value) -> Vec<Value> {
    vec![
        json!({"direction":"runtime","message":artifact["compiled"]}),
        json!({"direction":"runtime","message":{"type":"protocol_frame","action":"send","frame_id":8,
            "session":6,"frame_label":0,"lane":2,"source":0,"target":1,"payload":[1,2,3]}}),
        json!({"direction":"runtime","message":{"type":"protocol_frame","action":"receive","frame_id":8,
            "session":6,"frame_label":0,"lane":2,"source":0,"target":1,"payload":[1,2,3]}}),
    ]
}

#[test]
fn matching_observations_cannot_hide_a_wrong_compiled_header() -> Result<()> {
    let artifact = artifact();
    let original = rows(&artifact);
    assert_eq!(frames(&artifact, &original)?, BTreeMap::from([(8, 7)]));
    for (field, value) in [
        ("frame_label", 65535),
        ("lane", 3),
        ("source", 7),
        ("target", 10),
    ] {
        let mut changed = original.clone();
        for row in &mut changed[1..] {
            row["message"][field] = json!(value);
        }
        let mut send = changed[1]["message"].clone();
        let mut receive = changed[2]["message"].clone();
        send.as_object_mut().unwrap().remove("action");
        receive.as_object_mut().unwrap().remove("action");
        assert_eq!(send, receive, "the old identity-only condition still holds");
        assert!(frames(&artifact, &changed).is_err(), "accepted {field}");
    }
    Ok(())
}

#[test]
fn a_new_named_role_task_checks_its_published_inventory_without_another_allow_list() -> Result<()> {
    let mut image = artifact();
    for task in ["10M", "11E"] {
        image["compiled"]["task"] = json!(task);
        assert_eq!(contracts(&image["compiled"])?.len(), 2);
    }
    image["compiled"]["roles"]["extra_named_role"] = json!(42);
    assert!(contracts(&image["compiled"]).is_err());
    image["compiled"]["task"] = json!("unknown");
    assert!(contracts(&image["compiled"]).is_err());
    Ok(())
}

#[test]
fn real_source_inventory_keeps_missing_actors_visible_without_task_role_tables() -> Result<()> {
    use hibana::{Endpoint, g, runtime::program::project};
    type File<'run> = g::Msg<10, Endpoint<'run, 35>>;
    type Request = g::Msg<20, u32>;
    type Reply = g::Msg<21, u32>;
    let global = g::seq(
        g::send::<42, 14, File>(),
        g::seq(g::send::<35, 42, Request>(), g::send::<42, 35, Reply>()),
    );
    let image = json!({"type":"compiled_projection","format_version":1,"task":"9E",
    "programs":[
        {"role":42,"rows":hibana::runtime::tap::projected(&project::<42>(&global)).map(|r|[r.ts(),u32::from(r.id()),u32::from(r.causal_key()),r.arg0(),r.arg1()]).collect::<Vec<_>>()},
        {"role":14,"rows":hibana::runtime::tap::projected(&project::<14>(&global)).map(|r|[r.ts(),u32::from(r.id()),u32::from(r.causal_key()),r.arg0(),r.arg1()]).collect::<Vec<_>>()},
        {"role":35,"rows":hibana::runtime::tap::projected(&project::<35>(&global)).map(|r|[r.ts(),u32::from(r.id()),u32::from(r.causal_key()),r.arg0(),r.arg1()]).collect::<Vec<_>>()},
    ]});
    assert_eq!(contracts(&image)?.len(), 3);
    let mut incomplete = image;
    incomplete["programs"]
        .as_array_mut()
        .unwrap()
        .retain(|p| p["role"] == 14);
    assert!(contracts(&incomplete).is_err());
    Ok(())
}

#[test]
fn an_owned_payload_preserves_the_full_occurrence_and_resource() -> Result<()> {
    let artifact = artifact();
    let mut rows = rows(&artifact);
    for row in &mut rows[1..] {
        let message = &mut row["message"];
        message["frame_label"] = json!(256);
        message["lane"] = json!(3);
        message["source"] = json!(7);
        message["target"] = json!(0);
        message["payload"] = json!([13, 1, 0]);
    }
    assert_eq!(frames(&artifact, &rows)?, BTreeMap::from([(8, 10)]));
    for payload in [json!([13, 0, 0]), json!([12, 1, 0]), json!([13, 1, 0, 0])] {
        let mut changed = rows.clone();
        for row in &mut changed[1..] {
            row["message"]["payload"] = payload.clone();
        }
        assert!(frames(&artifact, &changed).is_err());
    }
    Ok(())
}

#[test]
fn programs_must_cover_both_directions_and_preserve_actor_aliases() -> Result<()> {
    let original = artifact();
    let mut missing = original.clone();
    missing["compiled"]["programs"]
        .as_array_mut()
        .unwrap()
        .pop();
    assert!(contracts(&missing["compiled"]).is_err());
    let mut wrong_actor = original.clone();
    wrong_actor["compiled"]["programs"][9]["rows"][0][2] = json!(0x0d0d);
    assert!(contracts(&wrong_actor["compiled"]).is_err());
    let mut missing_receive = original.clone();
    missing_receive["compiled"]["programs"][1]["rows"] = json!([]);
    assert!(contracts(&missing_receive["compiled"]).is_err());
    let mut conflicting = original.clone();
    conflicting["compiled"]["programs"][9]["rows"][0][3] = json!(256 << 16 | 4 << 8 | 10);
    assert!(contracts(&conflicting["compiled"]).is_err());
    assert_eq!(contracts(&original["compiled"])?.len(), 2);
    Ok(())
}

#[test]
fn runtime_observation_must_match_the_separately_frozen_projection() -> Result<()> {
    let artifact = artifact();
    let original = rows(&artifact);
    let mut changed = original.clone();
    changed[0]["message"]["programs"][0]["rows"][0][3] = json!(3 << 8 | 7);
    assert!(frames(&artifact, &changed).is_err());
    assert!(frames(&artifact, &original[1..]).is_err());
    let mut duplicate = original.clone();
    duplicate.insert(1, duplicate[0].clone());
    assert!(frames(&artifact, &duplicate).is_err());
    let mut late = original.clone();
    late.swap(0, 1);
    assert!(frames(&artifact, &late).is_err());
    Ok(())
}

#[test]
fn the_projection_artifact_must_match_the_saved_run_receipt() -> Result<()> {
    let directory = std::env::temp_dir().join(format!(
        "choreofix-projection-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos()
    ));
    fs::create_dir(&directory)?;
    let artifact = artifact();
    let path = directory.join("projection.json");
    fs::write(&path, serde_json::to_vec(&artifact)?)?;
    let receipt = json!({"runtime_binary_sha256":artifact["runtime_binary_sha256"],
        "compiled_projection_sha256":digest(&path)?});
    fs::write(directory.join("case.json"), serde_json::to_vec(&receipt)?)?;
    let events = directory.join("events.jsonl");
    assert_eq!(load(&events)?, artifact);
    let mut wrong = receipt.clone();
    wrong["runtime_binary_sha256"] = json!("b".repeat(64));
    fs::write(directory.join("case.json"), serde_json::to_vec(&wrong)?)?;
    assert!(load(&events).is_err());
    fs::remove_file(directory.join("case.json"))?;
    let versions = json!({"binaries":{"runtime":receipt["runtime_binary_sha256"]},
        "compiled_projection_sha256":receipt["compiled_projection_sha256"]});
    fs::write(
        directory.join("versions.json"),
        serde_json::to_vec(&versions)?,
    )?;
    assert_eq!(load(&events)?, artifact);
    let mut case = json!({"versions":versions});
    fs::write(directory.join("case.json"), serde_json::to_vec(&case)?)?;
    assert_eq!(load(&events)?, artifact);
    case["versions"]["binaries"]["runtime"] = json!("b".repeat(64));
    fs::write(directory.join("case.json"), serde_json::to_vec(&case)?)?;
    assert!(load(&events).is_err());
    fs::remove_file(directory.join("case.json"))?;
    fs::write(&path, b"{}")?;
    assert!(load(&events).is_err());
    fs::remove_dir_all(directory)?;
    Ok(())
}

fn cicd_artifact() -> Value {
    let mut programs = Vec::new();
    for role in 0..=12 {
        let actor = match role {
            11 => 6,
            12 => 7,
            role => role,
        };
        let mut rows = Vec::new();
        let context = actor << 8 | role;
        if role == 0 {
            rows.push(json!([0, SEND, context, 2 << 8 | 7, 3 << 16]));
            rows.push(json!([
                1,
                RECV,
                context,
                256 << 16 | 3 << 8 | 10,
                6 << 24 | 1 << 8 | 11
            ]));
        } else if role == 3 {
            rows.push(json!([0, RECV, context, 2 << 8 | 7, 3 << 16]));
        } else if actor == 6 {
            rows.push(json!([
                0,
                SEND,
                context,
                256 << 16 | 3 << 8 | 10,
                6 << 24 | 1 << 8 | 11
            ]));
        } else if actor == 7 {
            rows.push(json!([
                0,
                SEND,
                context,
                257 << 16 | 4 << 8 | 10,
                7 << 24 | 2 << 16 | 1 << 8 | 12
            ]));
        } else if role == 2 {
            rows.push(json!([
                0,
                RECV,
                context,
                257 << 16 | 4 << 8 | 10,
                7 << 24 | 2 << 16 | 1 << 8 | 12
            ]));
        }
        programs.push(json!({"role":role,"rows":rows}));
    }
    declare_fixture_roles(&mut programs);
    json!({"format_version":1,"runtime_binary_sha256":"a".repeat(64),
        "compiled":{"type":"compiled_projection","format_version":1,"task":"16E","programs":programs}})
}

#[test]
fn cicd_frames_use_both_compiled_resource_bindings_and_reject_coherent_header_changes() -> Result<()>
{
    let artifact = cicd_artifact();
    let mut original = rows(&artifact);
    for row in &mut original[1..] {
        row["message"]["session"] = json!(16);
        row["message"]["target"] = json!(3);
    }
    assert_eq!(frames(&artifact, &original)?, BTreeMap::from([(8, 7)]));
    for (wire, lane, source, target, resource) in [(256, 3, 6, 0, 11), (257, 4, 7, 2, 12)] {
        let mut delegated = original.clone();
        for row in &mut delegated[1..] {
            let frame = &mut row["message"];
            frame["frame_label"] = json!(wire);
            frame["lane"] = json!(lane);
            frame["source"] = json!(source);
            frame["target"] = json!(target);
            frame["payload"] = json!([resource, 1, wire & 255]);
        }
        assert_eq!(frames(&artifact, &delegated)?, BTreeMap::from([(8, 10)]));
        for (field, value) in [("frame_label", 65535), ("lane", 255)] {
            let mut changed = delegated.clone();
            for row in &mut changed[1..] {
                row["message"][field] = json!(value);
            }
            assert!(frames(&artifact, &changed).is_err());
        }
    }
    Ok(())
}

#[test]
fn actor_images_are_checked_from_exported_rows_without_a_second_owner_map() -> Result<()> {
    let original = cicd_artifact();
    assert_eq!(contracts(&original["compiled"])?.len(), 3);
    let mut changed = original.clone();
    changed["compiled"]["programs"][11]["rows"][0][2] = json!(11 << 8 | 11);
    assert!(contracts(&changed["compiled"]).is_err());
    changed = original.clone();
    changed["compiled"]["programs"][11]["rows"][0][3] = json!(256 << 16 | 3 << 8 | 99);
    assert!(contracts(&changed["compiled"]).is_err());
    changed = original.clone();
    changed["compiled"]["programs"][11]["rows"] = json!([]);
    assert!(contracts(&changed["compiled"]).is_err());
    changed = original.clone();
    changed["compiled"]["programs"]
        .as_array_mut()
        .unwrap()
        .pop();
    assert!(contracts(&changed["compiled"]).is_err());
    Ok(())
}
