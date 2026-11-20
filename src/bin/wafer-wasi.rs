use choreofix::{
    imp::projection::rows,
    shell::{
        GUEST_MEMORY_BYTES, ShellError, ShellResult,
        global::{Exit, FileExit, Loan, file_metadata, private_io},
        imp::{bus::Bus, private_fs::PrivateFs},
        localside,
    },
};
use hibana::{
    g,
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

#[path = "../tasks/wafer_fabrication/spot_checks.rs"]
mod spot_checks;
use spot_checks::*;

const TERMINAL: u8 = 5;
const SESSION: u32 = 150;

#[hibana_runtime::main]
async fn main() -> ShellResult<()> {
    let path = std::env::args()
        .nth(1)
        .ok_or_else(|| ShellError::message("usage: wafer-wasi GUEST.wasm"))?;
    let source = g::seq(
        g::send::<ALIGNER, ENGINEER, Loan<WAFER>>(),
        g::par(
            g::seq(
                g::par(
                    checks(),
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
    let terminal = project::<TERMINAL>(&source);
    let projection = json!({"type":"compiled_projection","format_version":1,"task":"15E","session":SESSION,
        "roles":{"ALIGNER":ALIGNER,"ENGINEER":ENGINEER,"WAFER":WAFER,"DEPOSITION_CHAMBER":DEPOSITION_CHAMBER,
                 "ETCH_CHAMBER":ETCH_CHAMBER,"TERMINAL":TERMINAL},
        "programs":[rows(&aligner),rows(&engineer),rows(&wafer),rows(&deposition_chamber),rows(&etch_chamber),rows(&terminal)]});
    if path == "--projection" {
        println!("{projection}");
        return Ok(());
    }
    let module = std::fs::read(path)?;
    let session = SessionId::new(SESSION);
    let (mut connection, mut io, [shell]) = Bus::open(["INTEGRATION_ENGINEER"], [(session, WAFER)]);
    let (bus, effects) = connection.split();
    bus.projected(projection);
    let files = [
        (b"service/run_deposition".as_slice(), 4),
        (b"service/run_etch_process".as_slice(), 5),
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
            let [mut film, mut film_result, mut etch, mut etch_result] =
                [(); 4].map(|_| vec![0; GUEST_MEMORY_BYTES]);
            let wafer = ChoreoFsFile::new(files[0], wafer)
                .write_as::<Film>(&mut film)
                .read_as::<FilmResult>(&mut film_result)
                .file(files[1])?
                .write_as::<Uniformity>(&mut etch)
                .read_as::<UniformityResult>(&mut etch_result);
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
    io.finish(bus, result, Some("aligned_wafer_spot_checks"))
        .await
}
