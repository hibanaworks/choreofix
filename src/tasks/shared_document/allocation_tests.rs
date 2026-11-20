//! Finite real-Wasm custody prefixes; these do not certify the whole task.
use super::{global::*, localside};
use crate::shell::global::{FileExit, Loan};
use crate::shell::{
    ShellError, ShellResult,
    imp::{
        bus::Bus,
        service::{ReceivedEffects, SimulatorOperation},
    },
};
use hibana::runtime::{SessionKitStorage, ids::SessionId};
use hibana_wasip1_runtime::{ChoreoFsFile, ChoreoFsFiles, GuestMemory, HibanaWasiGuestStorage};
use serde_json::Value;

struct NoEffect;
impl ReceivedEffects for NoEffect {
    fn apply<'io, P: SimulatorOperation>(
        &'io self,
        _: hibana::runtime::wire::ReceivedPayload<'_, P>,
    ) -> impl Future<Output = ShellResult<Value>> + use<'io, P> {
        async { panic!("choosing custody must not execute a document effect") }
    }
}

struct Clock(usize);
impl hibana_runtime::Driver for Clock {
    fn waker(&self) -> std::task::Waker {
        std::task::Waker::noop().clone()
    }
    fn wait(&mut self) {
        self.0 += 1;
        assert!(self.0 < 1000, "document allocation stopped making progress");
    }
}

