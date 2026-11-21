//! Wash files for the three jobs that use the two-tool cabinet.
use super::manufacturing_rework::{global::*, imp, localside};
use crate::shell::{
    GUEST_MEMORY_BYTES, ShellError, ShellResult,
    global::{Exit, FileExit, Loan, file_metadata, private_io},
    imp::{bus::Bus, private_fs::PrivateFs},
    localside::{file_metadata, private_io},
};
use hibana::{
    Endpoint, g,
    runtime::{
        SessionKitStorage,
        ids::SessionId,
        program::{Projectable, project},
        resolver::ResolverRef,
    },
};
use hibana_wasip1_runtime::{
    ChoreoFsFile, ChoreoFsFiles, GuestMemory, HibanaWasiGuestStorage,
    choreofs::{ChoreoFsObject, FdSpec, ObjectId},
    protocol::{FdBinding, FdReadRow, FdWriteRow},
};
use serde_json::{Value, json};
use std::cell::Cell;

const A_VM: u8 = 9;
const B_VM: u8 = 10;
const C_VM: u8 = 11;
const A_TERM: u8 = 12;
const B_TERM: u8 = 13;
const C_TERM: u8 = 14;
const A_FILES: u8 = 15;
const B_FILES: u8 = 16;
const C_FILES: u8 = 17;
const INSPECTOR_VM: u8 = 18;
const INSPECTOR_TERM: u8 = 19;
const INSPECTOR_FILES: u8 = 20;
const DISPATCHER_VM: u8 = 21;
const DISPATCHER_TERM: u8 = 22;
const DISPATCHER_FILES: u8 = 23;

