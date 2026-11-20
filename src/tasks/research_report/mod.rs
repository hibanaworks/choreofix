//! Direct ChoreoFS projections and actual report resources for 3E.
pub mod global;
mod imp;
pub(crate) mod localside;
#[cfg(test)]
mod reviewer_tests;
#[cfg(test)]
mod tests;
use crate::shell::{
    ShellError, ShellResult,
    global::{FileExit, Loan},
};
use hibana::runtime::{SessionKitStorage, ids::SessionId};

#[hibana_runtime::scope]
pub async fn run_stdio(module: &[u8]) -> ShellResult<()> {
    use crate::shell::{
        imp::{bus::Bus, private_fs::PrivateFs},
        localside::{file_metadata, private_io},
    };
    use global::*;
    use hibana_wasip1_runtime::{ChoreoFsFile, ChoreoFsFiles, GuestMemory, HibanaWasiGuestStorage};
    let (bus, mut io, [a_terminal, b_terminal, editor_terminal]) = Bus::open(
        ["RESEARCHER_A", "RESEARCHER_B", "EDITOR"],
        [
            (SessionId::new(SESSION), DOCUMENT),
            (SessionId::new(SESSION), DATABASE),
            (SessionId::new(SESSION), A_WASI),
            (SessionId::new(SESSION), B_WASI),
            (SessionId::new(SESSION), REVIEW_WASI),
            (SessionId::new(SESSION), REVIEW_A),
            (SessionId::new(SESSION), REVIEW_B),
            (SessionId::new(SESSION), RESEARCH_A),
            (SessionId::new(SESSION), RESEARCH_B),
            (SessionId::new(SESSION), A_FILE),
            (SessionId::new(SESSION), B_FILE),
            (SessionId::new(SESSION), EDITOR_FILE),
        ],
    );
    let result = hibana_runtime::race(async {
        let p = programs();
        bus.observe(p.observation());
        let capture = |event| bus.observe(serde_json::json!({"type":"protocol_capture","producer":0,"records":[crate::imp::projection::record(event)]}));
        let mut slab = vec![0; 1024 * 1024];
        let incoming_capacity = slab.len();
        let mut storage = SessionKitStorage::uninit();
        let rv = storage.init().rendezvous(&mut slab, &bus.transport)?;
        rv.observe(&capture);
        let observations = rv.tap();
        let session = SessionId::new(SESSION);
        let result = async {
            let mut initial_document = rv.enter(session,&p.initial_document)?;
            let mut write_requests = rv.enter(session,&p.write_requests)?;
            let mut reference_requests = rv.enter(session,&p.reference_requests)?;
            let mut service = rv.enter(session,&p.document_service)?;
            let mut editor = rv.enter(session,&p.review_wasi)?;
            let review_a = rv.enter(session,&p.review_a)?;
            let review_b = rv.enter(session,&p.review_b)?;
            let research_a = rv.enter(session,&p.research_a)?;
            let research_b = rv.enter(session,&p.research_b)?;
            editor.send::<Loan<REVIEW_A>>(review_a).await?;
            let review_a = service.recv::<Loan<REVIEW_A>>().await?;
            editor.send::<Loan<REVIEW_B>>(review_b).await?;
            let review_b = service.recv::<Loan<REVIEW_B>>().await?;
            editor.send::<Loan<RESEARCH_A>>(research_a).await?;
            let research_a = service.recv::<Loan<RESEARCH_A>>().await?;
            editor.send::<Loan<RESEARCH_B>>(research_b).await?;
            let research_b = service.recv::<Loan<RESEARCH_B>>().await?;
            let a = rv.enter(session,&p.a_wasi)?;
            let b = rv.enter(session,&p.b_wasi)?;
            let document = rv.enter(session,&p.document)?;
            let mut database_service = rv.enter(session,&p.database_service)?;
            let database = rv.enter(session,&p.database)?;
            let mut a_vm = rv.enter(session,&p.a_vm)?;
            let mut b_vm = rv.enter(session,&p.b_vm)?;
            let mut editor_vm = rv.enter(session,&p.editor_vm)?;
            let mut a_data = rv.enter(session,&p.a_data)?;
            let mut b_data = rv.enter(session,&p.b_data)?;
            let mut editor_data = rv.enter(session,&p.editor_data)?;
            let a_file = rv.enter(session,&p.a_file)?;
            let b_file = rv.enter(session,&p.b_file)?;
            let editor_file = rv.enter(session,&p.editor_file)?;
            let mut a_io = rv.enter(session,&p.a_io)?;
            let mut b_io = rv.enter(session,&p.b_io)?;
            let mut editor_io = rv.enter(session,&p.review_io)?;
            async { file_metadata::run(&mut a_data,a_file,AUTHOR_FILES).await?; };
            async { file_metadata::run(&mut b_data,b_file,AUTHOR_FILES).await?; };
            async { file_metadata::run(&mut editor_data,editor_file,EDITOR_FILES).await?; };
            let (mut a,mut b,mut editor,editor_file) = async {
                let [mut a_owner,mut b_owner,mut editor_owner,mut file_owner] = service.par()?;
                async { a_owner.send::<Loan<A_WASI>>(a).await?; a_vm.recv::<Loan<A_WASI>>().await };
                async { b_owner.send::<Loan<B_WASI>>(b).await?; b_vm.recv::<Loan<B_WASI>>().await };
                async { editor_owner.send::<Loan<REVIEW_WASI>>(editor).await?; editor_vm.recv::<Loan<REVIEW_WASI>>().await };
                async { file_owner.recv::<Loan<EDITOR_FILE>>().await };
            }.await?;
            async { localside::resources::initial_document(&mut initial_document,(document,&mut write_requests),&bus).await?; };
            async { localside::resources::document(&mut service,&bus).await?; };
            async { localside::resources::database(&mut database_service,(database,&mut reference_requests),&bus).await?; };
            async { localside::reviewer_a::run(review_a,research_a,&bus).await?; };
            async { localside::reviewer_b::run(review_b,research_b,&bus).await?; };
            async { localside::author::run(&mut a,&bus).await?; };
            async { localside::author::run(&mut b,&bus).await?; };
            async { let (_document,_review_a,_review_b) = localside::editor::run(&mut editor).await?; };
            async {
                let file = a_vm.recv::<Loan<A_FILE>>().await?;
                let (mut research,mut write,mut references,mut revision) = (vec![0;crate::shell::GUEST_MEMORY_BYTES],vec![0;crate::shell::GUEST_MEMORY_BYTES],vec![0;crate::shell::GUEST_MEMORY_BYTES],vec![0;crate::shell::GUEST_MEMORY_BYTES]);
                let (mut researched,mut written,mut referenced,mut revised,mut feedback) = (vec![0;incoming_capacity],vec![0;incoming_capacity],vec![0;incoming_capacity],vec![0;incoming_capacity],vec![0;incoming_capacity]);
                let files = ChoreoFsFiles::new(a_vm).file(
                    ChoreoFsFile::new(AUTHOR_FILES[0],file).write_as::<ResearchA>(&mut research).read_as::<ResearchResult>(&mut researched)
                        .file(AUTHOR_FILES[1])?.write_as::<WriteA>(&mut write).read_as::<WriteResultA>(&mut written)
                        .file(AUTHOR_FILES[2])?.write_as::<ReferencesA>(&mut references).read_as::<ReferenceResultA>(&mut referenced)
                        .file(AUTHOR_FILES[3])?.write_as::<ReviseA>(&mut revision).read_as::<RevisionResultA>(&mut revised)
                        .file(AUTHOR_FILES[4])?.read_as::<FeedbackA>(&mut feedback),
                )?;
                let mut memory = vec![0;crate::shell::GUEST_MEMORY_BYTES];
                let mut storage = HibanaWasiGuestStorage::uninit();
                let guest = storage.init(module,GuestMemory::new(&mut memory),files)?;
                let (exit,files) = guest.run().await?;
                let status = exit.status() as i32;
                let (mut root,(file,())) = files.into_parts();
                let mut file = file.into_endpoint();
                file.send::<FileExit>(&status).await?;
                root.send::<crate::shell::global::Exit>(&status).await?;
            };
            async {
                let file = b_vm.recv::<Loan<B_FILE>>().await?;
                let (mut research,mut write,mut references,mut revision) = (vec![0;crate::shell::GUEST_MEMORY_BYTES],vec![0;crate::shell::GUEST_MEMORY_BYTES],vec![0;crate::shell::GUEST_MEMORY_BYTES],vec![0;crate::shell::GUEST_MEMORY_BYTES]);
                let (mut researched,mut written,mut referenced,mut revised,mut feedback) = (vec![0;incoming_capacity],vec![0;incoming_capacity],vec![0;incoming_capacity],vec![0;incoming_capacity],vec![0;incoming_capacity]);
                let files = ChoreoFsFiles::new(b_vm).file(
                    ChoreoFsFile::new(AUTHOR_FILES[0],file).write_as::<ResearchB>(&mut research).read_as::<ResearchResult>(&mut researched)
                        .file(AUTHOR_FILES[1])?.write_as::<WriteB>(&mut write).read_as::<WriteResultB>(&mut written)
                        .file(AUTHOR_FILES[2])?.write_as::<ReferencesB>(&mut references).read_as::<ReferenceResultB>(&mut referenced)
                        .file(AUTHOR_FILES[3])?.write_as::<ReviseB>(&mut revision).read_as::<RevisionResultB>(&mut revised)
                        .file(AUTHOR_FILES[4])?.read_as::<FeedbackB>(&mut feedback),
                )?;
                let mut memory = vec![0;crate::shell::GUEST_MEMORY_BYTES];
                let mut storage = HibanaWasiGuestStorage::uninit();
                let guest = storage.init(module,GuestMemory::new(&mut memory),files)?;
                let (exit,files) = guest.run().await?;
                let status = exit.status() as i32;
                let (mut root,(file,())) = files.into_parts();
                let mut file = file.into_endpoint();
                file.send::<FileExit>(&status).await?;
                root.send::<crate::shell::global::Exit>(&status).await?;
            };
            async {
                let (mut review_a,mut review_b,mut combine) = (vec![0;crate::shell::GUEST_MEMORY_BYTES],vec![0;crate::shell::GUEST_MEMORY_BYTES],vec![0;crate::shell::GUEST_MEMORY_BYTES]);
                let (mut reviewed_a,mut reviewed_b,mut combined) = (vec![0;incoming_capacity],vec![0;incoming_capacity],vec![0;incoming_capacity]);
                let files = ChoreoFsFiles::new(editor_vm).file(
                    ChoreoFsFile::new(EDITOR_FILES[0],editor_file).write_as::<ReviewA>(&mut review_a).read_as::<ReviewResultA>(&mut reviewed_a)
                        .file(EDITOR_FILES[1])?.write_as::<ReviewB>(&mut review_b).read_as::<ReviewResultB>(&mut reviewed_b)
                        .file(EDITOR_FILES[2])?.write_as::<Combine>(&mut combine).read_as::<CombinedReply>(&mut combined),
                )?;
                let mut memory = vec![0;crate::shell::GUEST_MEMORY_BYTES];
                let mut storage = HibanaWasiGuestStorage::uninit();
                let guest = storage.init(module,GuestMemory::new(&mut memory),files)?;
                let (exit,files) = guest.run().await?;
                let status = exit.status() as i32;
                let (mut root,(file,())) = files.into_parts();
                let mut file = file.into_endpoint();
                file.send::<FileExit>(&status).await?;
                root.send::<crate::shell::global::Exit>(&status).await?;
            };
            async { private_io::run(&mut a_io,PrivateFs::new(a_terminal,&bus,&AUTHOR_FILES)).await?; };
            async { private_io::run(&mut b_io,PrivateFs::new(b_terminal,&bus,&AUTHOR_FILES)).await?; };
            async { private_io::run(&mut editor_io,PrivateFs::new(editor_terminal,&bus,&EDITOR_FILES)).await?; };
            Ok::<(),ShellError>(())
        }.await;
        bus.observe(serde_json::json!({"type":"protocol_checkpoint","producer":0,"session":SESSION,"checkpoint":observations.checkpoint()}));
        result
    },io.run(&bus)).await;
    match &result {
        Ok(()) => io.emit(&bus,serde_json::json!({"type":"run_result","status":"complete"})).await?,
        Err(error) => io.emit(&bus,serde_json::json!({"type":"run_result","status":"failed","error":error.to_string()})).await?,
    }
    result
}
pub fn projection() -> serde_json::Value {
    global::programs().observation()
}
