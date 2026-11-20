use choreofix::{
    imp::json::encode,
    imp::projection::rows,
    shell::{
        GUEST_MEMORY_BYTES, ShellError, ShellResult,
        global::{Exit, FileExit, Loan, file_metadata, private_io},
        imp::{
            bus::Bus, private_fs::PrivateFs, review_reply::ReviewReply, service::ReceivedEffects,
        },
        localside,
    },
};
use hibana::{
    Endpoint,
    g::{self, Message},
    runtime::{
        SessionKitStorage,
        ids::SessionId,
        program::{Projectable, project},
    },
};
use hibana_wasip1_runtime::{
    ChoreoFsFile, ChoreoFsFiles, GuestMemory, HibanaWasiGuestStorage,
    choreofs::{ChoreoFsObject, FdSpec, ObjectId},
    protocol::{FdBinding, FdReadRow, FdWriteRow},
};
use serde_json::json;

#[path = "../tasks/wafer_fabrication/instruments.rs"]
mod instruments;
use instruments::*;

const TERMINAL: u8 = 5;
const SESSION: u32 = 150;
const LITHO: u8 = 9;
const LITHO_INBOX: u8 = 10;
const LITHO_FILES: u8 = 11;
const LITHO_TERM: u8 = 12;
const DEP: u8 = 13;
const DEP_INBOX: u8 = 14;
const DEP_FILES: u8 = 15;
const DEP_TERM: u8 = 16;
const ETCH: u8 = 17;
const ETCH_INBOX: u8 = 18;
const ETCH_FILES: u8 = 19;
const ETCH_TERM: u8 = 20;
type Disposition<'a, const INBOX: u8> = g::Msg<INBOX, &'a [u8]>;

const INBOX_FILE: ChoreoFsObject<'static> = ChoreoFsObject::readable(
    b"service/lot_disposition",
    ObjectId(4),
    FdSpec::new(4, 2),
    b"",
    FdBinding::read(FdReadRow::Base),
);

fn recipient_io<const VM: u8, const INBOX: u8, const DATA: u8, const TERM: u8>()
-> impl Projectable + Copy {
    g::par(
        g::seq(
            file_metadata::<INBOX, DATA>().roll(),
            g::send::<INBOX, DATA, FileExit>(),
        ),
        g::seq(private_io::<VM, TERM>(), g::send::<VM, TERM, Exit>()),
    )
}

async fn fabrication_guest<'run, const VM: u8, const INBOX: u8>(
    mut vm: Endpoint<'run, VM>,
    module: &[u8],
) -> ShellResult<()> {
    let inbox = vm.recv::<Loan<INBOX>>().await?;
    let mut buffer = vec![0; GUEST_MEMORY_BYTES];
    let file = ChoreoFsFile::new(INBOX_FILE, inbox).read_as::<Disposition<INBOX>>(&mut buffer);
    let files = ChoreoFsFiles::new(vm).file(file)?;
    let mut memory = vec![0; GUEST_MEMORY_BYTES];
    let mut storage = Box::new(HibanaWasiGuestStorage::uninit());
    let guest = storage.init(module, GuestMemory::new(&mut memory), files)?;
    let (exit, files) = guest.run().await?;
    let status = exit.status() as i32;
    let (mut vm, (file, ())) = files.into_parts();
    let mut inbox = file.into_endpoint();
    inbox.send::<FileExit>(&status).await?;
    vm.send::<Exit>(&status).await?;
    Ok(())
}

