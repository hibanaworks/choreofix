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
const CONTROL: u8 = 22;
const CONTROL_FILE: u8 = 23;
const CONTROL_FILES: u8 = 24;
const CONTROL_TERM: u8 = 25;
const CONTROL_OPERATOR: u8 = 26;
const MONITOR_STATION: u8 = 27;
type Confirmation<'a, const LABEL: u8> = g::Msg<LABEL, &'a [u8]>;
const CONFIRM_LITHO: u8 = 40;
const CONFIRM_DEP: u8 = 41;
const CONFIRM_ETCH: u8 = 42;
const ALL_VALIDATED: u8 = 43;
type Disposition<'a, const INBOX: u8> = g::Msg<INBOX, &'a [u8]>;

const INBOX_FILE: ChoreoFsObject<'static> = ChoreoFsObject::readable(
    b"service/lot_disposition",
    ObjectId(4),
    FdSpec::new(4, 2),
    b"",
    FdBinding::read(FdReadRow::Base),
);

const VALIDATION_FILE: ChoreoFsObject<'static> = ChoreoFsObject::readable(
    b"service/process_validation",
    ObjectId(5),
    FdSpec::new(5, 2),
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

async fn fabrication_guest<'run, const VM: u8, const INBOX: u8, const VALIDATION: u8>(
    mut vm: Endpoint<'run, VM>,
    module: &[u8],
) -> ShellResult<()> {
    let inbox = vm.recv::<Loan<INBOX>>().await?;
    let mut buffer = vec![0; GUEST_MEMORY_BYTES];
    let mut validation = vec![0; GUEST_MEMORY_BYTES];
    let file = ChoreoFsFile::new(INBOX_FILE, inbox)
        .read_as::<Disposition<INBOX>>(&mut buffer)
        .file(VALIDATION_FILE)?
        .read_as::<Confirmation<VALIDATION>>(&mut validation);
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
    let check = g::send::<CONTROL_FILE, MONITOR, CrossReference>();
    let checked = g::send::<MONITOR_STATION, CONTROL_FILE, CrossReferenceResult>();
    let released = g::seq(
        g::send::<MONITOR, MONITOR_STATION, CrossReferencePassed>(),
        g::seq(
            g::send::<CONTROL_OPERATOR, MONITOR_STATION, Loan<MONITOR>>(),
            checked,
        ),
    );
    let validation = g::seq(
        check,
        g::route(
            released,
            g::seq(
                g::seq(
                    g::send::<MONITOR, MONITOR_STATION, CrossReferenceFailed>(),
                    g::seq(checked, check),
                )
                .roll(),
                released,
            ),
        ),
    );
    let confirmations = g::par(
        g::par(
            g::send::<CONTROL_FILE, LITHO_INBOX, Confirmation<CONFIRM_LITHO>>(),
            g::send::<CONTROL_FILE, DEP_INBOX, Confirmation<CONFIRM_DEP>>(),
        ),
        g::par(
            g::send::<CONTROL_FILE, ETCH_INBOX, Confirmation<CONFIRM_ETCH>>(),
            g::send::<CONTROL_FILE, WAFER, Confirmation<ALL_VALIDATED>>(),
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
                g::par(
                    g::send::<ETCH_FILES, ETCH, Loan<ETCH_INBOX>>(),
                    g::send::<CONTROL_FILES, CONTROL, Loan<CONTROL_FILE>>(),
                ),
            ),
        ),
    );
    let services = g::par(
        recipient_io::<LITHO, LITHO_INBOX, LITHO_FILES, LITHO_TERM>(),
        g::par(
            recipient_io::<DEP, DEP_INBOX, DEP_FILES, DEP_TERM>(),
            g::par(
                recipient_io::<ETCH, ETCH_INBOX, ETCH_FILES, ETCH_TERM>(),
                recipient_io::<CONTROL, CONTROL_FILE, CONTROL_FILES, CONTROL_TERM>(),
            ),
        ),
    );
    let source = g::seq(
        bootstrap,
        g::par(
            services,
            g::par(
                g::seq(
                    g::par(
                        g::seq(
                            validation,
                            g::seq(confirmations, g::seq(alignment, g::seq(checks(), notices))),
                        ),
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
    let p_monitor = project::<MONITOR>(&source);
    let p_control_operator = project::<CONTROL_OPERATOR>(&source);
    let p_monitor_station = project::<MONITOR_STATION>(&source);
    let p_control = project::<CONTROL>(&source);
    let p_control_file = project::<CONTROL_FILE>(&source);
    let p_control_files = project::<CONTROL_FILES>(&source);
    let p_control_term = project::<CONTROL_TERM>(&source);
    let projection = json!({
        "type":"compiled_projection", "format_version":1, "task":"15E", "session":SESSION,
        "roles":{
            "ALIGNER":ALIGNER, "ENGINEER":ENGINEER, "WAFER":WAFER, "TERMINAL":TERMINAL,
            "DEPOSITION_CHAMBER":DEPOSITION_CHAMBER, "ETCH_CHAMBER":ETCH_CHAMBER,
            "STEPPER":STEPPER, "METROLOGY":METROLOGY, "STAGE":STAGE,
            "LITHO":LITHO, "LITHO_INBOX":LITHO_INBOX, "LITHO_FILES":LITHO_FILES, "LITHO_TERM":LITHO_TERM,
            "DEP":DEP, "DEP_INBOX":DEP_INBOX, "DEP_FILES":DEP_FILES, "DEP_TERM":DEP_TERM,
            "ETCH":ETCH, "ETCH_INBOX":ETCH_INBOX, "ETCH_FILES":ETCH_FILES, "ETCH_TERM":ETCH_TERM,
            "MONITOR":MONITOR,"CONTROL_OPERATOR":CONTROL_OPERATOR,"MONITOR_STATION":MONITOR_STATION,"CONTROL":CONTROL,"CONTROL_FILE":CONTROL_FILE,"CONTROL_FILES":CONTROL_FILES,"CONTROL_TERM":CONTROL_TERM
        },
        "programs":[
            rows(&aligner), rows(&engineer), rows(&wafer), rows(&deposition_chamber),
            rows(&etch_chamber), rows(&terminal), rows(&stepper), rows(&metrology), rows(&stage),
            rows(&p_litho), rows(&p_litho_inbox), rows(&p_litho_files), rows(&p_litho_term),
            rows(&p_dep), rows(&p_dep_inbox), rows(&p_dep_files), rows(&p_dep_term),
            rows(&p_etch), rows(&p_etch_inbox), rows(&p_etch_files), rows(&p_etch_term),rows(&p_monitor),rows(&p_control_operator),rows(&p_monitor_station),rows(&p_control),rows(&p_control_file),rows(&p_control_files),rows(&p_control_term)
        ]
    });
    if path == "--projection" {
        println!("{projection}");
        return Ok(());
    }
    let module = std::fs::read(path)?;
    let session = SessionId::new(SESSION);
    let (mut connection, mut io, [shell, litho_shell, dep_shell, etch_shell, control_shell]) =
        Bus::open(
            [
                "INTEGRATION_ENGINEER",
                "LITHOGRAPHER",
                "DEPOSITOR",
                "ETCHER",
                "PROCESS_CONTROLLER",
            ],
            [
                (session, WAFER),
                (session, STAGE),
                (session, LITHO_INBOX),
                (session, DEP_INBOX),
                (session, ETCH_INBOX),
                (session, CONTROL_FILE),
                (session, MONITOR),
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
    let control_files = [
        instrument(b"service/check_monitoring", 4),
        notice(b"service/validated_lithographer", 5),
        notice(b"service/validated_depositor", 6),
        notice(b"service/validated_etcher", 7),
        notice(b"service/all_validated", 8),
    ];
    let files = [
        instrument(b"service/run_deposition", 4),
        instrument(b"service/run_etch_process", 5),
        instrument(b"service/run_stepper_exposure", 6),
        instrument(b"service/run_metrology_measurement", 7),
        notice(b"service/lot_lithographer", 8),
        notice(b"service/lot_depositor", 9),
        notice(b"service/lot_etcher", 10),
        ChoreoFsObject::readable(
            b"service/all_validated",
            ObjectId(11),
            FdSpec::new(11, 2),
            b"",
            FdBinding::read(FdReadRow::Base),
        ),
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
        let mut operator = rv.enter(session, &p_control_operator)?;
        let mut monitor = rv.enter(session, &p_monitor)?;
        let mut station = rv.enter(session, &p_monitor_station)?;
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
        async { fabrication_guest::<LITHO, LITHO_INBOX, CONFIRM_LITHO>(litho_ep, &module).await };
        async {
            localside::file_metadata::run(
                &mut litho_files_ep,
                litho_inbox_ep,
                [INBOX_FILE, VALIDATION_FILE],
            )
            .await
        };
        async {
            localside::private_io::run(
                &mut litho_term_ep,
                PrivateFs::new(litho_shell, bus, &[INBOX_FILE, VALIDATION_FILE]),
            )
            .await
        };
        async { fabrication_guest::<DEP, DEP_INBOX, CONFIRM_DEP>(dep_ep, &module).await };
        async {
            localside::file_metadata::run(
                &mut dep_files_ep,
                dep_inbox_ep,
                [INBOX_FILE, VALIDATION_FILE],
            )
            .await
        };
        async {
            localside::private_io::run(
                &mut dep_term_ep,
                PrivateFs::new(dep_shell, bus, &[INBOX_FILE, VALIDATION_FILE]),
            )
            .await
        };
        async { fabrication_guest::<ETCH, ETCH_INBOX, CONFIRM_ETCH>(etch_ep, &module).await };
        async {
            localside::file_metadata::run(
                &mut etch_files_ep,
                etch_inbox_ep,
                [INBOX_FILE, VALIDATION_FILE],
            )
            .await
        };
        async {
            localside::private_io::run(
                &mut etch_term_ep,
                PrivateFs::new(etch_shell, bus, &[INBOX_FILE, VALIDATION_FILE]),
            )
            .await
        };
        let mut control = rv.enter(session, &p_control)?;
        let mut control_data = rv.enter(session, &p_control_files)?;
        let control_file = rv.enter(session, &p_control_file)?;
        let mut control_term = rv.enter(session, &p_control_term)?;
        async {
            localside::file_metadata::run(&mut control_data, control_file, control_files).await
        };
        async {
            localside::private_io::run(
                &mut control_term,
                PrivateFs::new(control_shell, bus, &control_files),
            )
            .await
        };
        async {
            let file = control.recv::<Loan<CONTROL_FILE>>().await?;
            let [mut check, mut result, mut litho, mut dep, mut etch, mut all] =
                [(); 6].map(|_| vec![0; GUEST_MEMORY_BYTES]);
            let file = ChoreoFsFile::new(control_files[0], file)
                .write_as::<CrossReference>(&mut check)
                .read_as::<CrossReferenceResult>(&mut result)
                .file(control_files[1])?
                .write_as::<Confirmation<CONFIRM_LITHO>>(&mut litho)
                .file(control_files[2])?
                .write_as::<Confirmation<CONFIRM_DEP>>(&mut dep)
                .file(control_files[3])?
                .write_as::<Confirmation<CONFIRM_ETCH>>(&mut etch)
                .file(control_files[4])?
                .write_as::<Confirmation<ALL_VALIDATED>>(&mut all);
            let files = ChoreoFsFiles::new(control).file(file)?;
            let mut memory = vec![0; GUEST_MEMORY_BYTES];
            let mut storage = Box::new(HibanaWasiGuestStorage::uninit());
            let guest = storage.init(&module, GuestMemory::new(&mut memory), files)?;
            let (exit, files) = guest.run().await?;
            let status = exit.status() as i32;
            let (mut control, (file, ())) = files.into_parts();
            file.into_endpoint().send::<FileExit>(&status).await?;
            control.send::<Exit>(&status).await?;
            Ok(())
        };
        async {
            loop {
                let actual = effects
                    .apply(monitor.recv::<CrossReference>().await?)
                    .await?;
                let bytes = encode(&actual)?;
                if actual["success"] == true {
                    monitor
                        .send::<CrossReferencePassed>(&ReviewReply::from_bytes(&bytes)?)
                        .await?;
                    operator.send::<Loan<MONITOR>>(monitor).await?;
                    return Ok(());
                }
                monitor
                    .send::<CrossReferenceFailed>(&ReviewReply::from_bytes(&bytes)?)
                    .await?;
            }
        };
        async {
            loop {
                let report = station.offer().await?;
                match report.label() {
                    CrossReferenceFailed::LOGICAL_LABEL => {
                        let bytes = report.recv::<CrossReferenceFailed>().await?.to_vec();
                        station
                            .send::<CrossReferenceResult>(&bytes.as_slice())
                            .await?;
                    }
                    CrossReferencePassed::LOGICAL_LABEL => {
                        let bytes = report.recv::<CrossReferencePassed>().await?.to_vec();
                        let _released = station.recv::<Loan<MONITOR>>().await?;
                        station
                            .send::<CrossReferenceResult>(&bytes.as_slice())
                            .await?;
                        return Ok(());
                    }
                    label => {
                        return Err(ShellError::message(format!(
                            "unexpected monitor result {label}"
                        )));
                    }
                }
            }
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
                mut validated,
            ] = [(); 12].map(|_| vec![0; GUEST_MEMORY_BYTES]);
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
                .write_as::<Disposition<ETCH_INBOX>>(&mut etch_note)
                .file(files[7])?
                .read_as::<Confirmation<ALL_VALIDATED>>(&mut validated);
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
