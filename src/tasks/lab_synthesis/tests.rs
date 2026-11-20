//! Production resource receivers reject forged material before invoking an effect.
use super::arguments::InstrumentRequest;
use super::{global::*, imp, localside};
use crate::shell::{
    ShellError, ShellResult,
    imp::{
        bus::Bus,
        service::{ReceivedEffects, SimulatorOperation},
    },
};
use core::cell::Cell;
use hibana::{
    g,
    runtime::{SessionKitStorage, ids::SessionId, program::project},
};
use serde_json::{Value, json};
struct Effects(Cell<Vec<Value>>);
impl ReceivedEffects for Effects {
    fn apply<'io, P: SimulatorOperation>(
        &'io self,
        received: hibana::runtime::wire::ReceivedPayload<'_, P>,
    ) -> impl Future<Output = ShellResult<Value>> + use<'io, P> {
        let packet = imp::decode(received.as_bytes());
        async move {
            let mut calls = self.0.take();
            calls.push(packet?);
            self.0.set(calls);
            Err(ShellError::message(
                "unexpected physical effect in a refused request",
            ))
        }
    }
}
fn material((tool, actor, target): (&str, &str, &str), (id, inputs): (u64, Value)) -> Value {
    let key = match tool {
        "run_analysis" => "sample",
        "run_separation" => "material",
        _ => "compound",
    };
    json!({"tool_name":tool,"success":true,"data":{"scientist":actor,key:target,"artifact_id":id,
        "inputs":inputs,"status":"complete","missing_inputs":[],"cell_type":""}})
}
fn quality() -> Value {
    let assay = json!({
        "raw":material(("run_analysis","CHEMIST","raw_materials"),(0,json!({}))),
        "cells":material(("run_separation","BIOLOGIST","cell_cultures"),(1,json!({}))),
        "compound":material(("run_separation","CHEMIST","compound_purification"),(2,json!({"raw":0}))),
        "calibration":material(("run_analysis","BIOLOGIST","fluorescence"),(3,json!({"prepared_cells":1}))),
        "bioassay":material(("run_bioassay","BIOLOGIST","compound_purification"),(4,json!({"compound":2,"prepared_cells":1,"calibration":3})))
    });
    json!({"assay":assay,
        "molecular":material(("run_analysis","ANALYST","molecular_detail"),(5,json!({"compound":2,"bioassay":4}))),
        "verification":material(("run_separation","ANALYST","verification"),(6,json!({"compound":2,"bioassay":4})))})
}
#[hibana_runtime::main]
#[test]
async fn spectrometer_rejects_twelve_forged_argument_or_material_requests() -> ShellResult<()> {
    for bad in 0..12 {
        let source = g::seq(
            g::send::<INIT, SPEC_OWNER, SpecLoan>(),
            g::par(
                g::send::<CHEMIST, SPECTROMETER, RawOperation>(),
                g::par(
                    g::send::<BIOLOGIST, SPECTROMETER, CalibrationOperation>(),
                    g::par(
                        g::send::<ANALYST, SPECTROMETER, MolecularOperation>(),
                        g::send::<CHEMIST, SPECTROMETER, FinalOperation>(),
                    ),
                ),
            ),
        );
        let p_owner = project::<SPEC_OWNER>(&source);
        let p_init = project::<INIT>(&source);
        let p_spec = project::<SPECTROMETER>(&source);
        let p_chemist = project::<CHEMIST>(&source);
        let p_biologist = project::<BIOLOGIST>(&source);
        let p_analyst = project::<ANALYST>(&source);
        let session = SessionId::new(SESSION);
        let (bus, _, []) = Bus::open([], [(session, SPECTROMETER)]);
        let mut slab = vec![0; 128 * 1024];
        let mut storage = SessionKitStorage::uninit();
        let rv = storage.init().rendezvous(&mut slab, &bus.transport)?;
        let mut owner = rv.enter(session, &p_owner)?;
        let mut init = rv.enter(session, &p_init)?;
        let mut chemist = rv.enter(session, &p_chemist)?;
        let mut biologist = rv.enter(session, &p_biologist)?;
        let mut analyst = rv.enter(session, &p_analyst)?;
        let spec = rv.enter(session, &p_spec)?;
        let effects = Effects(Cell::new(Vec::new()));
        let packet = if bad < 4 {
            let args = match bad {
                0 => json!({"sample":"final_validation"}),
                1 => json!({"sample":7}),
                2 => json!({"material":"raw_materials"}),
                _ => json!({"sample":"raw_materials","next":true}),
            };
            json!({"arguments":args})
        } else if bad < 10 {
            let mut cells = material(
                ("run_separation", "BIOLOGIST", "cell_cultures"),
                (0, json!({})),
            );
            match bad {
                4 => cells["success"] = json!(false),
                5 => cells["data"]["scientist"] = json!("CHEMIST"),
                6 => cells["data"]["material"] = json!("other"),
                7 => cells["data"]["inputs"] = json!({"old":1}),
                8 => cells["data"]["missing_inputs"] = json!(["cells"]),
                _ => cells["data"]["artifact_id"] = Value::Null,
            }
            json!({"arguments":{"sample":"fluorescence"},"cells":cells})
        } else if bad == 10 {
            let q = quality();
            imp::quality(&q)?;
            json!({"arguments":{"sample":"final_validation"},"quality":q})
        } else {
            let q = quality();
            imp::assay(&q["assay"])?;
            json!({"arguments":{"sample":"molecular_detail"},"assay":q["assay"]})
        };
        let bytes = imp::encode(&packet)?;
        let error = hibana_runtime::race(
            hibana_runtime::race(localside::spectrometer::run(&mut owner, &effects), async {
                async {
                    init.send::<SpecLoan>(spec).await?;
                };
                async {
                    match bad {
                        0..4 => {
                            chemist
                                .send::<RawOperation>(&InstrumentRequest(&bytes))
                                .await?
                        }
                        4..10 => {
                            biologist
                                .send::<CalibrationOperation>(&InstrumentRequest(&bytes))
                                .await?
                        }
                        10 => {
                            chemist
                                .send::<FinalOperation>(&InstrumentRequest(&bytes))
                                .await?
                        }
                        _ => {
                            analyst
                                .send::<MolecularOperation>(&InstrumentRequest(&bytes))
                                .await?
                        }
                    }
                    Ok(())
                };
                core::future::pending::<ShellResult<()>>().await
            }),
            async {
                hibana_runtime::host::sleep(std::time::Duration::from_secs(3)).await?;
                Err(ShellError::message("spectrometer fixture deadline"))
            },
        )
        .await
        .expect_err("forged scientific request succeeded");
        let expected = match bad {
            0..4 => None,
            4..9 => Some("laboratory result does not match the actual completed scientific inputs"),
            9 => Some("missing actual scientific artifact"),
            10 => Some("final request substituted actual spectrometer results"),
            _ => Some("molecular request substituted actual spectrometer inputs"),
        };
        match (expected, error) {
            (None, ShellError::Endpoint(cause)) => {
                assert_eq!(
                    format!("{cause:?}"),
                    r#"EndpointError { operation: "recv", kind: Codec(Malformed) }"#,
                    "case {bad}"
                );
            }
            (Some(expected), ShellError::Message(actual)) => {
                assert_eq!(actual, expected, "case {bad}")
            }
            (_, error) => panic!("case {bad}: unexpected rejection: {error}"),
        }
        assert!(effects.0.take().is_empty(), "{bad}: actual physical effect");
    }
    Ok(())
}
#[hibana_runtime::main]
#[test]
async fn centrifuge_rejects_substituted_targets_and_fabricated_positive_assays() -> ShellResult<()>
{
    for bad in 0..3 {
        let source = g::seq(
            g::send::<INIT, CENT_OWNER, CentLoan>(),
            g::send::<ANALYST, CENTRIFUGE, VerificationOperation>(),
        );
        let p_owner = project::<CENT_OWNER>(&source);
        let p_init = project::<INIT>(&source);
        let p_cent = project::<CENTRIFUGE>(&source);
        let p_analyst = project::<ANALYST>(&source);
        let session = SessionId::new(SESSION);
        let (bus, _, []) = Bus::open([], [(session, CENTRIFUGE)]);
        let mut slab = vec![0; 128 * 1024];
        let mut storage = SessionKitStorage::uninit();
        let rv = storage.init().rendezvous(&mut slab, &bus.transport)?;
        let mut owner = rv.enter(session, &p_owner)?;
        let mut init = rv.enter(session, &p_init)?;
        let mut analyst = rv.enter(session, &p_analyst)?;
        let cent = rv.enter(session, &p_cent)?;
        let effects = Effects(Cell::new(Vec::new()));
        let mut packet =
            json!({"arguments":{"material":"verification"},"assay":quality()["assay"]});
        if bad == 0 {
            packet["arguments"]["material"] = json!("compound_purification");
        }
        if bad == 1 {
            packet["assay"]["bioassay"]["data"]["inputs"]["compound"] = json!(999);
        }
        if bad == 2 {
            imp::assay(&packet["assay"])?;
        }
        let bytes = imp::encode(&packet)?;
        let error = hibana_runtime::race(
            hibana_runtime::race(localside::centrifuge::run(&mut owner, &effects), async {
                async {
                    init.send::<CentLoan>(cent).await?;
                };
                async {
                    analyst
                        .send::<VerificationOperation>(&InstrumentRequest(&bytes))
                        .await?;
                };
                core::future::pending::<ShellResult<()>>().await
            }),
            async {
                hibana_runtime::host::sleep(std::time::Duration::from_secs(3)).await?;
                Err(ShellError::message("centrifuge fixture deadline"))
            },
        )
        .await
        .expect_err("forged verification succeeded");
        let expected = match bad {
            0 => None,
            1 => Some("laboratory result does not match the actual completed scientific inputs"),
            _ => Some("verification request substituted actual centrifuge results"),
        };
        match (expected, error) {
            (None, ShellError::Endpoint(cause)) => {
                assert_eq!(
                    format!("{cause:?}"),
                    r#"EndpointError { operation: "recv", kind: Codec(Malformed) }"#,
                    "case {bad}"
                );
            }
            (Some(expected), ShellError::Message(actual)) => {
                assert_eq!(actual, expected, "case {bad}")
            }
            (_, error) => panic!("case {bad}: unexpected rejection: {error}"),
        }
        assert!(effects.0.take().is_empty(), "{bad}: actual physical effect");
    }
    Ok(())
}