fn leb(mut value: usize, bytes: &mut Vec<u8>) {
    loop {
        let low = (value & 127) as u8;
        value >>= 7;
        bytes.push(low | if value == 0 { 0 } else { 128 });
        if value == 0 {
            break;
        }
    }
}
fn section(tag: u8, bytes: &[u8], module: &mut Vec<u8>) {
    module.push(tag);
    leb(bytes.len(), module);
    module.extend_from_slice(bytes);
}
fn constant(mut value: usize, body: &mut Vec<u8>) {
    body.push(0x41);
    loop {
        let low = (value & 127) as u8;
        value >>= 7;
        let last = value == 0 && low & 64 == 0;
        body.push(low | if last { 0 } else { 128 });
        if last {
            break;
        }
    }
}
fn equal(expected: usize, body: &mut Vec<u8>) {
    constant(expected, body);
    body.extend_from_slice(&[0x47, 0x04, 0x40, 0, 0x0b]);
}
fn request(pointer: usize, length: usize, expected: usize, body: &mut Vec<u8>) {
    for value in [3, 0, 32, ALLOCATION_OBJECT.path().len(), 8] {
        constant(value, body);
    }
    body.extend_from_slice(&[0x42, 0xc2, 0, 0x42, 0]);
    for value in [0, 0] {
        constant(value, body);
    }
    body.extend_from_slice(&[0x10, 0]);
    equal(0, body);
    // The guest itself loops over chunks, with bounded static control targets.
    // local 0 is the byte offset and local 1 the current chunk length.
    constant(0, body);
    body.extend_from_slice(&[0x21, 0, 0x02, 0x40, 0x03, 0x40]);
    body.extend_from_slice(&[0x20, 0]);
    constant(length, body);
    body.extend_from_slice(&[0x4f, 0x0d, 1]);
    constant(length, body);
    body.extend_from_slice(&[0x20, 0, 0x6b, 0x22, 1]);
    constant(
        hibana_wasip1_runtime::protocol::WASIP1_IO_CHUNK_CAPACITY,
        body,
    );
    body.extend_from_slice(&[0x20, 1]);
    constant(
        hibana_wasip1_runtime::protocol::WASIP1_IO_CHUNK_CAPACITY,
        body,
    );
    body.extend_from_slice(&[0x49, 0x1b, 0x21, 1]);
    constant(64, body);
    constant(pointer, body);
    body.extend_from_slice(&[0x20, 0, 0x6a, 0x36, 2, 0]);
    constant(68, body);
    body.extend_from_slice(&[0x20, 1, 0x36, 2, 0]);
    for value in [4, 64, 1, 4] {
        constant(value, body);
    }
    body.extend_from_slice(&[0x10, 1]);
    equal(0, body);
    constant(4, body);
    body.extend_from_slice(&[0x28, 2, 0, 0x20, 1, 0x47, 0x04, 0x40, 0, 0x0b]);
    body.extend_from_slice(&[0x20, 0, 0x20, 1, 0x6a, 0x21, 0, 0x0c, 0, 0x0b, 0x0b]);
    constant(4, body);
    body.extend_from_slice(&[0x10, 2]);
    equal(expected, body);
}
fn module(first: &str, padding: usize) -> Vec<u8> {
    let expected = serde_json::to_vec(&serde_json::json!({"first_writer":first})).unwrap();
    let mut input = expected.clone();
    input.resize(input.len() + padding, b' ');
    let invalid = br#"{"first_writer":"OUTSIDE"}"#;
    let mut body = vec![1, 2, 0x7f];
    request(256, invalid.len(), 28, &mut body);
    request(1024, input.len(), 0, &mut body);
    // Read the actual selected-choice reply before closing this finite prefix.
    for value in [3, 0, 32, ALLOCATION_OBJECT.path().len(), 0] {
        constant(value, &mut body);
    }
    body.extend_from_slice(&[0x42, 2, 0x42, 0]);
    for value in [0, 0] {
        constant(value, &mut body);
    }
    body.extend_from_slice(&[0x10, 0]);
    equal(0, &mut body);
    constant(64, &mut body);
    constant(512, &mut body);
    body.extend_from_slice(&[0x36, 2, 0]);
    constant(68, &mut body);
    constant(128, &mut body);
    body.extend_from_slice(&[0x36, 2, 0]);
    for value in [4, 64, 1, 4] {
        constant(value, &mut body);
    }
    body.extend_from_slice(&[0x10, 4]);
    equal(0, &mut body);
    constant(4, &mut body);
    body.extend_from_slice(&[0x28, 2, 0]);
    equal(expected.len(), &mut body);
    for (index, byte) in expected.iter().enumerate() {
        constant(512 + index, &mut body);
        body.extend_from_slice(&[0x2d, 0, 0]);
        equal(usize::from(*byte), &mut body);
    }
    constant(4, &mut body);
    body.extend_from_slice(&[0x10, 2]);
    equal(0, &mut body);
    request(1024, input.len(), 2, &mut body);
    constant(0, &mut body);
    body.extend_from_slice(&[0x10, 3, 0x0b]);
    let mut module = b"\0asm\x01\0\0\0".to_vec();
    section(
        1,
        &[
            5, 0x60, 9, 0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0x7e, 0x7e, 0x7f, 0x7f, 1, 0x7f, 0x60, 4,
            0x7f, 0x7f, 0x7f, 0x7f, 1, 0x7f, 0x60, 1, 0x7f, 1, 0x7f, 0x60, 1, 0x7f, 0, 0x60, 0, 0,
        ],
        &mut module,
    );
    let mut imports = vec![5];
    for (name, ty) in [
        ("path_open", 0),
        ("fd_write", 1),
        ("fd_close", 2),
        ("proc_exit", 3),
        ("fd_read", 1),
    ] {
        let namespace = b"wasi_snapshot_preview1";
        imports.push(namespace.len() as u8);
        imports.extend_from_slice(namespace);
        imports.push(name.len() as u8);
        imports.extend_from_slice(name.as_bytes());
        imports.extend_from_slice(&[0, ty]);
    }
    section(2, &imports, &mut module);
    section(3, &[1, 4], &mut module);
    section(5, &[1, 0, 1], &mut module);
    section(
        7,
        &[1, 6, b'_', b's', b't', b'a', b'r', b't', 0, 5],
        &mut module,
    );
    let mut code = vec![1];
    leb(body.len(), &mut code);
    code.extend_from_slice(&body);
    section(10, &code, &mut module);
    let mut bytes = vec![0; 1024 + input.len()];
    let path = ALLOCATION_OBJECT.path();
    bytes[32..32 + path.len()].copy_from_slice(path);
    bytes[1024..].copy_from_slice(&input);
    bytes[256..256 + invalid.len()].copy_from_slice(invalid);
    let mut data = vec![1, 0, 0x41, 0, 0x0b];
    leb(bytes.len(), &mut data);
    data.extend_from_slice(&bytes);
    section(11, &data, &mut module);
    module
}

