use super::{global::*, publication};
use crate::{
    imp::json::encode,
    shell::{
        ShellError, ShellResult,
        global::{Exit, Loan},
        imp::{
            bus::{Bus, Terminal},
            private_fs::PrivateFs,
            review_reply::ReviewReply,
            service::Domain,
        },
        localside::file_metadata,
    },
};
use hibana::{
    Endpoint, EndpointBorrow, g,
    runtime::{SessionKitStorage, ids::SessionId, program::project},
};
use hibana_wasip1_runtime::protocol;
use serde_json::{Value, json};
use std::{cell::Cell, path::Path, process::Command, time::Duration};

const INITIAL: u8 = 20;

fn review(chapter: &str) -> Value {
    json!({
        "tool_name":"peer_review","success":true,
        "data":{
            "scope":chapter,"agent":"REVIEWER","review_id":format!("review:{chapter}"),
            "input_snapshot":{
                "sections":{chapter:{"id":format!("draft:{chapter}"),"content":"current text"}},
                "references":{chapter:{"id":format!("refs:{chapter}"),"content":"citations"}},
                "figures":{chapter:{"id":format!("figure:{chapter}"),"content":"actual plot"}}
            }
        }
    })
}

struct Effects {
    calls: Cell<Vec<&'static str>>,
    snapshot: Value,
    reviews: Value,
    stale_editorial: bool,
}

impl Domain for Effects {
    async fn execute(
        &self,
        agent: &'static str,
        tool: &'static str,
        arguments: Value,
    ) -> Result<Value, String> {
        let mut calls = self.calls.take();
        calls.push(tool);
        self.calls.set(calls);
        assert_eq!(agent, "EDITOR_IN_CHIEF");
        let mut snapshot = self.snapshot.clone();
        let data = match tool {
            "make_editorial_decision" => {
                if self.stale_editorial {
                    snapshot["sections"]["section_b"]["content"] = json!("older draft");
                }
                json!({
                    "agent":agent,"decision_type":"finalize","decision_id":"editorial:1",
                    "review_summary":arguments["review_summary"],"input_snapshot":snapshot,
                    "review_receipts":self.reviews
                })
            }
            "combine_sections" => json!({
                "agent":agent,"artifact_id":"combined:2",
                "sections":["section_d","section_b","section_a","section_c"],
                "input_snapshot":snapshot,"review_receipts":self.reviews,
                "editorial_receipts":{"finalize":"editorial:1"}
            }),
            _ => panic!("unexpected effect {tool}"),
        };
        Ok(json!({"tool_name":tool,"success":true,"data":data}))
    }
}

async fn deliver_review<'r, const SECTION: u8>(
    mut section: Endpoint<'r, SECTION>,
    reviewer: &mut EndpointBorrow<'_, 'r, REVIEWER>,
    value: &Value,
) -> ShellResult<()> {
    let bytes = encode(value)?;
    section
        .send::<Reviewed<SECTION>>(&ReviewReply::from_bytes(&bytes)?)
        .await?;
    reviewer.send::<Accepted<SECTION>>(section).await?;
    Ok(())
}

pub(super) fn editor_fixture() -> ShellResult<Vec<u8>> {
    let guest_source = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(file!())
        .with_file_name("editor_fixture.rs");
    let guest_file = std::env::temp_dir().join(format!(
        "choreofix-editor-fixture-{}-{}.wasm",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos(),
    ));
    let compiled = Command::new("rustc")
        .args([
            "+stable",
            "--edition=2024",
            "--target=wasm32-wasip1",
            "-Copt-level=s",
            "-Cpanic=abort",
            "-Cstrip=debuginfo",
        ])
        .arg(guest_source)
        .arg("-o")
        .arg(&guest_file)
        .status()?;
    if !compiled.success() {
        return Err(ShellError::message(
            "survey editor WASI fixture did not compile",
        ));
    }
    let module = std::fs::read(&guest_file);
    std::fs::remove_file(guest_file)?;
    Ok(module?)
}

