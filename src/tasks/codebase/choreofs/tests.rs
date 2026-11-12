//! Real projected operations; shell and original simulator checks run separately.
use super::{global::*, input::assignment, localside, wire::*};
use crate::shell::{
    ShellResult,
    global::Exit,
    imp::{bus::Bus, service::Domain},
};
use hibana::{
    Endpoint,
    runtime::{SessionKitStorage, ids::SessionId},
};
use hibana_wasip1_runtime::protocol;
use std::{cell::Cell, future::poll_fn, task::Poll};

#[derive(Default)]
struct Effects {
    calls: Cell<Vec<(&'static str, &'static str)>>,
    active: Cell<Vec<Vec<String>>>,
    peak: Cell<usize>,
    test_results: &'static [bool],
    concurrent_agents: &'static [&'static str],
}

impl Domain for Effects {
    async fn execute(
        &self,
        agent: &'static str,
        tool: &'static str,
        arguments: serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        let mut calls = self.calls.take();
        calls.push((agent, tool));
        let visit = calls
            .iter()
            .filter(|entry| **entry == (agent, tool))
            .count();
        self.calls.set(calls);
        poll_fn(|cx| {
            cx.waker().wake_by_ref();
            Poll::Ready(())
        })
        .await;
        if tool == "commit_changes" {
            let modules: Vec<_> = arguments["modules"]
                .as_str()
                .unwrap()
                .split(',')
                .map(str::to_owned)
                .collect();
            let mut active = self.active.take();
            assert!(
                active
                    .iter()
                    .all(|other| other.iter().all(|name| !modules.contains(name))),
                "concurrent commits used the same module"
            );
            active.push(modules.clone());
            self.peak.set(self.peak.get().max(active.len()));
            self.active.set(active);
            if self.concurrent_agents.contains(&agent) && visit == 1 {
                poll_fn(|_| {
                    let calls = self.calls.take();
                    let ready = self
                        .concurrent_agents
                        .iter()
                        .all(|who| calls.contains(&(*who, "commit_changes")));
                    self.calls.set(calls);
                    if ready {
                        Poll::Ready(())
                    } else {
                        Poll::Pending
                    }
                })
                .await;
            }
            if agent == "DEVELOPER_B" && visit == 1 && !self.test_results.is_empty() {
                poll_fn(|_| {
                    let calls = self.calls.take();
                    let ready = calls.contains(&("DEVELOPER_A", "run_local_tests"));
                    self.calls.set(calls);
                    if ready {
                        Poll::Ready(())
                    } else {
                        Poll::Pending
                    }
                })
                .await;
            }
            hibana_runtime::yield_now().await;
            let mut active = self.active.take();
            active.retain(|pair| *pair != modules);
            self.active.set(active);
        }
        if agent == "DEVELOPER_A"
            && tool == "run_local_tests"
            && visit == 1
            && !self.test_results.is_empty()
        {
            poll_fn(|_| {
                let calls = self.calls.take();
                let ready = calls.contains(&("DEVELOPER_B", "commit_changes"));
                self.calls.set(calls);
                if ready {
                    Poll::Ready(())
                } else {
                    Poll::Pending
                }
            })
            .await;
        }
        let success = if tool != "run_local_tests" {
            true
        } else if self.test_results.is_empty() {
            false
        } else {
            *self
                .test_results
                .get(visit - 1)
                .expect("unexpected test after the supplied results")
        };
        Ok(serde_json::json!({"success":success, "agent":agent,"tool":tool}))
    }
}

#[hibana_runtime::scope]
async fn client<
    const TASK: u8,
    const SUBJECT: u8,
    const ROLE: u8,
    const FIRST: u8,
    const SECOND: u8,
