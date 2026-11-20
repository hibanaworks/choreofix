use super::{Arguments, CHAPTERS, global::*, localside::drafts};
use crate::{
    imp::json::decode,
    shell::{
        ShellResult,
        global::Loan,
        imp::{
            bus::Bus,
            service::{ReceivedEffects, SimulatorOperation},
        },
    },
};
use hibana::runtime::{SessionKitStorage, ids::SessionId, program::project};
use serde_json::{Value, json};
use std::{cell::Cell, time::Duration};

#[derive(Default)]
struct World(Cell<Vec<Value>>);

impl ReceivedEffects for World {
    fn apply<'io, P: SimulatorOperation>(
        &'io self,
        received: hibana::runtime::wire::ReceivedPayload<'_, P>,
    ) -> impl Future<Output = ShellResult<Value>> + use<'io, P> {
        let args = crate::imp::json::decode(received.as_bytes());
        async move {
            let args = args?;
            let (agent, tool) = P::RPC;

            assert_eq!(tool, "write_section");
            let mut writes = self.0.take();
            let result = json!({"success":true,"tool_name":tool,"data":{
                "agent":agent,"section":args["section_name"], "artifact":{
                    "id":format!("draft:{}", writes.len() + 1), "content":args["content_notes"]
                }
            }});
            writes.push(result.clone());
            self.0.set(writes);
            Ok(result)
        }
    }
}

#[hibana_runtime::main]
#[test]
async fn both_writer_endpoints_can_admit_all_six_orders_on_one_document() -> ShellResult<()> {
    let source = initial_drafts();
    let document_program = project::<DOCUMENT>(&source);
    let drafting_program = project::<DRAFTING>(&source);
    let writer_a_data = project::<WRITER_A_DATA>(&source);
    let writer_b_data = project::<WRITER_B_DATA>(&source);
    let file_a_program = project::<WRITER_FILE>(&source);
    let file_b_program = project::<WRITER_B_FILE>(&source);
    for (trial, order) in [
        [0, 1, 2],
        [0, 2, 1],
        [1, 0, 2],
        [1, 2, 0],
        [2, 0, 1],
        [2, 1, 0],
    ]
    .into_iter()
    .enumerate()
    {
        let session = SessionId::new(77 + trial as u32);
        let (bus, _, []) = Bus::open([], [WRITER_FILE, WRITER_B_FILE].map(|role| (session, role)));
        let mut slab = vec![0; 128 * 1024];
        let mut storage = SessionKitStorage::uninit();
        let rv = storage.init().rendezvous(&mut slab, &bus.transport)?;
        let mut document = rv.enter(session, &document_program)?;
        let mut drafting = rv.enter(session, &drafting_program)?;
        let mut a_data = rv.enter(session, &writer_a_data)?;
        let mut b_data = rv.enter(session, &writer_b_data)?;
        let a_file = rv.enter(session, &file_a_program)?;
        let b_file = rv.enter(session, &file_b_program)?;
        let world = World::default();
        let trial = async {
            async {
                a_data.send::<Loan<WRITER_FILE>>(a_file).await?;
            };
            async {
                b_data.send::<Loan<WRITER_B_FILE>>(b_file).await?;
            };
            async {
                let (mut a, mut b) = async {
                    let [mut a, mut b] = drafting.par()?;
                    async { a.recv::<Loan<WRITER_FILE>>().await };
                    async { b.recv::<Loan<WRITER_B_FILE>>().await };
                }
                .await?;
                for slot in order {
                    let name = CHAPTERS[slot];
                    let arguments = json!({"section_name":name,"content_notes":format!("Draft by its author for {name}")});
                    let actual = match slot {
                        0 => {
                            a.send::<DraftRequest<DRAFT_1>>(&Arguments::new(&arguments)?)
                                .await?;
                            decode(a.recv::<DraftWritten<DRAFT_1>>().await?)?
                        }
                        1 => {
                            a.send::<DraftRequest<DRAFT_2>>(&Arguments::new(&arguments)?)
                                .await?;
                            decode(a.recv::<DraftWritten<DRAFT_2>>().await?)?
                        }
                        2 => {
                            b.send::<DraftRequest<DRAFT_3>>(&Arguments::new(&arguments)?)
                                .await?;
                            decode(b.recv::<DraftWritten<DRAFT_3>>().await?)?
                        }
                        _ => unreachable!(),
                    };
                    assert_eq!(actual["data"]["section"], name);
                    assert_eq!(
                        actual["data"]["artifact"]["content"],
                        arguments["content_notes"]
                    );
                }
            };
            async {
                let actual = drafts::initial(&mut document, &world).await?;
                assert_eq!(actual.as_object().unwrap().len(), 3);
            };
            Ok(())
        };
        hibana_runtime::race(
            trial,
            hibana_runtime::host::deadline(Duration::from_secs(3)),
        )
        .await?;
        let actual = world.0.take();
        assert_eq!(actual.len(), 3);
        for (event, slot) in actual.iter().zip(order) {
            assert_eq!(event["data"]["section"], CHAPTERS[slot]);
            assert_eq!(
                event["data"]["agent"],
                if slot == 2 { "WRITER_B" } else { "WRITER_A" }
            );
        }
    }
    Ok(())
}