#[hibana_runtime::main(Clock(0))]
#[test]
async fn editor_wasm_chooses_both_first_custodians_and_retries_in_the_same_guest() -> ShellResult<()>
{
    for (first, padding) in [
        ("WRITER_A", 0),
        ("WRITER_B", 0),
        ("WRITER_A", 10_000),
        ("WRITER_B", 10_000),
    ] {
        let p = programs();
        let session = SessionId::new(SESSION);
        let (bus, _reader, _) = Bus::open(
            [],
            [
                (session, DOCUMENT),
                (session, WRITER_A),
                (session, WRITER_B),
                (session, EDITOR),
                (session, A_FILE),
                (session, B_FILE),
                (session, ALLOCATION_FILE),
            ],
        );
        let mut slab = vec![0; 1024 * 1024];
        let mut storage = SessionKitStorage::uninit();
        let rv = storage.init().rendezvous(&mut slab, &bus.transport)?;
        let mut service = rv.enter(session, &p.service)?;
        let document = rv.enter(session, &p.document)?;
        let mut a_vm = rv.enter(session, &p.a_vm)?;
        let mut b_vm = rv.enter(session, &p.b_vm)?;
        let mut editor = rv.enter(session, &p.editor_vm)?;
        service
            .send::<Loan<WRITER_A>>(rv.enter(session, &p.a)?)
            .await?;
        let mut a = a_vm.recv::<Loan<WRITER_A>>().await?;
        service
            .send::<Loan<WRITER_B>>(rv.enter(session, &p.b)?)
            .await?;
        let mut b = b_vm.recv::<Loan<WRITER_B>>().await?;
        service
            .send::<Loan<EDITOR>>(rv.enter(session, &p.editor)?)
            .await?;
        let _editor = editor.recv::<Loan<EDITOR>>().await?;
        let mut a_data = rv.enter(session, &p.a_data)?;
        let mut b_data = rv.enter(session, &p.b_data)?;
        a_data
            .send::<Loan<A_FILE>>(rv.enter(session, &p.a_file)?)
            .await?;
        let _a_file = a_vm.recv::<Loan<A_FILE>>().await?;
        b_data
            .send::<Loan<B_FILE>>(rv.enter(session, &p.b_file)?)
            .await?;
        let _b_file = b_vm.recv::<Loan<B_FILE>>().await?;
        let mut data = rv.enter(session, &p.allocation_data)?;
        let file = rv.enter(session, &p.allocation_file)?;
        let outcome = hibana_runtime::race(
            async {
                let _document = localside::document::run(&mut service, document, &NoEffect).await?;
                Err::<(), _>(ShellError::message(
                    "the full document must not finish in a custody prefix",
                ))
            },
            async {
                let (_, (_editor, _file), _document) = async {
                    async {
                        crate::shell::localside::file_metadata::run(
                            &mut data,
                            file,
                            [ALLOCATION_OBJECT],
                        )
                        .await
                    };
                    async {
                        let file = editor.recv::<Loan<ALLOCATION_FILE>>().await?;
                        let mut incoming = [0; 128];
                        let mut outgoing = vec![0; hibana_wasip1_runtime::GUEST_MEMORY_PAGE_SIZE];
                        let files = ChoreoFsFiles::new(editor).file(
                            ChoreoFsFile::new(ALLOCATION_OBJECT, file)
                                .write_as::<Allocate>(&mut outgoing)
                                .read_as::<AllocationAccepted>(&mut incoming),
                        )?;
                        let module = module(first, padding);
                        let mut memory = vec![0; 65536];
                        let mut storage = HibanaWasiGuestStorage::uninit();
                        let guest = storage.init(&module, GuestMemory::new(&mut memory), files)?;
                        let (exit, files) = guest.run().await?;
                        assert_eq!(exit.status(), 0);
                        let (editor, (file, ())) = files.into_parts();
                        let mut file = file.into_endpoint();
                        file.send::<FileExit>(&0).await?;
                        Ok::<_, ShellError>((editor, file))
                    };
                    async {
                        let document = match first {
                            "WRITER_A" => a.recv::<InitialDocument>().await?,
                            "WRITER_B" => b.recv::<InitialDocumentB>().await?,
                            _ => unreachable!(),
                        };
                        Ok::<_, ShellError>(document)
                    };
                }
                .await?;
                Ok::<(), ShellError>(())
            },
        )
        .await;
        outcome?;
    }
    Ok(())
}