>(
    endpoint: &mut Endpoint<'_, ROLE>,
) -> ShellResult<()> {
    let [mut task, mut private] = endpoint.par()?;
    let input = assignment::<TASK, SUBJECT>();
    let close = protocol::FdCloseReq(protocol::FdRequest::new(SERVICE_FD));
    let design = Close::<TASK, SUBJECT, DESIGN>::new(
        close,
        serde_json::json!({"feature_name":input.feature}),
    )?;
    task.send::<DesignRequest<TASK, SUBJECT>>(&design).await?;
    let bytes = task.recv::<DesignCompleted>().await?.to_vec();
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&bytes).unwrap()["tool"],
        "design_feature"
    );
    let implementation = Close::<TASK, SUBJECT, IMPLEMENT>::new(
        close,
        serde_json::json!({"feature_name":input.feature}),
    )?;
    task.send::<ImplementRequest<TASK, SUBJECT>>(&implementation)
        .await?;
    let bytes = task.recv::<ImplementCompleted>().await?.to_vec();
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&bytes).unwrap()["tool"],
        "implement_code"
    );
    let commit = Close::<TASK, SUBJECT, COMMIT>::new(
        close,
        serde_json::json!({"modules":input.modules.join(",")}),
    )?;
    let (mut first, mut second) = {
        let [mut first_loan, mut second_loan] = task.par()?;
        async {
            async {
                let mut resource = first_loan.recv::<Loan<FIRST>>().await?;
                resource
                    .send::<CommitRequest<TASK, SUBJECT, FIRST>>(&commit)
                    .await?;
                Ok(resource)
            };
            async {
                let mut resource = second_loan.recv::<Loan<SECOND>>().await?;
                resource
                    .send::<CommitRequest<TASK, SUBJECT, SECOND>>(&commit)
                    .await?;
                Ok(resource)
            };
        }
        .await?
    };
    let (first_result, second_result) = async {
        async { Ok(first.recv::<CommitCompleted<FIRST>>().await?.to_vec()) };
        async { Ok(second.recv::<CommitCompleted<SECOND>>().await?.to_vec()) };
    }
    .await?;
    assert_eq!(first_result, second_result);
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&first_result).unwrap()["tool"],
        "commit_changes"
    );
    {
        let [mut first_return, mut second_return] = task.par()?;
        async {
            async {
                first_return.send::<Loan<FIRST>>(first).await?;
            };
            async {
                second_return.send::<Loan<SECOND>>(second).await?;
            };
            Ok(())
        }
        .await?;
    }
    let test = Close::<TASK, SUBJECT, TEST>::new(
        close,
        serde_json::json!({"feature_name":input.feature}),
    )?;
    task.send::<TestRequest<TASK, SUBJECT>>(&test).await?;
    let bytes = task.recv::<TestCompleted>().await?.to_vec();
    let result: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(result["tool"], "run_local_tests");
    assert_eq!(result["success"], false, "terminal test failure was hidden");
    task.send::<ServiceExit>(&0).await?;
    private.send::<Exit>(&0).await?;
    Ok(())
}

// Attach fixture roles and declare their production participants.
macro_rules! fixture {
    ($test:ident, $project:ident, $roles:ident, $task:ident, $peak:expr;
     agents [$(($service:ident, $subject:ident, $vm:ident, $vm_role:ident, $io:ident, $io_role:ident, $first:ident, $second:ident, $name:literal)),+];
     resources [$(($owner:ident, $owner_role:ident, $resource:ident, $resource_role:ident)),+]) => {
        #[hibana_runtime::main]
        #[test]
        async fn $test() -> ShellResult<()> {
            use super::global::$roles::*;
            let programs = $project();
            let session = SessionId::new(20);
            let (bus, _reader, _terminals) = Bus::open([$($name),+], [$( (session, $resource_role) ),+]);
            let mut slab = vec![0; 2 * 1024 * 1024];
            let mut storage = SessionKitStorage::uninit();
            let runtime = storage.init().rendezvous(&mut slab, &bus.transport)?;
            $(let mut $service = runtime.enter(session, &programs.$service)?;
              let mut $vm = runtime.enter(session, &programs.$vm)?;
              let mut $io = runtime.enter(session, &programs.$io)?;)+
            $(let mut $owner = runtime.enter(session, &programs.$owner)?;
              let $resource = runtime.enter(session, &programs.$resource)?;)+
            let effects = Effects {
                concurrent_agents: if $peak == 2 { &["DEVELOPER_A", "DEVELOPER_C"] } else { &[] },
                ..Effects::default()
            };
            hibana_runtime::race(async {
                $(
                    async { localside::developer::run::<$task, $subject, $first, $second>(&mut $service, &effects).await?; };
                    async { client::<$task, $subject, $vm_role, $first, $second>(&mut $vm).await?; };
                    async { assert_eq!($io.offer().await?.recv::<Exit>().await?, 0); };
                )+
                $(
                    async { localside::module::run::<$owner_role, $resource_role>(&mut $owner, $resource).await?; };
                )+
                Ok(())
            }, hibana_runtime::host::deadline(std::time::Duration::from_secs(5))).await?;
            let calls = effects.calls.take();
            $(for tool in ["design_feature", "implement_code", "commit_changes", "run_local_tests"] {
                assert_eq!(calls.iter().filter(|entry| **entry == ($name, tool)).count(), 1);
            })+
            assert_eq!(effects.peak.get(), $peak);
            assert!(effects.active.take().is_empty());
            Ok(())
        }
    };
}