pub(super) async fn fixture_terminal<'r>(
    io: &mut Endpoint<'r, EDITOR_IO>,
    bus: &Bus,
    terminal: Terminal,
) -> ShellResult<()> {
    let data = PrivateFs::new(terminal, bus, &EDITOR_OBJECTS);
    loop {
        let branch = io.offer().await?;
        match branch.label() {
            protocol::LABEL_WASI_FD_FDSTAT_GET => {
                let request = branch.recv::<protocol::FdFdstatGetReqMsg>().await?;
                let reply = data.objects().choreofs().fd_fdstat_get(request);
                io.send::<protocol::FdFdstatGetRetMsg>(&reply).await?;
            }
            protocol::LABEL_WASI_FD_PRESTAT_GET => {
                let fd = branch.recv::<protocol::FdPrestatGetReqMsg>().await?.0.fd();
                let reply = protocol::FdPrestatRet(protocol::FdPrestat::new_with_errno(
                    fd,
                    1,
                    if fd == 3 { 0 } else { 8 },
                ));
                io.send::<protocol::FdPrestatGetRetMsg>(&reply).await?;
            }
            protocol::LABEL_WASI_FD_PRESTAT_DIR_NAME => {
                let fd = branch
                    .recv::<protocol::FdPrestatDirNameReqMsg>()
                    .await?
                    .0
                    .fd();
                let reply = protocol::FdPrestatDirNameRet(protocol::FdPrestatDirNameDone::new(
                    fd,
                    b"/",
                    if fd == 3 { 0 } else { 8 },
                )?);
                io.send::<protocol::FdPrestatDirNameRetMsg>(&reply).await?;
            }
            240 => {
                assert_eq!(branch.recv::<Exit>().await?, 0);
                return Ok(());
            }
            label => panic!("unexpected survey editor fixture I/O {label}"),
        }
    }
}