const fn readable(path: &'static [u8], fd: u8) -> ChoreoFsObject<'static> {
    ChoreoFsObject::readable(
        path,
        ObjectId(fd as u32),
        FdSpec::new(fd, 2),
        b"",
        FdBinding::read(FdReadRow::Base),
    )
}
const fn inspect(path: &'static [u8], fd: u8) -> ChoreoFsObject<'static> {
    ChoreoFsObject::writable(
        path,
        ObjectId(fd as u32),
        FdSpec::new(fd, 64),
        FdBinding::write(FdWriteRow::Object),
    )
}
const INSPECTION_FILES: [ChoreoFsObject<'static>; 7] = [
    readable(b"service/parts/alpha_job_a", 4),
    readable(b"service/parts/beta_job_b", 5),
    readable(b"service/parts/gamma_job_c", 6),
    inspect(b"service/inspect/alpha_job_a", 7),
    inspect(b"service/inspect/beta_job_b", 8),
    inspect(b"service/inspect/gamma_job_c", 9),
    readable(b"service/inspection_result", 10),
];

const fn request_reply(path: &'static [u8], fd: u8) -> ChoreoFsObject<'static> {
    ChoreoFsObject::read_write(
        path,
        ObjectId(fd as u32),
        FdSpec::new(fd, 66),
        b"",
        FdBinding {
            read: Some(FdReadRow::Base),
            write: Some(FdWriteRow::Object),
            readdir: None,
        },
    )
}
const JOB: ChoreoFsObject<'static> = request_reply(b"service/do_job", 4);
const WORKSTATION_FILES: [ChoreoFsObject<'static>; 2] = [JOB, readable(b"service/job_update", 5)];
const ORDERS: [ChoreoFsObject<'static>; 9] = [
    readable(b"service/approved/alpha_job_a", 4),
    readable(b"service/approved/beta_job_b", 5),
    readable(b"service/approved/gamma_job_c", 6),
    readable(b"service/failed/alpha_job_a", 7),
    readable(b"service/failed/beta_job_b", 8),
    readable(b"service/failed/gamma_job_c", 9),
    request_reply(b"service/dispatch/alpha_job_a", 10),
    request_reply(b"service/dispatch/beta_job_b", 11),
    request_reply(b"service/dispatch/gamma_job_c", 12),
];

fn shell_io<const VM: u8, const STATION: u8, const FILES: u8, const TERM: u8>() -> impl Projectable
{
    g::par(
        g::seq(
            file_metadata::<STATION, FILES>().roll(),
            g::send::<STATION, FILES, FileExit>(),
        ),
        g::seq(private_io::<VM, TERM>(), g::send::<VM, TERM, Exit>()),
    )
}

#[hibana_runtime::scope]
async fn worker_shell<const WORKER: u8, const VM: u8, const STATION: u8>(
    mut vm: Endpoint<VM>,
    module: &[u8],
) -> ShellResult<()> {
    let station = vm.recv::<Loan<STATION>>().await?;
    let mut request = vec![0; GUEST_MEMORY_BYTES];
    let mut result = vec![0; GUEST_MEMORY_BYTES];
    let mut update = vec![0; GUEST_MEMORY_BYTES];
    let file = ChoreoFsFile::new(JOB, station)
        .write_as::<StationJob<WORKER>>(&mut request)
        .read_as::<JobResult>(&mut result)
        .file(WORKSTATION_FILES[1])?
        .read_as::<JobUpdate>(&mut update);
    let files = ChoreoFsFiles::new(vm).file(file)?;
    let mut memory = vec![0; GUEST_MEMORY_BYTES];
    let mut storage = Box::new(HibanaWasiGuestStorage::uninit());
    let guest = storage.init(module, GuestMemory::new(&mut memory), files)?;
    let (exit, files) = guest.run().await?;
    let status = exit.status() as i32;
    let (mut vm, (file, ())) = files.into_parts();
    async {
        file.into_endpoint().send::<FileExit>(&status).await?;
    };
    vm.send::<Exit>(&status).await?;
    Ok(())
}

#[hibana_runtime::scope]
pub async fn run(path: &str) -> ShellResult<()> {
    use crate::imp::projection::rows;
    let loans = g::par(
        g::send::<A_FILES, A_VM, Loan<ALPHA>>(),
        g::par(
            g::send::<B_FILES, B_VM, Loan<BETA>>(),
            g::send::<C_FILES, C_VM, Loan<GAMMA>>(),
        ),
    );
    let io = g::par(
        shell_io::<A_VM, ALPHA, A_FILES, A_TERM>(),
        g::par(
            shell_io::<B_VM, BETA, B_FILES, B_TERM>(),
            shell_io::<C_VM, GAMMA, C_FILES, C_TERM>(),
        ),
    );
    let global = g::seq(
        g::par(
            loans,
            g::par(
                g::send::<INSPECTOR_FILES, INSPECTOR_VM, Loan<INSPECTOR>>(),
                g::send::<DISPATCHER_FILES, DISPATCHER_VM, Loan<DISPATCHER>>(),
            ),
        ),
        g::par(
            source(),
            g::par(
                io,
                g::par(
                    shell_io::<INSPECTOR_VM, INSPECTOR, INSPECTOR_FILES, INSPECTOR_TERM>(),
                    shell_io::<DISPATCHER_VM, DISPATCHER, DISPATCHER_FILES, DISPATCHER_TERM>(),
                ),
            ),
        ),
    );
    let inspector = project::<INSPECTOR>(&global);
    let dispatcher = project::<DISPATCHER>(&global);
    let dispatcher_vm = project::<DISPATCHER_VM>(&global);
    let dispatcher_files = project::<DISPATCHER_FILES>(&global);
    let dispatcher_term = project::<DISPATCHER_TERM>(&global);
    let inspector_vm = project::<INSPECTOR_VM>(&global);
    let inspector_files = project::<INSPECTOR_FILES>(&global);
    let inspector_term = project::<INSPECTOR_TERM>(&global);
    let worker_a = project::<WORKER_A>(&global);
    let worker_b = project::<WORKER_B>(&global);
    let worker_c = project::<WORKER_C>(&global);
    let depot = project::<DEPOT>(&global);
    let tool_0 = project::<TOOL_0>(&global);
    let tool_1 = project::<TOOL_1>(&global);
    let alpha = project::<ALPHA>(&global);
    let beta = project::<BETA>(&global);
    let gamma = project::<GAMMA>(&global);
    let a_vm = project::<A_VM>(&global);
    let b_vm = project::<B_VM>(&global);
    let c_vm = project::<C_VM>(&global);
    let a_term = project::<A_TERM>(&global);
    let b_term = project::<B_TERM>(&global);
    let c_term = project::<C_TERM>(&global);
    let a_files = project::<A_FILES>(&global);
    let b_files = project::<B_FILES>(&global);
    let c_files = project::<C_FILES>(&global);
    let projection = json!({
        "type": "compiled_projection", "format_version": 1, "task": "11M",
        "scope": "tool-requiring jobs, inspection and rework orders", "session": SESSION,
        "roles": {
            "DISPATCHER": DISPATCHER, "DISPATCHER_VM": DISPATCHER_VM, "DISPATCHER_FILES": DISPATCHER_FILES, "DISPATCHER_TERM": DISPATCHER_TERM,
            "INSPECTOR": INSPECTOR, "INSPECTOR_VM": INSPECTOR_VM, "INSPECTOR_FILES": INSPECTOR_FILES, "INSPECTOR_TERM": INSPECTOR_TERM,
            "WORKER_A": WORKER_A, "WORKER_B": WORKER_B, "WORKER_C": WORKER_C, "DEPOT": DEPOT,
            "TOOL_0": TOOL_0, "TOOL_1": TOOL_1, "ALPHA": ALPHA, "BETA": BETA, "GAMMA": GAMMA,
            "A_VM": A_VM, "B_VM": B_VM, "C_VM": C_VM,
            "A_TERM": A_TERM, "B_TERM": B_TERM, "C_TERM": C_TERM,
            "A_FILES": A_FILES, "B_FILES": B_FILES, "C_FILES": C_FILES,
        },
        "programs": [
            rows(&dispatcher), rows(&dispatcher_vm), rows(&dispatcher_files), rows(&dispatcher_term),
            rows(&inspector), rows(&inspector_vm), rows(&inspector_files), rows(&inspector_term),
            rows(&worker_a), rows(&worker_b), rows(&worker_c), rows(&depot),
            rows(&tool_0), rows(&tool_1), rows(&alpha), rows(&beta), rows(&gamma),
            rows(&a_vm), rows(&b_vm), rows(&c_vm), rows(&a_term), rows(&b_term), rows(&c_term),
            rows(&a_files), rows(&b_files), rows(&c_files),
        ],
    });
    if path == "--projection" {
        println!("{projection}");
        return Ok(());
    }
    let module = std::fs::read(path)?;
    let session = SessionId::new(SESSION);
    let (
        mut connection,
        mut io,
        [
            terminal_a,
            terminal_b,
            terminal_c,
            terminal_inspector,
            terminal_dispatcher,
        ],
    ) = Bus::open(
        [
            "WORKER_A",
            "WORKER_B",
            "WORKER_C",
            "INSPECTOR",
            "DISPATCHER",
        ],
        [
            (session, TOOL_0),
            (session, TOOL_1),
            (session, ALPHA),
            (session, BETA),
            (session, GAMMA),
            (session, INSPECTOR),
            (session, DISPATCHER),
        ],
    );
    let (bus, simulator) = connection.split();
    bus.projected(projection);
    let capture = |row| bus.capture(row);
    let mut slab = vec![0; 1024 * 1024];
    let inputs = [const { Cell::new(Value::Null) }; 5];
    let next = (&inputs[0], &inputs[1]);
    let mut storage = SessionKitStorage::uninit();
    let rv = storage.init().rendezvous(&mut slab, &bus.transport)?;
    rv.observe(&capture);
    let observations = rv.tap();
    let trial = async {
        rv.set_resolver(
            &depot,
            ResolverRef::<FIRST_A>::decision_state(&inputs[0], imp::worker_a),
        )
        .map_err(|e| ShellError::message(format!("{e:?}")))?;
        rv.set_resolver(
            &depot,
            ResolverRef::<FIRST_B>::decision_state(&inputs[0], imp::worker_b),
        )
        .map_err(|e| ShellError::message(format!("{e:?}")))?;
        rv.set_resolver(
            &depot,
            ResolverRef::<NEXT>::decision_state(&next, imp::next),
        )
        .map_err(|e| ShellError::message(format!("{e:?}")))?;
        rv.set_resolver(
            &depot,
            ResolverRef::<ARRIVAL_0>::decision_state(&inputs[2], imp::arrival),
        )
        .map_err(|e| ShellError::message(format!("{e:?}")))?;
        rv.set_resolver(
            &depot,
            ResolverRef::<ARRIVAL_1>::decision_state(&inputs[3], imp::arrival),
        )
        .map_err(|e| ShellError::message(format!("{e:?}")))?;
        rv.set_resolver(
            &depot,
            ResolverRef::<FINISHED_FIRST>::decision_state(&inputs[4], imp::finished),
        )
        .map_err(|e| ShellError::message(format!("{e:?}")))?;
        let mut inspector_files = rv.enter(session, &inspector_files)?;
        let mut dispatcher_files = rv.enter(session, &dispatcher_files)?;
        let dispatcher = rv.enter(session, &dispatcher)?;
        let mut dispatcher_vm = rv.enter(session, &dispatcher_vm)?;
        let mut dispatcher_term = rv.enter(session, &dispatcher_term)?;
        async {
            file_metadata::run(&mut dispatcher_files, dispatcher, ORDERS).await?;
        };
        async {
            private_io::run(
                &mut dispatcher_term,
                PrivateFs::new(terminal_dispatcher, bus, &ORDERS),
            )
            .await?;
        };
        async {
            let endpoint = dispatcher_vm.recv::<Loan<DISPATCHER>>().await?;
            let [
                mut approved_a,
                mut approved_b,
                mut approved_c,
                mut failed_a,
                mut failed_b,
                mut failed_c,
            ] = [(); 6].map(|_| vec![0; GUEST_MEMORY_BYTES]);
            let [
                mut request_a,
                mut request_b,
                mut request_c,
                mut order_a,
                mut order_b,
                mut order_c,
            ] = [(); 6].map(|_| vec![0; GUEST_MEMORY_BYTES]);
            let orders = ChoreoFsFile::new(ORDERS[0], endpoint)
                .read_as::<Approved<WORKER_A>>(&mut approved_a)
                .file(ORDERS[1])?
                .read_as::<Approved<WORKER_B>>(&mut approved_b)
                .file(ORDERS[2])?
                .read_as::<Approved<WORKER_C>>(&mut approved_c)
                .file(ORDERS[3])?
                .read_as::<Failed<WORKER_A>>(&mut failed_a)
                .file(ORDERS[4])?
                .read_as::<Failed<WORKER_B>>(&mut failed_b)
                .file(ORDERS[5])?
                .read_as::<Failed<WORKER_C>>(&mut failed_c)
                .file(ORDERS[6])?
                .write_as::<DispatchRequest<WORKER_A>>(&mut request_a)
                .read_as::<WorkOrder<WORKER_A>>(&mut order_a)
                .file(ORDERS[7])?
                .write_as::<DispatchRequest<WORKER_B>>(&mut request_b)
                .read_as::<WorkOrder<WORKER_B>>(&mut order_b)
                .file(ORDERS[8])?
                .write_as::<DispatchRequest<WORKER_C>>(&mut request_c)
                .read_as::<WorkOrder<WORKER_C>>(&mut order_c);
            let files = ChoreoFsFiles::new(dispatcher_vm).file(orders)?;
            let mut memory = vec![0; GUEST_MEMORY_BYTES];
            let mut storage = Box::new(HibanaWasiGuestStorage::uninit());
            let guest = storage.init(&module, GuestMemory::new(&mut memory), files)?;
            let (exit, files) = guest.run().await?;
            let status = exit.status() as i32;
            let (mut vm, (orders, ())) = files.into_parts();
            async {
                orders.into_endpoint().send::<FileExit>(&status).await?;
            };
            vm.send::<Exit>(&status).await?;
            Ok(())
        };
        let inspector = rv.enter(session, &inspector)?;
        let mut inspector_vm = rv.enter(session, &inspector_vm)?;
        let mut inspector_term = rv.enter(session, &inspector_term)?;
        async {
            file_metadata::run(&mut inspector_files, inspector, INSPECTION_FILES).await?;
        };
        async {
            private_io::run(
                &mut inspector_term,
                PrivateFs::new(terminal_inspector, bus, &INSPECTION_FILES),
            )
            .await?;
        };
        async {
            let inbox = inspector_vm.recv::<Loan<INSPECTOR>>().await?;
            let [mut alpha, mut beta, mut gamma] = [(); 3].map(|_| vec![0; GUEST_MEMORY_BYTES]);
            let [mut check_a, mut check_b, mut check_c, mut verdict] =
                [(); 4].map(|_| vec![0; GUEST_MEMORY_BYTES]);
            let parts = ChoreoFsFile::new(INSPECTION_FILES[0], inbox)
                .read_as::<FinishedPart<WORKER_A>>(&mut alpha)
                .file(INSPECTION_FILES[1])?
                .read_as::<FinishedPart<WORKER_B>>(&mut beta)
                .file(INSPECTION_FILES[2])?
                .read_as::<FinishedPart<WORKER_C>>(&mut gamma)
                .file(INSPECTION_FILES[3])?
                .write_as::<CheckPart<WORKER_A>>(&mut check_a)
                .file(INSPECTION_FILES[4])?
                .write_as::<CheckPart<WORKER_B>>(&mut check_b)
                .file(INSPECTION_FILES[5])?
                .write_as::<CheckPart<WORKER_C>>(&mut check_c)
                .file(INSPECTION_FILES[6])?
                .read_as::<InspectionResult>(&mut verdict);
            let files = ChoreoFsFiles::new(inspector_vm).file(parts)?;
            let mut memory = vec![0; GUEST_MEMORY_BYTES];
            let mut storage = Box::new(HibanaWasiGuestStorage::uninit());
            let guest = storage.init(&module, GuestMemory::new(&mut memory), files)?;
            let (exit, files) = guest.run().await?;
            let status = exit.status() as i32;
            let (mut vm, (parts, ())) = files.into_parts();
            async {
                parts.into_endpoint().send::<FileExit>(&status).await?;
            };
            vm.send::<Exit>(&status).await?;
            Ok(())
        };
        let mut depot = rv.enter(session, &depot)?;
        let tool_0 = rv.enter(session, &tool_0)?;
        let tool_1 = rv.enter(session, &tool_1)?;
        let mut a_files = rv.enter(session, &a_files)?;
        let mut b_files = rv.enter(session, &b_files)?;
        let mut c_files = rv.enter(session, &c_files)?;
        let a_vm = rv.enter(session, &a_vm)?;
        let b_vm = rv.enter(session, &b_vm)?;
        let c_vm = rv.enter(session, &c_vm)?;
        let alpha = rv.enter(session, &alpha)?;
        let beta = rv.enter(session, &beta)?;
        let gamma = rv.enter(session, &gamma)?;
        let mut worker_a = rv.enter(session, &worker_a)?;
        let mut worker_b = rv.enter(session, &worker_b)?;
        let mut worker_c = rv.enter(session, &worker_c)?;
        let mut a_term = rv.enter(session, &a_term)?;
        let mut b_term = rv.enter(session, &b_term)?;
        let mut c_term = rv.enter(session, &c_term)?;
        async {
            localside::tool_depot::run(&mut depot, (tool_0, tool_1), (&simulator, &inputs)).await?;
        };
        async {
            localside::worker::run(&mut worker_a, &simulator).await?;
        };
        async {
            worker_shell::<WORKER_A, A_VM, ALPHA>(a_vm, &module).await?;
        };
        async {
            file_metadata::run(&mut a_files, alpha, WORKSTATION_FILES).await?;
        };
        async {
            private_io::run(
                &mut a_term,
                PrivateFs::new(terminal_a, bus, &WORKSTATION_FILES),
            )
            .await?;
        };
        async {
            localside::worker::run(&mut worker_b, &simulator).await?;
        };
        async {
            worker_shell::<WORKER_B, B_VM, BETA>(b_vm, &module).await?;
        };
        async {
            file_metadata::run(&mut b_files, beta, WORKSTATION_FILES).await?;
        };
        async {
            private_io::run(
                &mut b_term,
                PrivateFs::new(terminal_b, bus, &WORKSTATION_FILES),
            )
            .await?;
        };
        async {
            localside::worker::run(&mut worker_c, &simulator).await?;
        };
        async {
            worker_shell::<WORKER_C, C_VM, GAMMA>(c_vm, &module).await?;
        };
        async {
            file_metadata::run(&mut c_files, gamma, WORKSTATION_FILES).await?;
        };
        async {
            private_io::run(
                &mut c_term,
                PrivateFs::new(terminal_c, bus, &WORKSTATION_FILES),
            )
            .await?;
        };
        Ok::<(), ShellError>(())
    };
    let result = hibana_runtime::race(trial, io.run(bus)).await;
    bus.checkpoint(SESSION, observations.checkpoint());
    io.finish(bus, result, Some("workstation_tool_jobs")).await
}