fixture!(three_guests_use_and_return_real_module_endpoints, project_1e, roles1e, EASY, 1;
    agents [(a,A,a_wasi,A_WASI,a_io,A_IO,AUTH_RESOURCE,DATABASE_RESOURCE,"DEVELOPER_A"),
            (b,B,b_wasi,B_WASI,b_io,B_IO,API_RESOURCE,DATABASE_RESOURCE,"DEVELOPER_B"),
            (c,C,c_wasi,C_WASI,c_io,C_IO,API_RESOURCE,AUTH_RESOURCE,"DEVELOPER_C")];
    resources [(auth,AUTH,auth_resource,AUTH_RESOURCE),
               (database,DATABASE,database_resource,DATABASE_RESOURCE),
               (api,API,api_resource,API_RESOURCE)]);

fixture!(five_guests_commit_disjoint_modules_concurrently, project_1m, roles1m, MEDIUM, 2;
    agents [(a,A,a_wasi,A_WASI,a_io,A_IO,AUTH_RESOURCE,DATABASE_RESOURCE,"DEVELOPER_A"),
            (b,B,b_wasi,B_WASI,b_io,B_IO,API_RESOURCE,DATABASE_RESOURCE,"DEVELOPER_B"),
            (c,C,c_wasi,C_WASI,c_io,C_IO,API_RESOURCE,PAYMENT_RESOURCE,"DEVELOPER_C"),
            (d,D,d_wasi,D_WASI,d_io,D_IO,NOTIFY_RESOURCE,PAYMENT_RESOURCE,"DEVELOPER_D"),
            (e,E,e_wasi,E_WASI,e_io,E_IO,AUTH_RESOURCE,NOTIFY_RESOURCE,"DEVELOPER_E")];
    resources [(auth,AUTH,auth_resource,AUTH_RESOURCE),
               (database,DATABASE,database_resource,DATABASE_RESOURCE),
               (api,API,api_resource,API_RESOURCE),
               (payment,PAYMENT,payment_resource,PAYMENT_RESOURCE),
               (notify,NOTIFY,notify_resource,NOTIFY_RESOURCE)]);
#[hibana_runtime::scope]
async fn revising_client<
    const TASK: u8,
    const SUBJECT: u8,
    const ROLE: u8,
    const FIRST: u8,
    const SECOND: u8,
