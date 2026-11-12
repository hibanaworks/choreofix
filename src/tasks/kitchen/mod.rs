//! 12E development: delegated kitchen; full Wasm/model qualification is pending.
pub mod global;
pub(crate) mod imp;
pub mod localside;

use crate::shell::{ShellError, ShellResult};
use hibana::runtime::{SessionKitStorage, ids::SessionId, resolver::ResolverRef};

#[hibana_runtime::scope]
pub async fn run_stdio(module: &[u8]) -> ShellResult<()> {
    use crate::shell::{
        imp::{
            bus::Bus,
            private_fs::{PrivateFs, bindings},
        },
        localside::private_io,
    };
    use global::*;
    use hibana_wasip1_runtime::{GuestMemory, HibanaWasiGuestStorage};
    use std::cell::Cell;
    let session = SessionId::new(SESSION);
    let (bus, mut io, [a_terminal, b_terminal, c_terminal]) =
        Bus::open(imp::CHEFS, [(session, OVEN), (session, STOVE)]);
    let result=hibana_runtime::race(async {
        let p=programs();
        bus.observe(p.observation());
        let capture=|event|bus.observe(serde_json::json!({"type":"protocol_capture","producer":0,
            "records":[crate::imp::projection::record(event)]}));
        let inputs=[const{Cell::new(serde_json::Value::Null)};2];
        let next=(&inputs[0],&inputs[1]);
        let mut slab=vec![0;1024*1024];
        let mut storage=SessionKitStorage::uninit();
        let rv=storage.init().rendezvous(&mut slab,&bus.transport)?;
        rv.observe(&capture);
        let observations=rv.tap();
        let result=async {
            let mut a=rv.enter(session,&p.a)?;let mut b=rv.enter(session,&p.b)?;let mut c=rv.enter(session,&p.c)?;
            let mut a_io=rv.enter(session,&p.a_io)?;let mut b_io=rv.enter(session,&p.b_io)?;let mut c_io=rv.enter(session,&p.c_io)?;
            let mut cooking=rv.enter(session,&p.cooking)?;let mut ingredients=rv.enter(session,&p.ingredients)?;
            let mut finish=rv.enter(session,&p.finish)?;
            let oven=rv.enter(session,&p.oven)?;let stove=rv.enter(session,&p.stove)?;
            rv.set_resolver(&p.cooking,ResolverRef::<FIRST_A>::decision_state(&inputs[0],imp::first_a))
                .map_err(|e|ShellError::message(format!("{e:?}")))?;
            rv.set_resolver(&p.cooking,ResolverRef::<FIRST_B>::decision_state(&inputs[0],imp::first_b))
                .map_err(|e|ShellError::message(format!("{e:?}")))?;
            rv.set_resolver(&p.cooking,ResolverRef::<NEXT>::decision_state(&next,imp::next))
                .map_err(|e|ShellError::message(format!("{e:?}")))?;
            let a_fs=PrivateFs::new(a_terminal,&bus);let b_fs=PrivateFs::new(b_terminal,&bus);let c_fs=PrivateFs::new(c_terminal,&bus);
            let mut a_memory=vec![0;crate::shell::GUEST_MEMORY_BYTES];
            let mut b_memory=vec![0;crate::shell::GUEST_MEMORY_BYTES];
            let mut c_memory=vec![0;crate::shell::GUEST_MEMORY_BYTES];
            let mut a_storage=HibanaWasiGuestStorage::uninit();
            let mut b_storage=HibanaWasiGuestStorage::uninit();
            let mut c_storage=HibanaWasiGuestStorage::uninit();
            let a_guest=a_storage.init(module,GuestMemory::new(&mut a_memory),bindings())?;
            let b_guest=b_storage.init(module,GuestMemory::new(&mut b_memory),bindings())?;
            let c_guest=c_storage.init(module,GuestMemory::new(&mut c_memory),bindings())?;
            async {localside::cooking::run(&mut cooking,(oven,stove),(&bus,&inputs)).await?;};
            async {localside::ingredients::run(&mut ingredients,&bus).await?;};
            async {localside::finish::run(&mut finish,&bus).await?;};
            async {let status=localside::chef_a::run(a_guest,&mut a).await?;if status!=0{return Err(ShellError::message("chef A exited unsuccessfully"));}};
            async {let status=localside::chef_b::run(b_guest,&mut b).await?;if status!=0{return Err(ShellError::message("chef B exited unsuccessfully"));}};
            async {let status=localside::chef_c::run(c_guest,&mut c).await?;if status!=0{return Err(ShellError::message("chef C exited unsuccessfully"));}};
            async {private_io::run(&mut a_io,a_fs).await?;};
            async {private_io::run(&mut b_io,b_fs).await?;};
            async {private_io::run(&mut c_io,c_fs).await?;};
            Ok::<(),ShellError>(())
        }.await;
        bus.observe(serde_json::json!({"type":"protocol_checkpoint","producer":0,"session":SESSION,"checkpoint":observations.checkpoint()}));
        result
    },io.run(&bus)).await;
    match &result {
        Ok(())=>io.emit(&bus,serde_json::json!({"type":"run_result","status":"complete"})).await?,
        Err(error)=>io.emit(&bus,serde_json::json!({"type":"run_result","status":"failed","error":error.to_string()})).await?,
    }
    result
}
pub fn projection() -> serde_json::Value {
    global::programs().observation()
}
