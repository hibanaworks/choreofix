//! Run all seven Wasm guests against the same source and transport.
use super::{files, global::*, preparation, publication, quality};
use crate::{
    imp::projection::rows,
    shell::{
        ShellResult,
        global::Loan,
        imp::{bus::Bus, private_fs::PrivateFs},
        localside::{file_metadata, private_io},
    },
};
use hibana::runtime::{SessionKitStorage, ids::SessionId, program::project};
use serde_json::json;

#[hibana_runtime::scope]
pub async fn run(module: Option<&[u8]>) -> ShellResult<()> {
    let source = wasi_task();
    let analyst_program = project::<ANALYST>(&source);
    let analyst_data_program = project::<ANALYST_DATA>(&source);
    let analyst_file_program = project::<ANALYST_FILE>(&source);
    let analyst_io_program = project::<ANALYST_IO>(&source);
    let analyst_vm_program = project::<ANALYST_VM>(&source);
    let author_a_program = project::<AUTHOR_A>(&source);
    let author_b_program = project::<AUTHOR_B>(&source);
    let author_c_program = project::<AUTHOR_C>(&source);
    let author_d_program = project::<AUTHOR_D>(&source);
    let author_data_a_program = project::<AUTHOR_DATA_A>(&source);
    let author_data_b_program = project::<AUTHOR_DATA_B>(&source);
    let author_data_c_program = project::<AUTHOR_DATA_C>(&source);
    let author_data_d_program = project::<AUTHOR_DATA_D>(&source);
    let author_file_a_program = project::<AUTHOR_FILE_A>(&source);
    let author_file_b_program = project::<AUTHOR_FILE_B>(&source);
    let author_file_c_program = project::<AUTHOR_FILE_C>(&source);
    let author_file_d_program = project::<AUTHOR_FILE_D>(&source);
    let author_io_a_program = project::<AUTHOR_IO_A>(&source);
    let author_io_b_program = project::<AUTHOR_IO_B>(&source);
    let author_io_c_program = project::<AUTHOR_IO_C>(&source);
    let author_io_d_program = project::<AUTHOR_IO_D>(&source);
    let author_vm_a_program = project::<AUTHOR_VM_A>(&source);
    let author_vm_b_program = project::<AUTHOR_VM_B>(&source);
    let author_vm_c_program = project::<AUTHOR_VM_C>(&source);
    let author_vm_d_program = project::<AUTHOR_VM_D>(&source);
    let database_program = project::<DATABASE>(&source);
    let editor_program = project::<EDITOR>(&source);
    let editor_data_program = project::<EDITOR_DATA>(&source);
    let editor_file_program = project::<EDITOR_FILE>(&source);
    let editor_io_program = project::<EDITOR_IO>(&source);
    let editor_vm_program = project::<EDITOR_VM>(&source);
    let figures_program = project::<FIGURES>(&source);
    let reviewer_program = project::<REVIEWER>(&source);
    let review_data_program = project::<REVIEW_DATA>(&source);
    let review_file_program = project::<REVIEW_FILE>(&source);
    let review_io_program = project::<REVIEW_IO>(&source);
    let review_vm_program = project::<REVIEW_VM>(&source);
    let section_a_program = project::<SECTION_A>(&source);
    let section_b_program = project::<SECTION_B>(&source);
    let section_c_program = project::<SECTION_C>(&source);
    let section_d_program = project::<SECTION_D>(&source);
    let roles = [
        ("DATA_ANALYST", ANALYST),
        ("ANALYST_DATA", ANALYST_DATA),
        ("ANALYST_FILE", ANALYST_FILE),
        ("ANALYST_IO", ANALYST_IO),
        ("ANALYST_VM", ANALYST_VM),
        ("RESEARCHER_A", AUTHOR_A),
        ("RESEARCHER_B", AUTHOR_B),
        ("RESEARCHER_C", AUTHOR_C),
        ("RESEARCHER_D", AUTHOR_D),
        ("AUTHOR_DATA_A", AUTHOR_DATA_A),
        ("AUTHOR_DATA_B", AUTHOR_DATA_B),
        ("AUTHOR_DATA_C", AUTHOR_DATA_C),
        ("AUTHOR_DATA_D", AUTHOR_DATA_D),
        ("AUTHOR_FILE_A", AUTHOR_FILE_A),
        ("AUTHOR_FILE_B", AUTHOR_FILE_B),
        ("AUTHOR_FILE_C", AUTHOR_FILE_C),
        ("AUTHOR_FILE_D", AUTHOR_FILE_D),
        ("AUTHOR_IO_A", AUTHOR_IO_A),
        ("AUTHOR_IO_B", AUTHOR_IO_B),
        ("AUTHOR_IO_C", AUTHOR_IO_C),
        ("AUTHOR_IO_D", AUTHOR_IO_D),
        ("AUTHOR_VM_A", AUTHOR_VM_A),
        ("AUTHOR_VM_B", AUTHOR_VM_B),
        ("AUTHOR_VM_C", AUTHOR_VM_C),
        ("AUTHOR_VM_D", AUTHOR_VM_D),
        ("DATABASE", DATABASE),
        ("EDITOR_IN_CHIEF", EDITOR),
        ("EDITOR_DATA", EDITOR_DATA),
        ("EDITOR_FILE", EDITOR_FILE),
        ("EDITOR_IO", EDITOR_IO),
        ("EDITOR_VM", EDITOR_VM),
        ("FIGURES", FIGURES),
        ("REVIEWER", REVIEWER),
        ("REVIEW_DATA", REVIEW_DATA),
        ("REVIEW_FILE", REVIEW_FILE),
        ("REVIEW_IO", REVIEW_IO),
        ("REVIEW_VM", REVIEW_VM),
        ("SECTION_A", SECTION_A),
        ("SECTION_B", SECTION_B),
        ("SECTION_C", SECTION_C),
        ("SECTION_D", SECTION_D),
    ]
    .into_iter()
    .collect::<std::collections::BTreeMap<_, _>>();
    let programs = [
        rows(&analyst_program),
        rows(&analyst_data_program),
        rows(&analyst_file_program),
        rows(&analyst_io_program),
        rows(&analyst_vm_program),
        rows(&author_a_program),
        rows(&author_b_program),
        rows(&author_c_program),
        rows(&author_d_program),
        rows(&author_data_a_program),
        rows(&author_data_b_program),
        rows(&author_data_c_program),
        rows(&author_data_d_program),
        rows(&author_file_a_program),
        rows(&author_file_b_program),
        rows(&author_file_c_program),
        rows(&author_file_d_program),
        rows(&author_io_a_program),
        rows(&author_io_b_program),
        rows(&author_io_c_program),
        rows(&author_io_d_program),
        rows(&author_vm_a_program),
        rows(&author_vm_b_program),
        rows(&author_vm_c_program),
        rows(&author_vm_d_program),
        rows(&database_program),
        rows(&editor_program),
        rows(&editor_data_program),
        rows(&editor_file_program),
        rows(&editor_io_program),
        rows(&editor_vm_program),
        rows(&figures_program),
        rows(&reviewer_program),
        rows(&review_data_program),
        rows(&review_file_program),
        rows(&review_io_program),
        rows(&review_vm_program),
        rows(&section_a_program),
        rows(&section_b_program),
        rows(&section_c_program),
        rows(&section_d_program),
    ];
    let projection = json!({"type":"compiled_projection","format_version":1,"task":"3H","session":SESSION,"roles":roles,"programs":programs.as_slice()});
    let Some(module) = module else {
        println!("{projection}");
        return Ok(());
    };
    let session = SessionId::new(SESSION);
    let (mut connection, mut io, terminals) = Bus::open(
        [
            "RESEARCHER_A",
            "RESEARCHER_B",
            "RESEARCHER_C",
            "RESEARCHER_D",
            "DATA_ANALYST",
            "REVIEWER",
            "EDITOR_IN_CHIEF",
        ],
        [
            SECTION_A,
            SECTION_B,
            SECTION_C,
            SECTION_D,
            DATABASE,
            FIGURES,
            AUTHOR_FILE_A,
            AUTHOR_FILE_B,
            AUTHOR_FILE_C,
            AUTHOR_FILE_D,
            ANALYST_FILE,
            REVIEW_FILE,
            EDITOR_FILE,
        ]
        .map(|role| (session, role)),
    );
    let (bus, simulator) = connection.split();
    let [
        author_a_terminal,
        author_b_terminal,
        author_c_terminal,
        author_d_terminal,
        analyst_terminal,
        reviewer_terminal,
        editor_terminal,
    ] = terminals;
    let participants = async {
        bus.projected(projection);
        let capture = |event| {
            bus.capture(event);
        };
        let mut slab = vec![0; 2 * 1024 * 1024];
        let mut storage = SessionKitStorage::uninit();
        let rv = storage.init().rendezvous(&mut slab, &bus.transport)?;
        rv.observe(&capture);
        let observations = rv.tap();
        let result = async {
            let mut analyst = rv.enter(session, &analyst_program)?;
            let mut analyst_data = rv.enter(session, &analyst_data_program)?;
            let mut analyst_io = rv.enter(session, &analyst_io_program)?;
            let analyst_vm = rv.enter(session, &analyst_vm_program)?;
            let mut author_a = rv.enter(session, &author_a_program)?;
            let mut author_b = rv.enter(session, &author_b_program)?;
            let mut author_c = rv.enter(session, &author_c_program)?;
            let mut author_d = rv.enter(session, &author_d_program)?;
            let mut author_data_a = rv.enter(session, &author_data_a_program)?;
            let mut author_data_b = rv.enter(session, &author_data_b_program)?;
            let mut author_data_c = rv.enter(session, &author_data_c_program)?;
            let mut author_data_d = rv.enter(session, &author_data_d_program)?;
            let mut author_io_a = rv.enter(session, &author_io_a_program)?;
            let mut author_io_b = rv.enter(session, &author_io_b_program)?;
            let mut author_io_c = rv.enter(session, &author_io_c_program)?;
            let mut author_io_d = rv.enter(session, &author_io_d_program)?;
            let author_vm_a = rv.enter(session, &author_vm_a_program)?;
            let author_vm_b = rv.enter(session, &author_vm_b_program)?;
            let author_vm_c = rv.enter(session, &author_vm_c_program)?;
            let author_vm_d = rv.enter(session, &author_vm_d_program)?;
            let mut database = rv.enter(session, &database_program)?;
            let mut editor = rv.enter(session, &editor_program)?;
            let mut editor_data = rv.enter(session, &editor_data_program)?;
            let mut editor_io = rv.enter(session, &editor_io_program)?;
            let editor_vm = rv.enter(session, &editor_vm_program)?;
            let mut figures = rv.enter(session, &figures_program)?;
            let mut reviewer = rv.enter(session, &reviewer_program)?;
            let mut review_data = rv.enter(session, &review_data_program)?;
            let mut review_io = rv.enter(session, &review_io_program)?;
            let review_vm = rv.enter(session, &review_vm_program)?;
            let author_file_a = rv.enter(session, &author_file_a_program)?;
            let author_file_b = rv.enter(session, &author_file_b_program)?;
            let author_file_c = rv.enter(session, &author_file_c_program)?;
            let author_file_d = rv.enter(session, &author_file_d_program)?;
            let analyst_file = rv.enter(session, &analyst_file_program)?;
            let review_file = rv.enter(session, &review_file_program)?;
            let editor_file = rv.enter(session, &editor_file_program)?;
            let section_a = rv.enter(session, &section_a_program)?;
            let section_b = rv.enter(session, &section_b_program)?;
            let section_c = rv.enter(session, &section_c_program)?;
            let section_d = rv.enter(session, &section_d_program)?;
            async {
                preparation::references(&mut database, &simulator).await?;
            };
            async {
                preparation::figures(&mut figures, &simulator).await?;
            };

            async {
                file_metadata::run(&mut author_data_a, author_file_a, files::AUTHOR_OBJECTS)
                    .await?;
            };
            async {
                private_io::run(
                    &mut author_io_a,
                    PrivateFs::new(author_a_terminal, bus, &files::AUTHOR_OBJECTS),
                )
                .await?;
            };
            async {
                let file = author_a.recv::<Loan<AUTHOR_FILE_A>>().await?;
                async {
                    quality::author::<SECTION_A, AUTHOR_A>(&mut author_a, section_a, &simulator)
                        .await?;
                };
                async {
                    files::author::<SECTION_A, AUTHOR_FILE_A, AUTHOR_VM_A>(
                        module,
                        file,
                        author_vm_a,
                    )
                    .await?;
                };
                Ok(())
            };

            async {
                file_metadata::run(&mut author_data_b, author_file_b, files::AUTHOR_OBJECTS)
                    .await?;
            };
            async {
                private_io::run(
                    &mut author_io_b,
                    PrivateFs::new(author_b_terminal, bus, &files::AUTHOR_OBJECTS),
                )
                .await?;
            };
            async {
                let file = author_b.recv::<Loan<AUTHOR_FILE_B>>().await?;
                async {
                    quality::author::<SECTION_B, AUTHOR_B>(&mut author_b, section_b, &simulator)
                        .await?;
                };
                async {
                    files::author::<SECTION_B, AUTHOR_FILE_B, AUTHOR_VM_B>(
                        module,
                        file,
                        author_vm_b,
                    )
                    .await?;
                };
                Ok(())
            };

            async {
                file_metadata::run(&mut author_data_c, author_file_c, files::AUTHOR_OBJECTS)
                    .await?;
            };
            async {
                private_io::run(
                    &mut author_io_c,
                    PrivateFs::new(author_c_terminal, bus, &files::AUTHOR_OBJECTS),
                )
                .await?;
            };
            async {
                let file = author_c.recv::<Loan<AUTHOR_FILE_C>>().await?;
                async {
                    quality::author::<SECTION_C, AUTHOR_C>(&mut author_c, section_c, &simulator)
                        .await?;
                };
                async {
                    files::author::<SECTION_C, AUTHOR_FILE_C, AUTHOR_VM_C>(
                        module,
                        file,
                        author_vm_c,
                    )
                    .await?;
                };
                Ok(())
            };

            async {
                file_metadata::run(&mut author_data_d, author_file_d, files::AUTHOR_OBJECTS)
                    .await?;
            };
            async {
                private_io::run(
                    &mut author_io_d,
                    PrivateFs::new(author_d_terminal, bus, &files::AUTHOR_OBJECTS),
                )
                .await?;
            };
            async {
                let file = author_d.recv::<Loan<AUTHOR_FILE_D>>().await?;
                async {
                    quality::author::<SECTION_D, AUTHOR_D>(&mut author_d, section_d, &simulator)
                        .await?;
                };
                async {
                    files::author::<SECTION_D, AUTHOR_FILE_D, AUTHOR_VM_D>(
                        module,
                        file,
                        author_vm_d,
                    )
                    .await?;
                };
                Ok(())
            };

            async {
                file_metadata::run(&mut analyst_data, analyst_file, files::ANALYST_OBJECTS).await?;
            };
            async {
                private_io::run(
                    &mut analyst_io,
                    PrivateFs::new(analyst_terminal, bus, &files::ANALYST_OBJECTS),
                )
                .await?;
            };
            async {
                let file = analyst.recv::<Loan<ANALYST_FILE>>().await?;
                async {
                    let [mut a, mut b, mut c, mut d] = analyst.par()?;
                    async {
                        preparation::insert::<SECTION_A>(&mut a, &simulator).await?;
                    };
                    async {
                        preparation::insert::<SECTION_B>(&mut b, &simulator).await?;
                    };
                    async {
                        preparation::insert::<SECTION_C>(&mut c, &simulator).await?;
                    };
                    async {
                        preparation::insert::<SECTION_D>(&mut d, &simulator).await?;
                    };
                    Ok::<(), crate::shell::ShellError>(())
                };
                async {
                    files::analyst(module, file, analyst_vm).await?;
                };
                Ok(())
            };

            async {
                file_metadata::run(&mut review_data, review_file, files::REVIEW_OBJECTS).await?;
            };
            async {
                private_io::run(
                    &mut review_io,
                    PrivateFs::new(reviewer_terminal, bus, &files::REVIEW_OBJECTS),
                )
                .await?;
            };
            async {
                let file = reviewer.recv::<Loan<REVIEW_FILE>>().await?;
                async {
                    let [mut a, mut b, mut c, mut d] = reviewer.par()?;
                    async {
                        quality::review::<SECTION_A>(&mut a, &simulator).await?;
                    };
                    async {
                        quality::review::<SECTION_B>(&mut b, &simulator).await?;
                    };
                    async {
                        quality::review::<SECTION_C>(&mut c, &simulator).await?;
                    };
                    async {
                        quality::review::<SECTION_D>(&mut d, &simulator).await?;
                    };
                    Ok::<(), crate::shell::ShellError>(())
                };
                async {
                    files::review(module, file, review_vm).await?;
                };
                Ok(())
            };

            async {
                file_metadata::run(&mut editor_data, editor_file, EDITOR_OBJECTS).await?;
            };
            async {
                private_io::run(
                    &mut editor_io,
                    PrivateFs::new(editor_terminal, bus, &EDITOR_OBJECTS),
                )
                .await?;
            };
            async {
                let file = editor.recv::<Loan<EDITOR_FILE>>().await?;
                async {
                    publication::run(&mut editor, &simulator).await?;
                };
                async {
                    super::editor_guest(module, file, editor_vm).await?;
                };
                Ok(())
            };
            Ok(())
        }
        .await;
        bus.checkpoint(SESSION, observations.checkpoint());
        result
    };
    let result = hibana_runtime::race(participants, io.run(bus)).await;
    let message = match &result {
        Ok(()) => json!({"type":"run_result","status":"complete"}),
        Err(error) => json!({"type":"run_result","status":"failed","error":error.to_string()}),
    };
    io.emit(bus, message).await?;
    result
}
