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

const fn part(path: &'static [u8], fd: u8) -> ChoreoFsObject<'static> {
    ChoreoFsObject::readable(
        path,
        ObjectId(fd as u32),
        FdSpec::new(fd, 2),
        b"",
        FdBinding::read(FdReadRow::Base),
    )
}
const PARTS: [ChoreoFsObject<'static>; 3] = [
    part(b"service/parts/alpha_job_a", 4),
    part(b"service/parts/beta_job_b", 5),
    part(b"service/parts/gamma_job_c", 6),
];

const JOB: ChoreoFsObject<'static> = ChoreoFsObject::read_write(
    b"service/do_job",
    ObjectId(4),
    FdSpec::new(4, 66),
    b"",
    FdBinding {
        read: Some(FdReadRow::Base),
        write: Some(FdWriteRow::Object),
        readdir: None,
    },
);

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
    let file = ChoreoFsFile::new(JOB, station)
        .write_as::<StationJob<WORKER>>(&mut request)
        .read_as::<JobResult>(&mut result);
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
            g::send::<INSPECTOR_FILES, INSPECTOR_VM, Loan<INSPECTOR>>(),
        ),
        g::par(
            source(),
            g::par(
                io,
                shell_io::<INSPECTOR_VM, INSPECTOR, INSPECTOR_FILES, INSPECTOR_TERM>(),
            ),
        ),
    );
    let inspector = project::<INSPECTOR>(&global);
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
        "scope": "tool-requiring jobs and part delivery", "session": SESSION,
        "roles": {
            "INSPECTOR": INSPECTOR, "INSPECTOR_VM": INSPECTOR_VM, "INSPECTOR_FILES": INSPECTOR_FILES, "INSPECTOR_TERM": INSPECTOR_TERM,
            "WORKER_A": WORKER_A, "WORKER_B": WORKER_B, "WORKER_C": WORKER_C, "DEPOT": DEPOT,
            "TOOL_0": TOOL_0, "TOOL_1": TOOL_1, "ALPHA": ALPHA, "BETA": BETA, "GAMMA": GAMMA,
            "A_VM": A_VM, "B_VM": B_VM, "C_VM": C_VM,
            "A_TERM": A_TERM, "B_TERM": B_TERM, "C_TERM": C_TERM,
            "A_FILES": A_FILES, "B_FILES": B_FILES, "C_FILES": C_FILES,
        },
        "programs": [
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
    let (mut connection, mut io, [terminal_a, terminal_b, terminal_c, terminal_inspector]) =
        Bus::open(
            ["WORKER_A", "WORKER_B", "WORKER_C", "INSPECTOR"],
            [
                (session, TOOL_0),
                (session, TOOL_1),
                (session, ALPHA),
                (session, BETA),
                (session, GAMMA),
                (session, INSPECTOR),
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
        let inspector = rv.enter(session, &inspector)?;
        let mut inspector_vm = rv.enter(session, &inspector_vm)?;
        let mut inspector_term = rv.enter(session, &inspector_term)?;
        async {
            file_metadata::run(&mut inspector_files, inspector, PARTS).await?;
        };
        async {
            private_io::run(
                &mut inspector_term,
                PrivateFs::new(terminal_inspector, bus, &PARTS),
            )
            .await?;
        };
        async {
            let inbox = inspector_vm.recv::<Loan<INSPECTOR>>().await?;
            let [mut alpha, mut beta, mut gamma] = [(); 3].map(|_| vec![0; GUEST_MEMORY_BYTES]);
            let parts = ChoreoFsFile::new(PARTS[0], inbox)
                .read_as::<FinishedPart<WORKER_A>>(&mut alpha)
                .file(PARTS[1])?
                .read_as::<FinishedPart<WORKER_B>>(&mut beta)
                .file(PARTS[2])?
                .read_as::<FinishedPart<WORKER_C>>(&mut gamma);
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
            localside::worker::run(&mut worker_a).await?;
        };
        async {
            worker_shell::<WORKER_A, A_VM, ALPHA>(a_vm, &module).await?;
        };
        async {
            file_metadata::run(&mut a_files, alpha, [JOB]).await?;
        };
        async {
            private_io::run(&mut a_term, PrivateFs::new(terminal_a, bus, &[JOB])).await?;
        };
        async {
            localside::worker::run(&mut worker_b).await?;
        };
        async {
            worker_shell::<WORKER_B, B_VM, BETA>(b_vm, &module).await?;
        };
        async {
            file_metadata::run(&mut b_files, beta, [JOB]).await?;
        };
        async {
            private_io::run(&mut b_term, PrivateFs::new(terminal_b, bus, &[JOB])).await?;
        };
        async {
            localside::worker::run(&mut worker_c).await?;
        };
        async {
            worker_shell::<WORKER_C, C_VM, GAMMA>(c_vm, &module).await?;
        };
        async {
            file_metadata::run(&mut c_files, gamma, [JOB]).await?;
        };
        async {
            private_io::run(&mut c_term, PrivateFs::new(terminal_c, bus, &[JOB])).await?;
        };
        Ok::<(), ShellError>(())
    };
    let result = hibana_runtime::race(trial, io.run(bus)).await;
    bus.checkpoint(SESSION, observations.checkpoint());
    io.finish(bus, result, Some("workstation_tool_jobs")).await
}