>(
    endpoint: &mut Endpoint<'_, ROLE>,
) -> ShellResult<()> {
    let [mut task, mut private] = endpoint.par()?;
    let input = assignment::<TASK, SUBJECT>();
    let close = protocol::FdCloseReq(protocol::FdRequest::new(SERVICE_FD));
    let design = Close::<TASK, SUBJECT, DESIGN>::new(
        close,
        serde_json::json!({"feature_name":input.feature}),
    )?;
    task.send::<DesignRequest<TASK, SUBJECT>>(&design).await?;
    let bytes = task.recv::<DesignCompleted>().await?.to_vec();
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&bytes).unwrap()["tool"],
        "design_feature"
    );
    'revisions: loop {
        let implementation = Close::<TASK, SUBJECT, IMPLEMENT>::new(
            close,
            serde_json::json!({"feature_name":input.feature}),
        )?;
        task.send::<ImplementRequest<TASK, SUBJECT>>(&implementation)
            .await?;
        let bytes = task.recv::<ImplementCompleted>().await?.to_vec();
        assert_eq!(
            serde_json::from_slice::<serde_json::Value>(&bytes).unwrap()["tool"],
            "implement_code"
        );
        let commit = Close::<TASK, SUBJECT, COMMIT>::new(
            close,
            serde_json::json!({"modules":input.modules.join(",")}),
        )?;
        let (mut first, mut second) = {
            let [mut first_loan, mut second_loan] = task.par()?;
            async {
                async {
                    let mut resource = first_loan.recv::<Loan<FIRST>>().await?;
                    resource
                        .send::<CommitRequest<TASK, SUBJECT, FIRST>>(&commit)
                        .await?;
                    Ok(resource)
                };
                async {
                    let mut resource = second_loan.recv::<Loan<SECOND>>().await?;
                    resource
                        .send::<CommitRequest<TASK, SUBJECT, SECOND>>(&commit)
                        .await?;
                    Ok(resource)
                };
            }
            .await?
        };
        let (first_result, second_result) = async {
            async { Ok(first.recv::<CommitCompleted<FIRST>>().await?.to_vec()) };
            async { Ok(second.recv::<CommitCompleted<SECOND>>().await?.to_vec()) };
        }
        .await?;
        assert_eq!(first_result, second_result);
        assert_eq!(
            serde_json::from_slice::<serde_json::Value>(&first_result).unwrap()["tool"],
            "commit_changes"
        );
        {
            let [mut first_return, mut second_return] = task.par()?;
            async {
                async {
                    first_return.send::<Loan<FIRST>>(first).await?;
                };
                async {
                    second_return.send::<Loan<SECOND>>(second).await?;
                };
                Ok(())
            }
            .await?;
        }
        let test = Close::<TASK, SUBJECT, TEST>::new(
            close,
            serde_json::json!({"feature_name":input.feature}),
        )?;
        task.send::<TestRequest<TASK, SUBJECT>>(&test).await?;

        let branch = task.offer().await?;
        match branch.label() {
            183 => {
                let bytes = branch.recv::<TestPassed>().await?;
                let result: serde_json::Value = serde_json::from_slice(bytes).unwrap();
                assert_eq!(result["success"], true);
                break 'revisions;
            }
            184 => {
                let bytes = branch.recv::<TestFailed>().await?;
                let result: serde_json::Value = serde_json::from_slice(bytes).unwrap();
                assert_eq!(result["success"], false);
            }
            _ => panic!("unexpected test result branch"),
        }
    }
    task.send::<ServiceExit>(&0).await?;
    private.send::<Exit>(&0).await?;
    Ok(())
}

