use choreofix::{
    imp::json::encode,
    imp::projection::rows,
    shell::{
        GUEST_MEMORY_BYTES, ShellError, ShellResult,
        global::{Exit, FileExit, Loan, file_metadata, private_io},
        imp::{
            bus::Bus,
            private_fs::PrivateFs,
            review_reply::ReviewReply,
            service::{ReceivedEffects, SimulatorOperation},
        },
        localside,
    },
};
use hibana::{
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
const MEASURER: u8 = 28;
const MEASURE_FILE: u8 = 29;
const MEASURE_FILES: u8 = 30;
const MEASURE_TERM: u8 = 31;
const ETCH_OPERATOR: u8 = 33;
const ETCH_STATION: u8 = 34;
type FabReady<'a, const FAB: u8> = g::Msg<FAB, &'a [u8]>;
type Confirmation<'a, const LABEL: u8> = g::Msg<LABEL, WaferReading<'a, MONITOR>>;
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

const READY_FILE: ChoreoFsObject<'static> = ChoreoFsObject::writable(
    b"service/fab_ready",
    ObjectId(6),
    FdSpec::new(6, 64),
    FdBinding::write(FdWriteRow::Object),
);

fn measure_wafer<const FAB: u8, const KIND: u8>() -> impl Projectable + Copy
where
    InstrumentRequest<KIND>: SimulatorOperation,
{
    g::seq(
        g::send::<FAB, MEASURE_FILE, FabReady<FAB>>(),
        g::seq(
            g::seq(
                g::send::<MEASURE_FILE, METROLOGY, Dimension<KIND>>(),
                g::send::<METROLOGY, MEASURE_FILE, Reading<KIND>>(),
            )
            .roll(),
            g::send::<MEASURE_FILE, CONTROL_FILE, MeasurementReport<KIND>>(),
        ),
    )
}

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
    let rate = g::send::<DEP_INBOX, CVD_STAGE, Rate>();
    let rate_result = g::send::<DEPOSITION_CHAMBER, DEP_INBOX, RateResult>();
    let stable = g::seq(
        g::send::<CVD_STAGE, DEPOSITION_CHAMBER, RatePassed>(),
        g::seq(
            g::send::<MONITOR, DEPOSITION_CHAMBER, Loan<CVD_STAGE>>(),
            rate_result,
        ),
    );
    let recheck = g::seq(
        g::send::<CVD_STAGE, DEPOSITION_CHAMBER, RateFailed>(),
        g::seq(rate_result, rate),
    );
    let growth = g::seq(
        g::send::<DEP_INBOX, CVD_STAGE, Growth>(),
        g::seq(
            g::send::<DEPOSITION_CHAMBER, MONITOR, Loan<CVD_STAGE>>(),
            g::par(
                g::send::<DEPOSITION_CHAMBER, DEP_INBOX, Grown>(),
                g::seq(rate, g::route(stable, g::seq(recheck.roll(), stable))),
            ),
        ),
    );
    let exposure = g::seq(
        g::send::<LITHO_INBOX, STAGE, Expose>(),
        g::send::<STAGE, LITHO_INBOX, Exposed>(),
    );
    let etch_request = g::send::<ETCH_INBOX, ETCH_CHAMBER, ReactiveEtch>();
    let etch_result = g::send::<ETCH_STATION, ETCH_INBOX, EtchResult>();
    let loaded = g::seq(
        g::send::<ETCH_CHAMBER, ETCH_STATION, EtchLoaded>(),
        g::seq(
            g::send::<ETCH_OPERATOR, ETCH_STATION, Loan<ETCH_CHAMBER>>(),
            etch_result,
        ),
    );
    let retry_etch = g::seq(
        g::send::<ETCH_CHAMBER, ETCH_STATION, EtchFailed>(),
        g::seq(etch_result, etch_request),
    );
    let reactive = g::seq(
        etch_request,
        g::route(loaded, g::seq(retry_etch.roll(), loaded)),
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
                    g::par(
                        g::send::<CONTROL_FILES, CONTROL, Loan<CONTROL_FILE>>(),
                        g::send::<MEASURE_FILES, MEASURER, Loan<MEASURE_FILE>>(),
                    ),
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
                g::par(
                    recipient_io::<CONTROL, CONTROL_FILE, CONTROL_FILES, CONTROL_TERM>(),
                    recipient_io::<MEASURER, MEASURE_FILE, MEASURE_FILES, MEASURE_TERM>(),
                ),
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
                            g::seq(
                                g::par(
                                    g::seq(exposure, measure_wafer::<LITHO_INBOX, CD>()),
                                    g::par(
                                        g::seq(growth, measure_wafer::<DEP_INBOX, THICKNESS>()),
                                        g::seq(reactive, measure_wafer::<ETCH_INBOX, PROFILE>()),
                                    ),
                                ),
                                validation,
                            ),
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
    let etch_operator = project::<ETCH_OPERATOR>(&source);
    let etch_station = project::<ETCH_STATION>(&source);
    let stepper = project::<STEPPER>(&source);
    let metrology = project::<METROLOGY>(&source);
    let stage = project::<STAGE>(&source);
    let cvd_stage = project::<CVD_STAGE>(&source);
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
    let p_measurer = project::<MEASURER>(&source);
    let p_measure_file = project::<MEASURE_FILE>(&source);
    let p_measure_files = project::<MEASURE_FILES>(&source);
    let p_measure_term = project::<MEASURE_TERM>(&source);
    let projection = json!({
        "type":"compiled_projection", "format_version":1, "task":"15E", "session":SESSION,
        "roles":{
            "ALIGNER":ALIGNER, "ENGINEER":ENGINEER, "WAFER":WAFER, "TERMINAL":TERMINAL,
            "DEPOSITION_CHAMBER":DEPOSITION_CHAMBER, "ETCH_CHAMBER":ETCH_CHAMBER,
            "STEPPER":STEPPER, "METROLOGY":METROLOGY, "STAGE":STAGE, "CVD_STAGE":CVD_STAGE,
            "LITHO":LITHO, "LITHO_INBOX":LITHO_INBOX, "LITHO_FILES":LITHO_FILES, "LITHO_TERM":LITHO_TERM,
            "DEP":DEP, "DEP_INBOX":DEP_INBOX, "DEP_FILES":DEP_FILES, "DEP_TERM":DEP_TERM,
            "ETCH":ETCH, "ETCH_INBOX":ETCH_INBOX, "ETCH_FILES":ETCH_FILES, "ETCH_TERM":ETCH_TERM,
            "ETCH_OPERATOR":ETCH_OPERATOR, "ETCH_STATION":ETCH_STATION,
            "MONITOR":MONITOR,"CONTROL_OPERATOR":CONTROL_OPERATOR,"MONITOR_STATION":MONITOR_STATION,"CONTROL":CONTROL,"CONTROL_FILE":CONTROL_FILE,"CONTROL_FILES":CONTROL_FILES,"CONTROL_TERM":CONTROL_TERM,
            "MEASURER":MEASURER,"MEASURE_FILE":MEASURE_FILE,"MEASURE_FILES":MEASURE_FILES,"MEASURE_TERM":MEASURE_TERM
        },
        "programs":[
            rows(&aligner), rows(&engineer), rows(&wafer), rows(&deposition_chamber),
            rows(&etch_chamber), rows(&etch_operator), rows(&etch_station), rows(&terminal), rows(&stepper), rows(&metrology), rows(&stage),
            rows(&p_litho), rows(&p_litho_inbox), rows(&p_litho_files), rows(&p_litho_term),
            rows(&p_dep), rows(&p_dep_inbox), rows(&p_dep_files), rows(&p_dep_term),
            rows(&p_etch), rows(&p_etch_inbox), rows(&p_etch_files), rows(&p_etch_term),rows(&p_monitor),rows(&p_control_operator),rows(&p_monitor_station),rows(&p_control),rows(&p_control_file),rows(&p_control_files),rows(&p_control_term),
            rows(&p_measurer), rows(&p_measure_file), rows(&p_measure_files), rows(&p_measure_term), rows(&cvd_stage)
        ]
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
            shell,
            litho_shell,
            dep_shell,
            etch_shell,
            control_shell,
            measure_shell,
        ],
    ) = Bus::open(
        [
            "INTEGRATION_ENGINEER",
            "LITHOGRAPHER",
            "DEPOSITOR",
            "ETCHER",
            "PROCESS_CONTROLLER",
            "METROLOGY_ENGINEER",
        ],
        [
            (session, WAFER),
            (session, STAGE),
            (session, CVD_STAGE),
            (session, LITHO_INBOX),
            (session, DEP_INBOX),
            (session, ETCH_INBOX),
            (session, ETCH_CHAMBER),
            (session, CONTROL_FILE),
            (session, MONITOR),
            (session, MEASURE_FILE),
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
    let readable = |path, fd: u8| {
        ChoreoFsObject::readable(
            path,
            ObjectId(fd.into()),
            FdSpec::new(fd, 2),
            b"",
            FdBinding::read(FdReadRow::Base),
        )
    };
    let litho_files = [
        INBOX_FILE,
        VALIDATION_FILE,
        READY_FILE,
        instrument(b"service/run_stepper_exposure", 7),
    ];
    let depositor_files = [
        INBOX_FILE,
        VALIDATION_FILE,
        READY_FILE,
        instrument(b"service/run_deposition", 7),
        instrument(b"service/check_monitoring", 8),
    ];
    let etcher_files = [
        INBOX_FILE,
        VALIDATION_FILE,
        READY_FILE,
        instrument(b"service/run_etch_process", 7),
    ];
    let measure_files = [
        instrument(b"service/pattern", 4),
        instrument(b"service/film", 5),
        instrument(b"service/etch", 6),
        instrument(b"service/cd_measurement", 7),
        instrument(b"service/thickness_measurement", 8),
        instrument(b"service/profile_measurement", 9),
    ];
    let control_files = [
        instrument(b"service/check_monitoring", 4),
        notice(b"service/validated_lithographer", 5),
        notice(b"service/validated_depositor", 6),
        notice(b"service/validated_etcher", 7),
        notice(b"service/all_validated", 8),
        readable(b"service/pattern_report", 9),
        readable(b"service/film_report", 10),
        readable(b"service/etch_report", 11),
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
        let mut etch_operator = rv.enter(session, &etch_operator)?;
        let mut etch = rv.enter(session, &etch_chamber)?;
        let mut etch_station = rv.enter(session, &etch_station)?;
        let mut terminal = rv.enter(session, &terminal)?;
        let mut stepper = rv.enter(session, &stepper)?;
        let mut metrology = rv.enter(session, &metrology)?;
        let mut stage = rv.enter(session, &stage)?;
        let mut cvd_stage = rv.enter(session, &cvd_stage)?;
        let mut operator = rv.enter(session, &p_control_operator)?;
        let mut monitor = rv.enter(session, &p_monitor)?;
        let mut station = rv.enter(session, &p_monitor_station)?;
        let litho_ep = rv.enter(session, &p_litho)?;
        let mut litho_files_ep = rv.enter(session, &p_litho_files)?;
        let mut litho_term_ep = rv.enter(session, &p_litho_term)?;
        let dep_ep = rv.enter(session, &p_dep)?;
        let mut dep_files_ep = rv.enter(session, &p_dep_files)?;
        let mut dep_term_ep = rv.enter(session, &p_dep_term)?;
        let mut etch_ep = rv.enter(session, &p_etch)?;
        let mut etch_files_ep = rv.enter(session, &p_etch_files)?;
        let mut etch_term_ep = rv.enter(session, &p_etch_term)?;
        let litho_inbox_ep = rv.enter(session, &p_litho_inbox)?;
        let dep_inbox_ep = rv.enter(session, &p_dep_inbox)?;
        let etch_inbox_ep = rv.enter(session, &p_etch_inbox)?;
        async {
            let mut vm = litho_ep;
            let inbox = vm.recv::<Loan<LITHO_INBOX>>().await?;
            let [
                mut disposition,
                mut validation,
                mut ready,
                mut expose,
                mut exposed,
            ] = [(); 5].map(|_| vec![0; GUEST_MEMORY_BYTES]);
            let file = ChoreoFsFile::new(INBOX_FILE, inbox)
                .read_as::<Disposition<LITHO_INBOX>>(&mut disposition)
                .file(VALIDATION_FILE)?
                .read_as::<Confirmation<CONFIRM_LITHO>>(&mut validation)
                .file(READY_FILE)?
                .write_as::<FabReady<LITHO_INBOX>>(&mut ready)
                .file(litho_files[3])?
                .write_as::<Expose>(&mut expose)
                .read_as::<Exposed>(&mut exposed);
            let files = ChoreoFsFiles::new(vm).file(file)?;
            let mut memory = vec![0; GUEST_MEMORY_BYTES];
            let mut storage = Box::new(HibanaWasiGuestStorage::uninit());
            let guest = storage.init(&module, GuestMemory::new(&mut memory), files)?;
            let (exit, files) = guest.run().await?;
            let status = exit.status() as i32;
            let (mut vm, (file, ())) = files.into_parts();
            file.into_endpoint().send::<FileExit>(&status).await?;
            vm.send::<Exit>(&status).await?;
            Ok(())
        };
        async {
            localside::file_metadata::run(&mut litho_files_ep, litho_inbox_ep, litho_files).await
        };
        async {
            localside::private_io::run(
                &mut litho_term_ep,
                PrivateFs::new(litho_shell, bus, &litho_files),
            )
            .await
        };
        async {
            let mut vm = dep_ep;
            let inbox = vm.recv::<Loan<DEP_INBOX>>().await?;
            let [
                mut disposition,
                mut validation,
                mut ready,
                mut grow,
                mut grown,
                mut rate,
                mut checked,
            ] = [(); 7].map(|_| vec![0; GUEST_MEMORY_BYTES]);
            let file = ChoreoFsFile::new(INBOX_FILE, inbox)
                .read_as::<Disposition<DEP_INBOX>>(&mut disposition)
                .file(VALIDATION_FILE)?
                .read_as::<Confirmation<CONFIRM_DEP>>(&mut validation)
                .file(READY_FILE)?
                .write_as::<FabReady<DEP_INBOX>>(&mut ready)
                .file(depositor_files[3])?
                .write_as::<Growth>(&mut grow)
                .read_as::<Grown>(&mut grown)
                .file(depositor_files[4])?
                .write_as::<Rate>(&mut rate)
                .read_as::<RateResult>(&mut checked);
            let files = ChoreoFsFiles::new(vm).file(file)?;
            let mut memory = vec![0; GUEST_MEMORY_BYTES];
            let mut storage = Box::new(HibanaWasiGuestStorage::uninit());
            let guest = storage.init(&module, GuestMemory::new(&mut memory), files)?;
            let (exit, files) = guest.run().await?;
            let status = exit.status() as i32;
            let (mut vm, (file, ())) = files.into_parts();
            file.into_endpoint().send::<FileExit>(&status).await?;
            vm.send::<Exit>(&status).await?;
            Ok(())
        };
        async {
            localside::file_metadata::run(&mut dep_files_ep, dep_inbox_ep, depositor_files).await
        };
        async {
            localside::private_io::run(
                &mut dep_term_ep,
                PrivateFs::new(dep_shell, bus, &depositor_files),
            )
            .await
        };
        async {
            let inbox = etch_ep.recv::<Loan<ETCH_INBOX>>().await?;
            let mut buffer = vec![0; GUEST_MEMORY_BYTES];
            let mut validation = vec![0; GUEST_MEMORY_BYTES];
            let mut ready = vec![0; GUEST_MEMORY_BYTES];
            let mut process = vec![0; GUEST_MEMORY_BYTES];
            let mut result = vec![0; GUEST_MEMORY_BYTES];
            let file = ChoreoFsFile::new(INBOX_FILE, inbox)
                .read_as::<Disposition<ETCH_INBOX>>(&mut buffer)
                .file(VALIDATION_FILE)?
                .read_as::<Confirmation<CONFIRM_ETCH>>(&mut validation)
                .file(READY_FILE)?
                .write_as::<FabReady<ETCH_INBOX>>(&mut ready)
                .file(etcher_files[3])?
                .write_as::<ReactiveEtch>(&mut process)
                .read_as::<EtchResult>(&mut result);
            let files = ChoreoFsFiles::new(etch_ep).file(file)?;
            let mut memory = vec![0; GUEST_MEMORY_BYTES];
            let mut storage = Box::new(HibanaWasiGuestStorage::uninit());
            let guest = storage.init(&module, GuestMemory::new(&mut memory), files)?;
            let (exit, files) = guest.run().await?;
            let status = exit.status() as i32;
            let (mut vm, (file, ())) = files.into_parts();
            let mut inbox = file.into_endpoint();
            inbox.send::<FileExit>(&status).await?;
            vm.send::<Exit>(&status).await?;
            Ok(())
        };
        async {
            localside::file_metadata::run(&mut etch_files_ep, etch_inbox_ep, etcher_files).await
        };
        async {
            localside::private_io::run(
                &mut etch_term_ep,
                PrivateFs::new(etch_shell, bus, &etcher_files),
            )
            .await
        };
        let mut measurer = rv.enter(session, &p_measurer)?;
        let mut measure_data = rv.enter(session, &p_measure_files)?;
        let measure_file = rv.enter(session, &p_measure_file)?;
        let mut measure_term = rv.enter(session, &p_measure_term)?;
        async {
            localside::file_metadata::run(&mut measure_data, measure_file, measure_files).await
        };
        async {
            localside::private_io::run(
                &mut measure_term,
                PrivateFs::new(measure_shell, bus, &measure_files),
            )
            .await
        };
        async {
            let file = measurer.recv::<Loan<MEASURE_FILE>>().await?;
            let [
                mut pattern,
                mut film,
                mut etch,
                mut pattern_report,
                mut film_report,
                mut etch_report,
            ] = [(); 6].map(|_| vec![0; GUEST_MEMORY_BYTES]);
            let [
                mut cd,
                mut thickness,
                mut profile,
                mut cd_reply,
                mut thickness_reply,
                mut profile_reply,
            ] = [(); 6].map(|_| vec![0; GUEST_MEMORY_BYTES]);
            let file = ChoreoFsFile::new(measure_files[0], file)
                .read_as::<FabReady<LITHO_INBOX>>(&mut pattern)
                .write_as::<MeasurementReport<CD>>(&mut pattern_report)
                .file(measure_files[1])?
                .read_as::<FabReady<DEP_INBOX>>(&mut film)
                .write_as::<MeasurementReport<THICKNESS>>(&mut film_report)
                .file(measure_files[2])?
                .read_as::<FabReady<ETCH_INBOX>>(&mut etch)
                .write_as::<MeasurementReport<PROFILE>>(&mut etch_report)
                .file(measure_files[3])?
                .write_as::<Dimension<CD>>(&mut cd)
                .read_as::<Reading<CD>>(&mut cd_reply)
                .file(measure_files[4])?
                .write_as::<Dimension<THICKNESS>>(&mut thickness)
                .read_as::<Reading<THICKNESS>>(&mut thickness_reply)
                .file(measure_files[5])?
                .write_as::<Dimension<PROFILE>>(&mut profile)
                .read_as::<Reading<PROFILE>>(&mut profile_reply);
            let files = ChoreoFsFiles::new(measurer).file(file)?;
            let mut memory = vec![0; GUEST_MEMORY_BYTES];
            let mut storage = Box::new(HibanaWasiGuestStorage::uninit());
            let guest = storage.init(&module, GuestMemory::new(&mut memory), files)?;
            let (exit, files) = guest.run().await?;
            let status = exit.status() as i32;
            let (mut measurer, (file, ())) = files.into_parts();
            file.into_endpoint().send::<FileExit>(&status).await?;
            measurer.send::<Exit>(&status).await?;
            Ok(())
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
            let [mut pattern, mut film, mut etch_report] =
                [(); 3].map(|_| vec![0; GUEST_MEMORY_BYTES]);
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
                .write_as::<Confirmation<ALL_VALIDATED>>(&mut all)
                .file(control_files[5])?
                .read_as::<MeasurementReport<CD>>(&mut pattern)
                .file(control_files[6])?
                .read_as::<MeasurementReport<THICKNESS>>(&mut film)
                .file(control_files[7])?
                .read_as::<MeasurementReport<PROFILE>>(&mut etch_report);
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
            let mut loaded = monitor.recv::<Loan<CVD_STAGE>>().await?;
            loop {
                let measured = effects.apply(loaded.recv::<Rate>().await?).await?;
                let bytes = encode(&measured)?;
                if measured["success"] == true {
                    loaded
                        .send::<RatePassed>(&ReviewReply::from_bytes(&bytes)?)
                        .await?;
                    monitor.send::<Loan<CVD_STAGE>>(loaded).await?;
                    break;
                }
                loaded
                    .send::<RateFailed>(&ReviewReply::from_bytes(&bytes)?)
                    .await?;
            }
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
            let exposed = effects.apply(stage.recv::<Expose>().await?).await?;
            stage.send::<Exposed>(&encode(&exposed)?.as_slice()).await?;
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
            let mut stage = loop {
                let request = metrology.offer().await?;
                match request.label() {
                    CD => {
                        let measured = effects
                            .apply(request.recv::<Dimension<CD>>().await?)
                            .await?;
                        metrology
                            .send::<Reading<CD>>(&encode(&measured)?.as_slice())
                            .await?;
                    }
                    THICKNESS => {
                        let measured = effects
                            .apply(request.recv::<Dimension<THICKNESS>>().await?)
                            .await?;
                        metrology
                            .send::<Reading<THICKNESS>>(&encode(&measured)?.as_slice())
                            .await?;
                    }
                    PROFILE => {
                        let measured = effects
                            .apply(request.recv::<Dimension<PROFILE>>().await?)
                            .await?;
                        metrology
                            .send::<Reading<PROFILE>>(&encode(&measured)?.as_slice())
                            .await?;
                    }
                    Loan::<STAGE>::LOGICAL_LABEL => break request.recv::<Loan<STAGE>>().await?,
                    label => {
                        return Err(ShellError::message(format!(
                            "unexpected metrology operation {label}"
                        )));
                    }
                }
            };
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
        async {
            let growth = effects.submit(cvd_stage.recv::<Growth>().await?)?;
            dep.send::<Loan<CVD_STAGE>>(cvd_stage).await?;
            async {
                let [mut load, mut release] = dep.par()?;
                async {
                    load.send::<Grown>(&encode(&growth.await?)?.as_slice())
                        .await?;
                    Ok(())
                };
                async {
                    loop {
                        let report = release.offer().await?;
                        match report.label() {
                            RateFailed::LOGICAL_LABEL => {
                                let bytes = report.recv::<RateFailed>().await?.to_vec();
                                release.send::<RateResult>(&bytes.as_slice()).await?;
                            }
                            RatePassed::LOGICAL_LABEL => {
                                let bytes = report.recv::<RatePassed>().await?.to_vec();
                                let _returned = release.recv::<Loan<CVD_STAGE>>().await?;
                                release.send::<RateResult>(&bytes.as_slice()).await?;
                                return Ok(());
                            }
                            label => {
                                return Err(ShellError::message(format!(
                                    "unexpected deposition rate result {label}"
                                )));
                            }
                        }
                    }
                };
                Ok(())
            }
            .await?;
            let actual = effects.apply(dep.recv::<Film>().await?).await?;
            dep.send::<FilmResult>(&encode(&actual)?.as_slice()).await?;
            Ok(())
        };
        async {
            loop {
                let actual = effects.apply(etch.recv::<ReactiveEtch>().await?).await?;
                let bytes = encode(&actual)?;
                if actual["success"] == true {
                    etch.send::<EtchLoaded>(&ReviewReply::from_bytes(&bytes)?)
                        .await?;
                    etch_operator.send::<Loan<ETCH_CHAMBER>>(etch).await?;
                    return Ok(());
                }
                etch.send::<EtchFailed>(&ReviewReply::from_bytes(&bytes)?)
                    .await?;
            }
        };
        async {
            let mut chamber = loop {
                let outcome = etch_station.offer().await?;
                match outcome.label() {
                    EtchFailed::LOGICAL_LABEL => {
                        let result = outcome.recv::<EtchFailed>().await?.to_vec();
                        etch_station.send::<EtchResult>(&result.as_slice()).await?;
                    }
                    EtchLoaded::LOGICAL_LABEL => {
                        let result = outcome.recv::<EtchLoaded>().await?.to_vec();
                        let chamber = etch_station.recv::<Loan<ETCH_CHAMBER>>().await?;
                        etch_station.send::<EtchResult>(&result.as_slice()).await?;
                        break chamber;
                    }
                    label => {
                        return Err(ShellError::message(format!(
                            "unexpected etch result {label}"
                        )));
                    }
                }
            };
            let actual = effects.apply(chamber.recv::<Uniformity>().await?).await?;
            chamber
                .send::<UniformityResult>(&encode(&actual)?.as_slice())
                .await?;
            Ok(())
        };
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