#[hibana_runtime::main]
#[test]
async fn publication_consumes_the_four_affine_chapters_and_actual_editorial_result()
-> ShellResult<()> {
    let module = editor_fixture()?;
    let source = g::seq(
        g::par(
            g::send::<INITIAL, REVIEWER, Accepted<SECTION_A>>(),
            g::par(
                g::send::<INITIAL, REVIEWER, Accepted<SECTION_B>>(),
                g::par(
                    g::send::<INITIAL, REVIEWER, Accepted<SECTION_C>>(),
                    g::send::<INITIAL, REVIEWER, Accepted<SECTION_D>>(),
                ),
            ),
        ),
        wasi_publication(g::par(
            approved_chapter::<SECTION_A>(),
            g::par(
                approved_chapter::<SECTION_B>(),
                g::par(
                    approved_chapter::<SECTION_C>(),
                    approved_chapter::<SECTION_D>(),
                ),
            ),
        )),
    );
    let init_program = project::<INITIAL>(&source);
    let reviewer_program = project::<REVIEWER>(&source);
    let editor_program = project::<EDITOR>(&source);
    let file_program = project::<EDITOR_FILE>(&source);
    let data_program = project::<EDITOR_DATA>(&source);
    let root_program = project::<EDITOR_VM>(&source);
    let io_program = project::<EDITOR_IO>(&source);
    let a_program = project::<SECTION_A>(&source);
    let b_program = project::<SECTION_B>(&source);
    let c_program = project::<SECTION_C>(&source);
    let d_program = project::<SECTION_D>(&source);

    for (wrong_chapter, stale_editorial) in [(false, false), (false, true), (true, false)] {
        let mut reviews = publication::SECTIONS.map(review);
        if wrong_chapter {
            reviews[1]["data"]["scope"] = json!("section_a");
        }
        let mut snapshots = json!({"sections":{}, "references":{}, "figures":{}});
        let mut review_ids = json!({});
        for (name, review) in publication::SECTIONS.into_iter().zip(&reviews) {
            review_ids[name] = review["data"]["review_id"].clone();
            for category in ["sections", "references", "figures"] {
                snapshots[category][name] =
                    review["data"]["input_snapshot"][category][name].clone();
            }
        }
        let effects = Effects {
            calls: Cell::new(Vec::new()),
            snapshot: snapshots,
            reviews: review_ids,
            stale_editorial,
        };

        let session = SessionId::new(301);
        let (bus, _, [terminal]) = Bus::open(
            ["EDITOR_IN_CHIEF"],
            [SECTION_A, SECTION_B, SECTION_C, SECTION_D, EDITOR_FILE].map(|role| (session, role)),
        );
        let mut slab = vec![0; 512 * 1024];
        let mut storage = SessionKitStorage::uninit();
        let rv = storage.init().rendezvous(&mut slab, &bus.transport)?;
        let mut initial = rv.enter(session, &init_program)?;
        let mut reviewer = rv.enter(session, &reviewer_program)?;
        let mut editor = rv.enter(session, &editor_program)?;
        let mut data = rv.enter(session, &data_program)?;
        let file = rv.enter(session, &file_program)?;
        let root = rv.enter(session, &root_program)?;
        let mut io = rv.enter(session, &io_program)?;
        let a = rv.enter(session, &a_program)?;
        let b = rv.enter(session, &b_program)?;
        let c = rv.enter(session, &c_program)?;
        let d = rv.enter(session, &d_program)?;

        let run = async {
            async {
                let [mut a_lane, mut b_lane, mut c_lane, mut d_lane] = initial.par()?;
                async {
                    a_lane.send::<Accepted<SECTION_A>>(a).await?;
                };
                async {
                    b_lane.send::<Accepted<SECTION_B>>(b).await?;
                };
                async {
                    c_lane.send::<Accepted<SECTION_C>>(c).await?;
                };
                async {
                    d_lane.send::<Accepted<SECTION_D>>(d).await?;
                };
                Ok(())
            };
            async {
                let (a, b, c, d) = async {
                    let [mut a, mut b, mut c, mut d] = reviewer.par()?;
                    async { a.recv::<Accepted<SECTION_A>>().await };
                    async { b.recv::<Accepted<SECTION_B>>().await };
                    async { c.recv::<Accepted<SECTION_C>>().await };
                    async { d.recv::<Accepted<SECTION_D>>().await };
                }
                .await?;
                let [mut a_lane, mut b_lane, mut c_lane, mut d_lane] = reviewer.par()?;
                async {
                    deliver_review(a, &mut a_lane, &reviews[0]).await?;
                };
                async {
                    deliver_review(b, &mut b_lane, &reviews[1]).await?;
                };
                async {
                    deliver_review(c, &mut c_lane, &reviews[2]).await?;
                };
                async {
                    deliver_review(d, &mut d_lane, &reviews[3]).await?;
                };
                Ok(())
            };
            async {
                file_metadata::run(&mut data, file, EDITOR_OBJECTS).await?;
            };
            async {
                let file = editor.recv::<Loan<EDITOR_FILE>>().await?;
                async {
                    let result = publication::run(&mut editor, &effects).await?;
                    assert_eq!(result["data"]["input_snapshot"], effects.snapshot);
                };
                async {
                    super::editor_guest(&module, file, root).await?;
                };
                Ok(())
            };
            async {
                fixture_terminal(&mut io, &bus, terminal).await?;
            };
            Ok::<(), ShellError>(())
        };
        let result =
            hibana_runtime::race(run, hibana_runtime::host::deadline(Duration::from_secs(3))).await;
        let calls = effects.calls.take();
        if wrong_chapter {
            assert!(
                result
                    .unwrap_err()
                    .to_string()
                    .contains("invalid peer review")
            );
            assert!(calls.is_empty());
        } else if stale_editorial {
            assert!(
                result
                    .unwrap_err()
                    .to_string()
                    .contains("actual admitted data")
            );
            assert_eq!(calls, ["make_editorial_decision"]);
        } else {
            result?;
            assert_eq!(calls, ["make_editorial_decision", "combine_sections"]);
        }
    }
    Ok(())
}
