//! 3M: independent affine chapters, two physical I/O owners and checked publication.
//! Each guest mounts the operation files on its owned Endpoint.
pub mod global;
pub(crate) mod imp;
pub mod localside;

use crate::shell::{
    GUEST_MEMORY_BYTES, ShellResult,
    imp::{bus::Bus, private_fs::PrivateFs},
    localside::{file_metadata, private_io},
};
use global::*;
use hibana::runtime::{SessionKitStorage, ids::SessionId};
use hibana_wasip1_runtime::{ChoreoFsFile, ChoreoFsFiles, GuestMemory, HibanaWasiGuestStorage};
use serde_json::json;

#[hibana_runtime::scope]
pub async fn run_stdio(module: &[u8]) -> ShellResult<()> {
    let session = SessionId::new(SESSION);
    let (
        mut connection,
        mut io,
        [
            author_a_terminal,
            author_b_terminal,
            author_c_terminal,
            fact_terminal,
            editor_terminal,
        ],
    ) = Bus::open(
        [
            "RESEARCHER_A",
            "RESEARCHER_B",
            "RESEARCHER_C",
            "FACT_CHECKER",
            "EDITOR",
        ],
        [
            DOCUMENT,
            DATABASE,
            SECTION_A,
            SECTION_B,
            SECTION_C,
            AUTHOR_A_FILE,
            AUTHOR_B_FILE,
            AUTHOR_C_FILE,
            FACT_FILE,
            EDITOR_FILE,
            AUTHOR_A_VM,
            AUTHOR_B_VM,
            AUTHOR_C_VM,
            FACT_VM,
            EDITOR_VM,
        ]
        .map(|role| (session, role)),
    );
    let (bus, simulator) = connection.split();
    let participants = async {
        let programs = programs();
        bus.projected(programs.observation());
        let capture = |event| {
            bus.capture(event);
        };
        let mut slab = vec![0; 2 * 1024 * 1024];
        let incoming_capacity = slab.len();
        let mut storage = SessionKitStorage::uninit();
        let rv = storage.init().rendezvous(&mut slab, &bus.transport)?;
        rv.observe(&capture);
        let observations = rv.tap();
        let result = async {
            let mut init = rv.enter(session, &programs.init)?;
            let mut research_service = rv.enter(session, &programs.research_service)?;
            let mut review_service = rv.enter(session, &programs.review_service)?;
            let mut archive = rv.enter(session, &programs.archive)?;
            let mut author_a = rv.enter(session, &programs.author_a)?;
            let mut author_b = rv.enter(session, &programs.author_b)?;
            let mut author_c = rv.enter(session, &programs.author_c)?;
            let mut fact = rv.enter(session, &programs.fact)?;
            let mut editor = rv.enter(session, &programs.editor)?;
            let mut author_a_data = rv.enter(session, &programs.author_a_data)?;
            let mut author_b_data = rv.enter(session, &programs.author_b_data)?;
            let mut author_c_data = rv.enter(session, &programs.author_c_data)?;
            let mut fact_data = rv.enter(session, &programs.fact_data)?;
            let mut editor_data = rv.enter(session, &programs.editor_data)?;
            let mut author_a_io = rv.enter(session, &programs.author_a_io)?;
            let mut author_b_io = rv.enter(session, &programs.author_b_io)?;
            let mut author_c_io = rv.enter(session, &programs.author_c_io)?;
            let mut fact_io = rv.enter(session, &programs.fact_io)?;
            let mut editor_io = rv.enter(session, &programs.editor_io)?;
            let document = rv.enter(session, &programs.document)?;
            let database = rv.enter(session, &programs.database)?;
            let section_a = rv.enter(session, &programs.section_a)?;
            let section_b = rv.enter(session, &programs.section_b)?;
            let section_c = rv.enter(session, &programs.section_c)?;
            let author_a_file = rv.enter(session, &programs.author_a_file)?;
            let author_b_file = rv.enter(session, &programs.author_b_file)?;
            let author_c_file = rv.enter(session, &programs.author_c_file)?;
            let fact_file = rv.enter(session, &programs.fact_file)?;
            let editor_file = rv.enter(session, &programs.editor_file)?;
            let author_a_vm = rv.enter(session, &programs.author_a_vm)?;
            let author_b_vm = rv.enter(session, &programs.author_b_vm)?;
            let author_c_vm = rv.enter(session, &programs.author_c_vm)?;
            let fact_vm = rv.enter(session, &programs.fact_vm)?;
            let editor_vm = rv.enter(session, &programs.editor_vm)?;

            async {
                let [mut a, mut b, mut c, mut fact, mut editor] = init.par()?;
                async {
                    a.send::<Loan<AUTHOR_A_VM>>(author_a_vm).await?;
                };
                async {
                    b.send::<Loan<AUTHOR_B_VM>>(author_b_vm).await?;
                };
                async {
                    c.send::<Loan<AUTHOR_C_VM>>(author_c_vm).await?;
                };
                async {
                    fact.send::<Loan<FACT_VM>>(fact_vm).await?;
                };
                async {
                    editor.send::<Loan<EDITOR_VM>>(editor_vm).await?;
                };
                Ok(())
            };
            async {
                localside::document::run(document, &simulator).await?;
            };
            async {
                localside::database::run(database, &simulator).await?;
            };
            async {
                let [mut a, mut b, mut c, mut extra_a, mut extra_b, mut extra_c] =
                    research_service.par()?;
                async {
                    localside::research::run::<AUTHOR_A, a::Research>(&mut a, &simulator).await?;
                };
                async {
                    localside::research::run::<AUTHOR_B, b::Research>(&mut b, &simulator).await?;
                };
                async {
                    localside::research::run::<AUTHOR_C, c::Research>(&mut c, &simulator).await?;
                };
                async {
                    localside::research::supplemental::<AUTHOR_A, a::Research>(
                        &mut extra_a,
                        &simulator,
                    )
                    .await?;
                };
                async {
                    localside::research::supplemental::<AUTHOR_B, b::Research>(
                        &mut extra_b,
                        &simulator,
                    )
                    .await?;
                };
                async {
                    localside::research::supplemental::<AUTHOR_C, c::Research>(
                        &mut extra_c,
                        &simulator,
                    )
                    .await?;
                };
                Ok(())
            };
            async {
                let (a, b, c) = async {
                    let [mut a, mut b, mut c] = review_service.par()?;
                    async {
                        localside::review::run::<AUTHOR_A, SECTION_A, a::Research>(
                            &mut a, &simulator,
                        )
                        .await
                    };
                    async {
                        localside::review::run::<AUTHOR_B, SECTION_B, b::Research>(
                            &mut b, &simulator,
                        )
                        .await
                    };
                    async {
                        localside::review::run::<AUTHOR_C, SECTION_C, c::Research>(
                            &mut c, &simulator,
                        )
                        .await
                    };
                }
                .await?;
                let result = json!({
                    "facts":{"section_a":a.0,"section_b":b.0,"section_c":c.0},
                    "reviews":{"section_a":a.1,"section_b":b.1,"section_c":c.1},
                });
                let bytes = imp::encode(&result)?;
                review_service
                    .send::<FrozenReviews>(&bytes.as_slice())
                    .await?;
            };
            async {
                let (a, b, c) = async {
                    let [mut a, mut b, mut c] = archive.par()?;
                    async { a.recv::<Section<SECTION_A>>().await };
                    async { b.recv::<Section<SECTION_B>>().await };
                    async { c.recv::<Section<SECTION_C>>().await };
                }
                .await?;
                let [mut first, mut second, mut third] = archive.par()?;
                async {
                    first.send::<Section<SECTION_A>>(a).await?;
                };
                async {
                    second.send::<Section<SECTION_B>>(b).await?;
                };
                async {
                    third.send::<Section<SECTION_C>>(c).await?;
                };
                Ok(())
            };

            async {
                file_metadata::run(&mut author_a_data, author_a_file, AUTHOR_FILES).await?;
            };
            async {
                private_io::run(
                    &mut author_a_io,
                    PrivateFs::new(author_a_terminal, bus, &AUTHOR_FILES),
                )
                .await?;
            };
            async {
                let (file, root) = async {
                    let [mut file, mut vm] = author_a.par()?;
                    async { file.recv::<Loan<AUTHOR_A_FILE>>().await };
                    async { vm.recv::<Loan<AUTHOR_A_VM>>().await };
                }
                .await?;
                let ((), (mut file, status)) = async {
                    async {
                        localside::author::run::<AUTHOR_A, SECTION_A, a::Research>(
                            &mut author_a,
                            section_a,
                        )
                        .await?;
                    };
                    async {
                        let [
                            mut request_0,
                            mut request_1,
                            mut request_2,
                            mut request_3,
                            mut request_5,
                        ] = [(); 5].map(|_| vec![0; GUEST_MEMORY_BYTES]);
                        let [
                            mut reply_0,
                            mut reply_1,
                            mut reply_2,
                            mut reply_3,
                            mut reply_4,
                            mut reply_5,
                        ] = [(); 6].map(|_| vec![0; incoming_capacity]);
                        let file = ChoreoFsFile::new(AUTHOR_FILES[0], file)
                            .write_as::<a::Research>(&mut request_0)
                            .read_as::<a::Researched>(&mut reply_0)
                            .file(AUTHOR_FILES[1])?
                            .write_as::<a::Write>(&mut request_1)
                            .read_as::<a::Written>(&mut reply_1)
                            .file(AUTHOR_FILES[2])?
                            .write_as::<a::References>(&mut request_2)
                            .read_as::<a::Referenced>(&mut reply_2)
                            .file(AUTHOR_FILES[3])?
                            .write_as::<a::Revise>(&mut request_3)
                            .read_as::<a::Revised>(&mut reply_3)
                            .file(AUTHOR_FILES[4])?
                            .read_as::<a::AuthorNotice>(&mut reply_4)
                            .file(AUTHOR_FILES[5])?
                            .write_as::<a::Supplement>(&mut request_5)
                            .read_as::<a::Supplemented>(&mut reply_5);
                        let mut memory = vec![0; GUEST_MEMORY_BYTES];
                        let mut storage = HibanaWasiGuestStorage::uninit();
                        let files = ChoreoFsFiles::new(root).file(file)?;
                        let guest = storage.init(module, GuestMemory::new(&mut memory), files)?;
                        let (exit, files) = guest.run().await?;
                        let status = exit.status() as i32;
                        let (mut root, (file, ())) = files.into_parts();
                        root.send::<Exit>(&status).await?;
                        let mut file = file.into_endpoint();
                        file.send::<a::ResearchDone>(&status).await?;
                        Ok((file, status))
                    };
                }
                .await?;
                file.send::<FileExit>(&status).await?;
            };

            async {
                file_metadata::run(&mut author_b_data, author_b_file, AUTHOR_FILES).await?;
            };
            async {
                private_io::run(
                    &mut author_b_io,
                    PrivateFs::new(author_b_terminal, bus, &AUTHOR_FILES),
                )
                .await?;
            };
            async {
                let (file, root) = async {
                    let [mut file, mut vm] = author_b.par()?;
                    async { file.recv::<Loan<AUTHOR_B_FILE>>().await };
                    async { vm.recv::<Loan<AUTHOR_B_VM>>().await };
                }
                .await?;
                let ((), (mut file, status)) = async {
                    async {
                        localside::author::run::<AUTHOR_B, SECTION_B, b::Research>(
                            &mut author_b,
                            section_b,
                        )
                        .await?;
                    };
                    async {
                        let [
                            mut request_0,
                            mut request_1,
                            mut request_2,
                            mut request_3,
                            mut request_5,
                        ] = [(); 5].map(|_| vec![0; GUEST_MEMORY_BYTES]);
                        let [
                            mut reply_0,
                            mut reply_1,
                            mut reply_2,
                            mut reply_3,
                            mut reply_4,
                            mut reply_5,
                        ] = [(); 6].map(|_| vec![0; incoming_capacity]);
                        let file = ChoreoFsFile::new(AUTHOR_FILES[0], file)
                            .write_as::<b::Research>(&mut request_0)
                            .read_as::<b::Researched>(&mut reply_0)
                            .file(AUTHOR_FILES[1])?
                            .write_as::<b::Write>(&mut request_1)
                            .read_as::<b::Written>(&mut reply_1)
                            .file(AUTHOR_FILES[2])?
                            .write_as::<b::References>(&mut request_2)
                            .read_as::<b::Referenced>(&mut reply_2)
                            .file(AUTHOR_FILES[3])?
                            .write_as::<b::Revise>(&mut request_3)
                            .read_as::<b::Revised>(&mut reply_3)
                            .file(AUTHOR_FILES[4])?
                            .read_as::<b::AuthorNotice>(&mut reply_4)
                            .file(AUTHOR_FILES[5])?
                            .write_as::<b::Supplement>(&mut request_5)
                            .read_as::<b::Supplemented>(&mut reply_5);
                        let mut memory = vec![0; GUEST_MEMORY_BYTES];
                        let mut storage = HibanaWasiGuestStorage::uninit();
                        let files = ChoreoFsFiles::new(root).file(file)?;
                        let guest = storage.init(module, GuestMemory::new(&mut memory), files)?;
                        let (exit, files) = guest.run().await?;
                        let status = exit.status() as i32;
                        let (mut root, (file, ())) = files.into_parts();
                        root.send::<Exit>(&status).await?;
                        let mut file = file.into_endpoint();
                        file.send::<b::ResearchDone>(&status).await?;
                        Ok((file, status))
                    };
                }
                .await?;
                file.send::<FileExit>(&status).await?;
            };

            async {
                file_metadata::run(&mut author_c_data, author_c_file, AUTHOR_FILES).await?;
            };
            async {
                private_io::run(
                    &mut author_c_io,
                    PrivateFs::new(author_c_terminal, bus, &AUTHOR_FILES),
                )
                .await?;
            };
            async {
                let (file, root) = async {
                    let [mut file, mut vm] = author_c.par()?;
                    async { file.recv::<Loan<AUTHOR_C_FILE>>().await };
                    async { vm.recv::<Loan<AUTHOR_C_VM>>().await };
                }
                .await?;
                let ((), (mut file, status)) = async {
                    async {
                        localside::author::run::<AUTHOR_C, SECTION_C, c::Research>(
                            &mut author_c,
                            section_c,
                        )
                        .await?;
                    };
                    async {
                        let [
                            mut request_0,
                            mut request_1,
                            mut request_2,
                            mut request_3,
                            mut request_5,
                        ] = [(); 5].map(|_| vec![0; GUEST_MEMORY_BYTES]);
                        let [
                            mut reply_0,
                            mut reply_1,
                            mut reply_2,
                            mut reply_3,
                            mut reply_4,
                            mut reply_5,
                        ] = [(); 6].map(|_| vec![0; incoming_capacity]);
                        let file = ChoreoFsFile::new(AUTHOR_FILES[0], file)
                            .write_as::<c::Research>(&mut request_0)
                            .read_as::<c::Researched>(&mut reply_0)
                            .file(AUTHOR_FILES[1])?
                            .write_as::<c::Write>(&mut request_1)
                            .read_as::<c::Written>(&mut reply_1)
                            .file(AUTHOR_FILES[2])?
                            .write_as::<c::References>(&mut request_2)
                            .read_as::<c::Referenced>(&mut reply_2)
                            .file(AUTHOR_FILES[3])?
                            .write_as::<c::Revise>(&mut request_3)
                            .read_as::<c::Revised>(&mut reply_3)
                            .file(AUTHOR_FILES[4])?
                            .read_as::<c::AuthorNotice>(&mut reply_4)
                            .file(AUTHOR_FILES[5])?
                            .write_as::<c::Supplement>(&mut request_5)
                            .read_as::<c::Supplemented>(&mut reply_5);
                        let mut memory = vec![0; GUEST_MEMORY_BYTES];
                        let mut storage = HibanaWasiGuestStorage::uninit();
                        let files = ChoreoFsFiles::new(root).file(file)?;
                        let guest = storage.init(module, GuestMemory::new(&mut memory), files)?;
                        let (exit, files) = guest.run().await?;
                        let status = exit.status() as i32;
                        let (mut root, (file, ())) = files.into_parts();
                        root.send::<Exit>(&status).await?;
                        let mut file = file.into_endpoint();
                        file.send::<c::ResearchDone>(&status).await?;
                        Ok((file, status))
                    };
                }
                .await?;
                file.send::<FileExit>(&status).await?;
            };

            async {
                file_metadata::run(&mut fact_data, fact_file, FACT_FILES).await?;
            };
            async {
                private_io::run(
                    &mut fact_io,
                    PrivateFs::new(fact_terminal, bus, &FACT_FILES),
                )
                .await?;
            };
            async {
                let (file, root) = async {
                    let [mut file, mut vm] = fact.par()?;
                    async { file.recv::<Loan<FACT_FILE>>().await };
                    async { vm.recv::<Loan<FACT_VM>>().await };
                }
                .await?;
                let ((), (mut file, status)) = async {
                    async {
                        let [mut a, mut b, mut c] = fact.par()?;
                        async {
                            localside::fact::run::<AUTHOR_A, SECTION_A, a::Research>(&mut a)
                                .await?;
                        };
                        async {
                            localside::fact::run::<AUTHOR_B, SECTION_B, b::Research>(&mut b)
                                .await?;
                        };
                        async {
                            localside::fact::run::<AUTHOR_C, SECTION_C, c::Research>(&mut c)
                                .await?;
                        };
                        Ok(())
                    };
                    async {
                        let [mut request_1, mut request_4, mut request_7] =
                            [(); 3].map(|_| vec![0; GUEST_MEMORY_BYTES]);
                        let [
                            mut reply_0,
                            mut reply_1,
                            mut reply_2,
                            mut reply_3,
                            mut reply_4,
                            mut reply_5,
                            mut reply_6,
                            mut reply_7,
                            mut reply_8,
                        ] = [(); 9].map(|_| vec![0; incoming_capacity]);
                        let file = ChoreoFsFile::new(FACT_FILES[0], file)
                            .read_as::<a::FactInput>(&mut reply_0)
                            .file(FACT_FILES[1])?
                            .write_as::<a::FactRequest>(&mut request_1)
                            .read_as::<a::FactResult>(&mut reply_1)
                            .file(FACT_FILES[2])?
                            .read_as::<a::FactFinished>(&mut reply_2)
                            .file(FACT_FILES[3])?
                            .read_as::<b::FactInput>(&mut reply_3)
                            .file(FACT_FILES[4])?
                            .write_as::<b::FactRequest>(&mut request_4)
                            .read_as::<b::FactResult>(&mut reply_4)
                            .file(FACT_FILES[5])?
                            .read_as::<b::FactFinished>(&mut reply_5)
                            .file(FACT_FILES[6])?
                            .read_as::<c::FactInput>(&mut reply_6)
                            .file(FACT_FILES[7])?
                            .write_as::<c::FactRequest>(&mut request_7)
                            .read_as::<c::FactResult>(&mut reply_7)
                            .file(FACT_FILES[8])?
                            .read_as::<c::FactFinished>(&mut reply_8);
                        let mut memory = vec![0; GUEST_MEMORY_BYTES];
                        let mut storage = HibanaWasiGuestStorage::uninit();
                        let files = ChoreoFsFiles::new(root).file(file)?;
                        let guest = storage.init(module, GuestMemory::new(&mut memory), files)?;
                        let (exit, files) = guest.run().await?;
                        let status = exit.status() as i32;
                        let (mut root, (file, ())) = files.into_parts();
                        root.send::<Exit>(&status).await?;
                        Ok((file.into_endpoint(), status))
                    };
                }
                .await?;
                file.send::<FileExit>(&status).await?;
            };

            async {
                file_metadata::run(&mut editor_data, editor_file, EDITOR_FILES).await?;
            };
            async {
                private_io::run(
                    &mut editor_io,
                    PrivateFs::new(editor_terminal, bus, &EDITOR_FILES),
                )
                .await?;
            };
            async {
                let (file, root) = async {
                    let [mut file, mut vm] = editor.par()?;
                    async { file.recv::<Loan<EDITOR_FILE>>().await };
                    async { vm.recv::<Loan<EDITOR_VM>>().await };
                }
                .await?;
                let ((), (mut file, status)) = async {
                    async {
                        let [mut a, mut b, mut c] = editor.par()?;
                        async {
                            localside::editor::run::<AUTHOR_A, SECTION_A, a::Research>(&mut a)
                                .await?;
                        };
                        async {
                            localside::editor::run::<AUTHOR_B, SECTION_B, b::Research>(&mut b)
                                .await?;
                        };
                        async {
                            localside::editor::run::<AUTHOR_C, SECTION_C, c::Research>(&mut c)
                                .await?;
                        };
                        Ok(())
                    };
                    async {
                        let [mut request_1, mut request_3, mut request_5, mut request_6] =
                            [(); 4].map(|_| vec![0; GUEST_MEMORY_BYTES]);
                        let [
                            mut reply_0,
                            mut reply_1,
                            mut reply_2,
                            mut reply_3,
                            mut reply_4,
                            mut reply_5,
                            mut reply_6,
                        ] = [(); 7].map(|_| vec![0; incoming_capacity]);
                        let file = ChoreoFsFile::new(EDITOR_FILES[0], file)
                            .read_as::<a::ReviewInput>(&mut reply_0)
                            .file(EDITOR_FILES[1])?
                            .write_as::<a::ReviewRequest>(&mut request_1)
                            .read_as::<a::ReviewResult>(&mut reply_1)
                            .file(EDITOR_FILES[2])?
                            .read_as::<b::ReviewInput>(&mut reply_2)
                            .file(EDITOR_FILES[3])?
                            .write_as::<b::ReviewRequest>(&mut request_3)
                            .read_as::<b::ReviewResult>(&mut reply_3)
                            .file(EDITOR_FILES[4])?
                            .read_as::<c::ReviewInput>(&mut reply_4)
                            .file(EDITOR_FILES[5])?
                            .write_as::<c::ReviewRequest>(&mut request_5)
                            .read_as::<c::ReviewResult>(&mut reply_5)
                            .file(EDITOR_FILES[6])?
                            .write_as::<Combine>(&mut request_6)
                            .read_as::<Published>(&mut reply_6);
                        let mut memory = vec![0; GUEST_MEMORY_BYTES];
                        let mut storage = HibanaWasiGuestStorage::uninit();
                        let files = ChoreoFsFiles::new(root).file(file)?;
                        let guest = storage.init(module, GuestMemory::new(&mut memory), files)?;
                        let (exit, files) = guest.run().await?;
                        let status = exit.status() as i32;
                        let (mut root, (file, ())) = files.into_parts();
                        root.send::<Exit>(&status).await?;
                        Ok((file.into_endpoint(), status))
                    };
                }
                .await?;
                file.send::<FileExit>(&status).await?;
            };

            Ok(())
        }
        .await;
        bus.checkpoint(SESSION, observations.checkpoint());
        result
    };
    let result = hibana_runtime::race(participants, io.run(bus)).await;
    io.finish(bus, result, None).await
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod publication_tests;

#[cfg(test)]
mod whole_tests;