macro_rules! revising_fixture {
    (agents [$(($service:ident,$subject:ident,$vm:ident,$vm_role:ident,$io:ident,$io_role:ident,$first:ident,$second:ident,$result:ident,$check:ident,$name:literal)),+];
     resources [$(($owner:ident,$owner_role:ident,$resource:ident,$resource_role:ident,$first_owner:ident,$first_resource:ident,$second_owner:ident,$second_resource:ident)),+])=>{
        #[hibana_runtime::main]
        #[test]
        async fn seven_developers_reconsider_twice_and_commit_disjoint_modules_in_parallel()->ShellResult<()> {
            use super::global::roles1h::*;
            use hibana::runtime::resolver::ResolverRef;
            let programs=project_1h();
            $(let $result=super::input::TestResult::default();)+
            let session=SessionId::new(20);
            let(bus,_reader,_terminals)=Bus::open([$($name),+],[$((session,$resource_role)),+]);
            let mut slab=vec![0;2*1024*1024];
            let mut storage=SessionKitStorage::uninit();
            let runtime=storage.init().rendezvous(&mut slab,&bus.transport)?;
            $(runtime.set_resolver(&programs.$service,ResolverRef::<$check>::decision_state(&$result,super::input::TestResult::decide)).map_err(|error|crate::shell::ShellError::message(format!("{error:?}")))?;
              let mut $service=runtime.enter(session,&programs.$service)?;
              let mut $vm=runtime.enter(session,&programs.$vm)?;
              let mut $io=runtime.enter(session,&programs.$io)?;)+
            $(let mut $owner=runtime.enter(session,&programs.$owner)?;
              let $resource=runtime.enter(session,&programs.$resource)?;)+
            let effects=Effects{test_results:&[false,false,true],concurrent_agents:&["DEVELOPER_A","DEVELOPER_C","DEVELOPER_E"],..Effects::default()};
            let outcome = hibana_runtime::race(async {
                $(
                    async {
                        localside::developer_reconsider::run::<HARD,$subject,$first,$second,{$first-21},{$second-21}>(&mut $service,&effects,&$result).await
                            .map_err(|error|crate::shell::ShellError::message(format!("service {}: {error:?}",$name)))?;
                    };
                    async {
                        revising_client::<HARD,$subject,$vm_role,$first,$second>(&mut $vm).await
                            .map_err(|error|crate::shell::ShellError::message(format!("guest {}: {error:?}",$name)))?;
                    };
                    async { assert_eq!($io.offer().await?.recv::<Exit>().await?,0); };
                )+
                $(
                    async {
                        localside::module_reconsider::run::<$owner_role,$resource_role,$first_owner,$first_resource,$second_owner,$second_resource>(&mut $owner,$resource).await
                            .map_err(|error|crate::shell::ShellError::message(format!("module {}: {error:?}",$owner_role)))?;
                    };
                )+
                Ok(())
            }, hibana_runtime::host::deadline(std::time::Duration::from_secs(10))).await;
            if outcome.is_err() {
                eprintln!("effects: {:?}",effects.calls.take());
                for event in runtime.tap() {eprintln!("tap: {event:?}");}
                for frame in bus.transport.take_observations() {eprintln!("frame: {frame}");}
            }
            outcome?;
            let calls=effects.calls.take();
            $(for (tool,expected) in [("design_feature",1),("implement_code",3),("commit_changes",3),("run_local_tests",3)] {
                assert_eq!(calls.iter().filter(|entry| **entry==($name,tool)).count(),expected,"wrong attempts for {} {tool}",$name);
            })+
            assert_eq!(effects.peak.get(),3);
            assert!(effects.active.take().is_empty());
            Ok(())
        }
    };
}
revising_fixture!(
    agents [(a,A,a_wasi,A_WASI,a_io,A_IO,AUTH_RESOURCE,DATABASE_RESOURCE,result_a,CHECK_A,"DEVELOPER_A"),
            (b,B,b_wasi,B_WASI,b_io,B_IO,API_RESOURCE,DATABASE_RESOURCE,result_b,CHECK_B,"DEVELOPER_B"),
            (c,C,c_wasi,C_WASI,c_io,C_IO,API_RESOURCE,PAYMENT_RESOURCE,result_c,CHECK_C,"DEVELOPER_C"),
            (d,D,d_wasi,D_WASI,d_io,D_IO,NOTIFY_RESOURCE,PAYMENT_RESOURCE,result_d,CHECK_D,"DEVELOPER_D"),
            (e,E,e_wasi,E_WASI,e_io,E_IO,ANALYTICS_RESOURCE,NOTIFY_RESOURCE,result_e,CHECK_E,"DEVELOPER_E"),
            (f,F,f_wasi,F_WASI,f_io,F_IO,ANALYTICS_RESOURCE,SEARCH_RESOURCE,result_f,CHECK_F,"DEVELOPER_F"),
            (g,G,g_wasi,G_WASI,g_io,G_IO,AUTH_RESOURCE,SEARCH_RESOURCE,result_g,CHECK_G,"DEVELOPER_G")];
    resources [(auth,AUTH,auth_resource,AUTH_RESOURCE,DATABASE,DATABASE_RESOURCE,SEARCH,SEARCH_RESOURCE),
               (database,DATABASE,database_resource,DATABASE_RESOURCE,AUTH,AUTH_RESOURCE,API,API_RESOURCE),
               (api,API,api_resource,API_RESOURCE,PAYMENT,PAYMENT_RESOURCE,DATABASE,DATABASE_RESOURCE),
               (payment,PAYMENT,payment_resource,PAYMENT_RESOURCE,API,API_RESOURCE,NOTIFY,NOTIFY_RESOURCE),
               (notify,NOTIFY,notify_resource,NOTIFY_RESOURCE,ANALYTICS,ANALYTICS_RESOURCE,PAYMENT,PAYMENT_RESOURCE),
               (analytics,ANALYTICS,analytics_resource,ANALYTICS_RESOURCE,NOTIFY,NOTIFY_RESOURCE,SEARCH,SEARCH_RESOURCE),
               (search,SEARCH,search_resource,SEARCH_RESOURCE,ANALYTICS,ANALYTICS_RESOURCE,AUTH,AUTH_RESOURCE)]);

mod owned_exclusion;
mod wasi_admission;
