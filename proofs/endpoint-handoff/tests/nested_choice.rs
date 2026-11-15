use hibana::{
    g,
    runtime::{
        SessionKitStorage,
        ids::SessionId,
        program::project,
        resolver::{DecisionArm, ResolverError, ResolverRef},
    },
};
#[path = "../../../src/imp/transport.rs"]
mod carrier;

fn left(_: &()) -> Result<DecisionArm, ResolverError> {
    Ok(DecisionArm::Left)
}

#[hibana_runtime::main]
#[test]
async fn preceding_visible_receive_is_not_skipped_for_nested_choice()
-> Result<(), std::convert::Infallible> {
    const C: u8 = 0;
    const D: u8 = 1;
    const A: u8 = 2;
    const B: u8 = 3;
    type Start = g::Msg<1, ()>;
    type AcceptA = g::Msg<2, ()>;
    type BeginB = g::Msg<3, ()>;
    type BeginDocB = g::Msg<4, ()>;
    type AcceptB = g::Msg<5, ()>;
    type BeginCombine = g::Msg<6, ()>;
    type ReviseA = g::Msg<7, ()>;
    type WaitB = g::Msg<8, ()>;
    type ReviseDocA = g::Msg<9, ()>;
    type ReviseB = g::Msg<10, ()>;
    type ReviseDocB = g::Msg<11, ()>;
    let second = g::seq(
        g::par(g::send::<C, B, BeginB>(), g::send::<C, D, BeginDocB>()),
        g::route(
            g::seq(g::send::<C, B, AcceptB>(), g::send::<C, D, BeginCombine>()),
            g::seq(g::send::<C, B, ReviseB>(), g::send::<C, D, ReviseDocB>()),
        )
        .resolve::<802>()
        .roll(),
    );
    let p = g::seq(
        g::send::<C, D, Start>(),
        g::route(
            g::seq(g::send::<C, A, AcceptA>(), second),
            g::seq(
                g::par(g::send::<C, A, ReviseA>(), g::send::<C, B, WaitB>()),
                g::send::<C, D, ReviseDocA>(),
            ),
        )
        .resolve::<801>()
        .roll(),
    );
    let pc = project::<C>(&p);
    let pd = project::<D>(&p);
    let wire = carrier::InMemoryTransport::new([]);
    let mut slab = vec![0; 65536];
    let mut storage = SessionKitStorage::uninit();
    let kit = storage.init();
    let rv = kit.rendezvous(&mut slab, &wire).unwrap();
    rv.set_resolver(&pc, ResolverRef::<801>::decision_state(&(), left))
        .unwrap();
    rv.set_resolver(&pc, ResolverRef::<802>::decision_state(&(), left))
        .unwrap();
    let mut c = rv.enter(SessionId::new(49), &pc).unwrap();
    let mut d = rv.enter(SessionId::new(49), &pd).unwrap();
    c.send::<Start>(&()).await.unwrap();
    c.send::<AcceptA>(&()).await.unwrap();
    c.send::<BeginB>(&()).await.unwrap();
    c.send::<BeginDocB>(&()).await.unwrap();
    c.send::<AcceptB>(&()).await.unwrap();
    c.send::<BeginCombine>(&()).await.unwrap();
    d.recv::<Start>().await.unwrap();
    let branch = d.offer().await.unwrap();
    let label = branch.label();
    eprintln!(
        "nested first label: {label}, frames={}",
        serde_json::json!(wire.take_observations().collect::<Vec<_>>())
    );
    assert_eq!(
        label, 4,
        "a branch cannot skip its preceding visible receive"
    );
    branch.recv::<BeginDocB>().await.unwrap();
    d.offer()
        .await
        .unwrap()
        .recv::<BeginCombine>()
        .await
        .unwrap();
    Ok(())
}

#[hibana_runtime::main]
#[test]
async fn later_choice_preview_cannot_commit_before_its_predecessor()
-> Result<(), std::convert::Infallible> {
    const C: u8 = 0;
    const D: u8 = 1;
    const A: u8 = 2;
    const B: u8 = 3;
    type Start = g::Msg<1, ()>;
    type AcceptA = g::Msg<2, ()>;
    type BeginB = g::Msg<3, ()>;
    type BeginDocB = g::Msg<4, ()>;
    type AcceptB = g::Msg<5, ()>;
    type BeginCombine = g::Msg<6, ()>;
    type ReviseA = g::Msg<7, ()>;
    type WaitB = g::Msg<8, ()>;
    type ReviseDocA = g::Msg<9, ()>;
    type ReviseB = g::Msg<10, ()>;
    type ReviseDocB = g::Msg<11, ()>;
    let second = g::seq(
        g::par(g::send::<C, B, BeginB>(), g::send::<C, D, BeginDocB>()),
        g::route(
            g::seq(g::send::<C, B, AcceptB>(), g::send::<C, D, BeginCombine>()),
            g::seq(g::send::<C, B, ReviseB>(), g::send::<C, D, ReviseDocB>()),
        )
        .resolve::<802>()
        .roll(),
    );
    let p = g::seq(
        g::send::<C, D, Start>(),
        g::route(
            g::seq(g::send::<C, A, AcceptA>(), second),
            g::seq(
                g::par(g::send::<C, A, ReviseA>(), g::send::<C, B, WaitB>()),
                g::send::<C, D, ReviseDocA>(),
            ),
        )
        .resolve::<801>()
        .roll(),
    );
    let pc = project::<C>(&p);
    let pd = project::<D>(&p);
    let wire = carrier::InMemoryTransport::new([]);
    let mut slab = vec![0; 65536];
    let mut storage = SessionKitStorage::uninit();
    let kit = storage.init();
    let rv = kit.rendezvous(&mut slab, &wire).unwrap();
    rv.set_resolver(&pc, ResolverRef::<801>::decision_state(&(), left))
        .unwrap();
    rv.set_resolver(&pc, ResolverRef::<802>::decision_state(&(), left))
        .unwrap();
    let mut c = rv.enter(SessionId::new(49), &pc).unwrap();
    let mut d = rv.enter(SessionId::new(49), &pd).unwrap();
    c.send::<Start>(&()).await.unwrap();
    c.send::<AcceptA>(&()).await.unwrap();
    c.send::<BeginB>(&()).await.unwrap();
    c.send::<BeginDocB>(&()).await.unwrap();
    c.send::<AcceptB>(&()).await.unwrap();
    c.send::<BeginCombine>(&()).await.unwrap();
    d.recv::<Start>().await.unwrap();
    let branch = d.offer().await.unwrap();
    let label = branch.label();
    eprintln!(
        "nested first label: {label}, frames={}",
        serde_json::json!(wire.take_observations().collect::<Vec<_>>())
    );
    if label == 6 {
        let result = branch.recv::<BeginCombine>().await;
        eprintln!("nested out-of-order consume: {result:?}");
        assert!(
            result.is_err(),
            "sequential predecessor must be consumed first"
        );
    } else {
        assert_eq!(label, 4);
    }
    Ok(())
}