#[hibana_runtime::main]
async fn main() -> ShellResult<()> {
    let path = std::env::args()
        .nth(1)
        .ok_or_else(|| ShellError::message("usage: wafer-wasi GUEST.wasm"))?;
    let probe = g::send::<WAFER, STAGE, Measure>();
    let reading = g::send::<STEPPER, WAFER, Measured>();
    let failed = g::seq(
        g::send::<STAGE, STEPPER, OverlayFailed>(),
        g::seq(reading, probe),
    );
    let passed = g::seq(
        g::send::<STAGE, STEPPER, OverlayPassed>(),
        g::seq(g::send::<METROLOGY, STEPPER, Loan<STAGE>>(), reading),
    );
    let alignment = g::seq(
        g::send::<WAFER, STAGE, Mount>(),
        g::seq(
            g::send::<STEPPER, METROLOGY, Loan<STAGE>>(),
            g::par(
                g::send::<STEPPER, WAFER, Mounted>(),
                g::seq(probe, g::route(passed, g::seq(failed.roll(), passed))),
            ),
        ),
    );
    let notices = g::par(
        g::send::<WAFER, LITHO_INBOX, Disposition<LITHO_INBOX>>(),
        g::par(
            g::send::<WAFER, DEP_INBOX, Disposition<DEP_INBOX>>(),
            g::send::<WAFER, ETCH_INBOX, Disposition<ETCH_INBOX>>(),
        ),
    );
    let bootstrap = g::par(
        g::send::<ALIGNER, ENGINEER, Loan<WAFER>>(),
        g::par(
            g::send::<LITHO_FILES, LITHO, Loan<LITHO_INBOX>>(),
            g::par(
                g::send::<DEP_FILES, DEP, Loan<DEP_INBOX>>(),
                g::send::<ETCH_FILES, ETCH, Loan<ETCH_INBOX>>(),
            ),
        ),
    );
    let services = g::par(
        recipient_io::<LITHO, LITHO_INBOX, LITHO_FILES, LITHO_TERM>(),
        g::par(
            recipient_io::<DEP, DEP_INBOX, DEP_FILES, DEP_TERM>(),
            recipient_io::<ETCH, ETCH_INBOX, ETCH_FILES, ETCH_TERM>(),
        ),
    );
    let source = g::seq(
        bootstrap,
        g::par(
            services,
            g::par(
                g::seq(
                    g::par(
                        g::seq(alignment, g::seq(checks(), notices)),
                        g::seq(
                            file_metadata::<WAFER, ALIGNER>().roll(),
                            g::send::<WAFER, ALIGNER, FileExit>(),
                        ),
                    ),
                    g::send::<ENGINEER, ALIGNER, Loan<WAFER>>(),
                ),
                g::seq(
                    private_io::<ENGINEER, TERMINAL>(),
                    g::send::<ENGINEER, TERMINAL, Exit>(),
                ),
            ),
        ),
    );
    let aligner = project::<ALIGNER>(&source);
    let engineer = project::<ENGINEER>(&source);
    let wafer = project::<WAFER>(&source);
    let deposition_chamber = project::<DEPOSITION_CHAMBER>(&source);
    let etch_chamber = project::<ETCH_CHAMBER>(&source);
    let stepper = project::<STEPPER>(&source);
    let metrology = project::<METROLOGY>(&source);
    let stage = project::<STAGE>(&source);
    let terminal = project::<TERMINAL>(&source);
    let p_litho = project::<LITHO>(&source);
    let p_litho_inbox = project::<LITHO_INBOX>(&source);
    let p_litho_files = project::<LITHO_FILES>(&source);
    let p_litho_term = project::<LITHO_TERM>(&source);
    let p_dep = project::<DEP>(&source);
    let p_dep_inbox = project::<DEP_INBOX>(&source);
    let p_dep_files = project::<DEP_FILES>(&source);
    let p_dep_term = project::<DEP_TERM>(&source);
    let p_etch = project::<ETCH>(&source);
    let p_etch_inbox = project::<ETCH_INBOX>(&source);
    let p_etch_files = project::<ETCH_FILES>(&source);
    let p_etch_term = project::<ETCH_TERM>(&source);
    let projection = json!({
        "type":"compiled_projection", "format_version":1, "task":"15E", "session":SESSION,
        "roles":{
            "ALIGNER":ALIGNER, "ENGINEER":ENGINEER, "WAFER":WAFER, "TERMINAL":TERMINAL,
            "DEPOSITION_CHAMBER":DEPOSITION_CHAMBER, "ETCH_CHAMBER":ETCH_CHAMBER,
            "STEPPER":STEPPER, "METROLOGY":METROLOGY, "STAGE":STAGE,
            "LITHO":LITHO, "LITHO_INBOX":LITHO_INBOX, "LITHO_FILES":LITHO_FILES, "LITHO_TERM":LITHO_TERM,
            "DEP":DEP, "DEP_INBOX":DEP_INBOX, "DEP_FILES":DEP_FILES, "DEP_TERM":DEP_TERM,
            "ETCH":ETCH, "ETCH_INBOX":ETCH_INBOX, "ETCH_FILES":ETCH_FILES, "ETCH_TERM":ETCH_TERM
        },
        "programs":[
            rows(&aligner), rows(&engineer), rows(&wafer), rows(&deposition_chamber),
            rows(&etch_chamber), rows(&terminal), rows(&stepper), rows(&metrology), rows(&stage),
            rows(&p_litho), rows(&p_litho_inbox), rows(&p_litho_files), rows(&p_litho_term),
            rows(&p_dep), rows(&p_dep_inbox), rows(&p_dep_files), rows(&p_dep_term),
            rows(&p_etch), rows(&p_etch_inbox), rows(&p_etch_files), rows(&p_etch_term)
        ]
    });
    if path == "--projection" {
        println!("{projection}");
        return Ok(());
    }
    let module = std::fs::read(path)?;
    let session = SessionId::new(SESSION);
    let (mut connection, mut io, [shell, litho_shell, dep_shell, etch_shell]) = Bus::open(
        [
            "INTEGRATION_ENGINEER",
            "LITHOGRAPHER",
            "DEPOSITOR",
            "ETCHER",
        ],
        [
            (session, WAFER),
            (session, STAGE),
            (session, LITHO_INBOX),
            (session, DEP_INBOX),
            (session, ETCH_INBOX),
        ],
    );
    let (bus, effects) = connection.split();
    bus.projected(projection);
    let instrument = |path, fd: u8| {
        ChoreoFsObject::read_write(
            path,
            ObjectId(fd.into()),
            FdSpec::new(fd, 66),
            b"",
            FdBinding {
                read: Some(FdReadRow::Base),
                write: Some(FdWriteRow::Object),
                readdir: None,
            },
        )
    };
    let notice = |path, fd: u8| {
        ChoreoFsObject::writable(
            path,
            ObjectId(fd.into()),
            FdSpec::new(fd, 64),
            FdBinding::write(FdWriteRow::Object),
        )
    };
    let files = [
        instrument(b"service/run_deposition", 4),
        instrument(b"service/run_etch_process", 5),
        instrument(b"service/run_stepper_exposure", 6),
        instrument(b"service/run_metrology_measurement", 7),
        notice(b"service/lot_lithographer", 8),
        notice(b"service/lot_depositor", 9),
        notice(b"service/lot_etcher", 10),
    ];
    let capture = |row| bus.capture(row);
    let mut slab = vec![0; 1024 * 1024];
    let mut storage = SessionKitStorage::uninit();
    let rv = storage.init().rendezvous(&mut slab, &bus.transport)?;
    rv.observe(&capture);
    let tap = rv.tap();
    let trial = async {
        let mut aligner = rv.enter(session, &aligner)?;
        let mut engineer = rv.enter(session, &engineer)?;
        let wafer = rv.enter(session, &wafer)?;
        let mut dep = rv.enter(session, &deposition_chamber)?;
        let mut etch = rv.enter(session, &etch_chamber)?;
        let mut terminal = rv.enter(session, &terminal)?;
        let mut stepper = rv.enter(session, &stepper)?;
        let mut metrology = rv.enter(session, &metrology)?;
        let mut stage = rv.enter(session, &stage)?;
        let litho_ep = rv.enter(session, &p_litho)?;
        let mut litho_files_ep = rv.enter(session, &p_litho_files)?;
        let mut litho_term_ep = rv.enter(session, &p_litho_term)?;
        let dep_ep = rv.enter(session, &p_dep)?;
        let mut dep_files_ep = rv.enter(session, &p_dep_files)?;
        let mut dep_term_ep = rv.enter(session, &p_dep_term)?;
        let etch_ep = rv.enter(session, &p_etch)?;
        let mut etch_files_ep = rv.enter(session, &p_etch_files)?;
        let mut etch_term_ep = rv.enter(session, &p_etch_term)?;
        let litho_inbox_ep = rv.enter(session, &p_litho_inbox)?;
        let dep_inbox_ep = rv.enter(session, &p_dep_inbox)?;
        let etch_inbox_ep = rv.enter(session, &p_etch_inbox)?;
        async { fabrication_guest::<LITHO, LITHO_INBOX>(litho_ep, &module).await };
        async {
            localside::file_metadata::run(&mut litho_files_ep, litho_inbox_ep, [INBOX_FILE]).await
        };
        async {
            localside::private_io::run(
                &mut litho_term_ep,
                PrivateFs::new(litho_shell, bus, &[INBOX_FILE]),
            )
            .await
        };
        async { fabrication_guest::<DEP, DEP_INBOX>(dep_ep, &module).await };
        async {
            localside::file_metadata::run(&mut dep_files_ep, dep_inbox_ep, [INBOX_FILE]).await
        };
        async {
            localside::private_io::run(
                &mut dep_term_ep,
                PrivateFs::new(dep_shell, bus, &[INBOX_FILE]),
            )
            .await
        };
        async { fabrication_guest::<ETCH, ETCH_INBOX>(etch_ep, &module).await };
        async {
            localside::file_metadata::run(&mut etch_files_ep, etch_inbox_ep, [INBOX_FILE]).await
        };
        async {
            localside::private_io::run(
                &mut etch_term_ep,
                PrivateFs::new(etch_shell, bus, &[INBOX_FILE]),
            )
            .await
        };
        async {
            let mounted = effects.submit(stage.recv::<Mount>().await?)?;
            stepper.send::<Loan<STAGE>>(stage).await?;
            let [mut load, mut release] = stepper.par()?;
            async {
                let result = mounted.await?;
                load.send::<Mounted>(&encode(&result)?.as_slice()).await?;
                Ok(())
            };
            async {
                loop {
                    let report = release.offer().await?;
                    match report.label() {
                        OverlayFailed::LOGICAL_LABEL => {
                            let bytes = report.recv::<OverlayFailed>().await?.to_vec();
                            release.send::<Measured>(&bytes.as_slice()).await?;
                        }
                        OverlayPassed::LOGICAL_LABEL => {
                            let bytes = report.recv::<OverlayPassed>().await?.to_vec();
                            let _returned = release.recv::<Loan<STAGE>>().await?;
                            release.send::<Measured>(&bytes.as_slice()).await?;
                            return Ok(());
                        }
                        label => {
                            return Err(ShellError::message(format!(
                                "unexpected overlay result {label}"
                            )));
                        }
                    }
                }
            };
            Ok(())
        };
        async {
            let mut stage = metrology.recv::<Loan<STAGE>>().await?;
            loop {
                let actual = effects.apply(stage.recv::<Measure>().await?).await?;
                let bytes = encode(&actual)?;
                if actual["success"] == true {
                    stage
                        .send::<OverlayPassed>(&ReviewReply::from_bytes(&bytes)?)
                        .await?;
                    metrology.send::<Loan<STAGE>>(stage).await?;
                    return Ok(());
                }
                stage
                    .send::<OverlayFailed>(&ReviewReply::from_bytes(&bytes)?)
                    .await?;
            }
        };
        async {
            localside::file_metadata::run(&mut aligner, wafer, files).await?;
            let _returned = aligner.recv::<Loan<WAFER>>().await?;
            Ok(())
        };
        async { deposition(&mut dep, &effects).await };
        async { etching(&mut etch, &effects).await };
        async {
            localside::private_io::run(&mut terminal, PrivateFs::new(shell, bus, &files)).await
        };
        async {
            let wafer = engineer.recv::<Loan<WAFER>>().await?;
            let [
                mut film,
                mut film_result,
                mut etch,
                mut etch_result,
                mut mount,
                mut mounted,
                mut measure,
                mut measured,
                mut litho_note,
                mut dep_note,
                mut etch_note,
            ] = [(); 11].map(|_| vec![0; GUEST_MEMORY_BYTES]);
            let wafer = ChoreoFsFile::new(files[0], wafer)
                .write_as::<Film>(&mut film)
                .read_as::<FilmResult>(&mut film_result)
                .file(files[1])?
                .write_as::<Uniformity>(&mut etch)
                .read_as::<UniformityResult>(&mut etch_result)
                .file(files[2])?
                .write_as::<Mount>(&mut mount)
                .read_as::<Mounted>(&mut mounted)
                .file(files[3])?
                .write_as::<Measure>(&mut measure)
                .read_as::<Measured>(&mut measured)
                .file(files[4])?
                .write_as::<Disposition<LITHO_INBOX>>(&mut litho_note)
                .file(files[5])?
                .write_as::<Disposition<DEP_INBOX>>(&mut dep_note)
                .file(files[6])?
                .write_as::<Disposition<ETCH_INBOX>>(&mut etch_note);
            let bound = ChoreoFsFiles::new(engineer).file(wafer)?;
            let mut memory = vec![0; GUEST_MEMORY_BYTES];
            let mut storage = Box::new(HibanaWasiGuestStorage::uninit());
            let guest = storage.init(&module, GuestMemory::new(&mut memory), bound)?;
            let (exit, files) = guest.run().await?;
            let status = exit.status() as i32;
            let (mut engineer, (file, ())) = files.into_parts();
            let mut wafer = file.into_endpoint();
            wafer.send::<FileExit>(&status).await?;
            engineer.send::<Loan<WAFER>>(wafer).await?;
            engineer.send::<Exit>(&status).await?;
            Ok(())
        };
        Ok(())
    };
    let result = hibana_runtime::race(trial, io.run(bus)).await;
    bus.checkpoint(SESSION, tap.checkpoint());
    io.finish(bus, result, Some("reference_wafer_integration"))
        .await
}
