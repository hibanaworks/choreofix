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
    let source = g::seq(
        g::send::<ALIGNER, ENGINEER, Loan<WAFER>>(),
        g::par(
            g::seq(
                g::par(
                    g::seq(alignment, checks()),
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
    let projection = json!({"type":"compiled_projection","format_version":1,"task":"15E","session":SESSION,
        "roles":{"ALIGNER":ALIGNER,"ENGINEER":ENGINEER,"WAFER":WAFER,"DEPOSITION_CHAMBER":DEPOSITION_CHAMBER,
                 "ETCH_CHAMBER":ETCH_CHAMBER,"TERMINAL":TERMINAL,"STEPPER":STEPPER,"METROLOGY":METROLOGY,"STAGE":STAGE},
        "programs":[rows(&aligner),rows(&engineer),rows(&wafer),rows(&deposition_chamber),rows(&etch_chamber),rows(&terminal),
                    rows(&stepper),rows(&metrology),rows(&stage)]});
    if path == "--projection" {
        println!("{projection}");
        return Ok(());
    }
    let module = std::fs::read(path)?;
    let session = SessionId::new(SESSION);
    let (mut connection, mut io, [shell]) = Bus::open(
        ["INTEGRATION_ENGINEER"],
        [(session, WAFER), (session, STAGE)],
    );
    let (bus, effects) = connection.split();
    bus.projected(projection);
    let files = [
        (b"service/run_deposition".as_slice(), 4),
        (b"service/run_etch_process".as_slice(), 5),
        (b"service/run_stepper_exposure".as_slice(), 6),
        (b"service/run_metrology_measurement".as_slice(), 7),
    ]
    .map(|(path, fd)| {
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
    });
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
            ] = [(); 8].map(|_| vec![0; GUEST_MEMORY_BYTES]);
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
                .read_as::<Measured>(&mut measured);
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
