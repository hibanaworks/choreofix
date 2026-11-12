//! Compile the single production 10M source without unrelated task globals.
#[path = "../../../src/imp/projection.rs"]
pub(crate) mod projection;
#[path = "../../../src/shell/mod.rs"]
pub mod shell;
#[path = "../../../src/imp/transport.rs"]
pub(crate) mod transport;
pub(crate) mod imp {
    pub(crate) use crate::{projection, transport};
}
#[path = "../../../src/tasks/cascading_build/mod.rs"]
pub mod cascading_build;

#[test]
fn the_whole_cascading_source_is_well_formed_and_projects_all_views() {
    assert_eq!(
        hibana::g::diagnose(&cascading_build::global::source()),
        None
    );
    let p = cascading_build::global::programs();
    use hibana::runtime::tap;
    fn check<const R: u8>(program: &hibana::runtime::program::RoleProgram<R>) -> usize {
        tap::projected(program)
            .filter(|r| r.id() == tap::PROJECTED_SOURCE)
            .count()
    }
    let sizes = [
        check(&p.a),
        check(&p.b),
        check(&p.c),
        check(&p.integrator),
        check(&p.tester),
        check(&p.compiler),
        check(&p.testing),
        check(&p.config),
        check(&p.core),
        check(&p.models),
        check(&p.a_io),
        check(&p.b_io),
        check(&p.c_io),
        check(&p.integrator_io),
        check(&p.tester_io),
    ];
    assert!(sizes[0] > 0);
    assert!(sizes.iter().all(|size| *size == sizes[0]));
    println!("source_events_per_view={}", sizes[0]);
    assert_eq!(p.observation()["programs"].as_array().unwrap().len(), 15);
}
