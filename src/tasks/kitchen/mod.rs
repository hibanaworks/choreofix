//! Direct operation-file Endpoints and actual equipment/material flows.
pub mod global;
pub(crate) mod imp;
pub mod localside;
#[cfg(test)]
mod tests;
use crate::shell::{ShellError, ShellResult};
use hibana::runtime::{SessionKitStorage, ids::SessionId};
#[hibana_runtime::scope]
pub async fn run_stdio(module: &[u8]) -> ShellResult<()> {
    use crate::shell::{
        imp::{bus::Bus, private_fs::PrivateFs},
        localside::{file_metadata, private_io},
    };
    use global::*;
    use hibana_wasip1_runtime::{ChoreoFsFile, ChoreoFsFiles, GuestMemory, HibanaWasiGuestStorage};
    let session = SessionId::new(SESSION);
    let (mut connection, mut io, [a_terminal, b_terminal, c_terminal]) = Bus::open(
        imp::CHEFS,
        [
            (session, OVEN),
            (session, STOVE),
            (session, BASE_FILE_A),
            (session, BASE_FILE_B),
            (session, BASE_FILE_C),
            (session, INGREDIENT_FILE_A),
            (session, INGREDIENT_FILE_B),
            (session, INGREDIENT_FILE_C),
            (session, COMBINE_FILE_A),
            (session, COMBINE_FILE_B),
            (session, COMBINE_FILE_C),
        ],
    );
    let (bus, simulator) = connection.split();
    let result = hibana_runtime::race(
        async {
            let p = programs();
            bus.projected(p.observation());
            let capture = |event| bus.capture(event);
            let mut slab = vec![0; 2 * 1024 * 1024];
            let incoming_capacity = slab.len();
            let mut storage = SessionKitStorage::uninit();
            let rv = storage.init().rendezvous(&mut slab, &bus.transport)?;
            rv.observe(&capture);
            let observations = rv.tap();
            let result = async {
                let mut a = rv.enter(session, &p.a)?;
                let mut b = rv.enter(session, &p.b)?;
                let mut c = rv.enter(session, &p.c)?;
                let mut cooking = rv.enter(session, &p.cooking)?;
                let mut ingredients = rv.enter(session, &p.ingredients)?;
                let mut finish = rv.enter(session, &p.finish)?;
                let oven = rv.enter(session, &p.oven)?;
                let stove = rv.enter(session, &p.stove)?;
                let mut a_io = rv.enter(session, &p.a_io)?;
                let mut b_io = rv.enter(session, &p.b_io)?;
                let mut c_io = rv.enter(session, &p.c_io)?;
                let mut base_data_a = rv.enter(session, &p.base_data_a)?;
                let mut base_data_b = rv.enter(session, &p.base_data_b)?;
                let mut base_data_c = rv.enter(session, &p.base_data_c)?;
                let mut ingredient_data_a = rv.enter(session, &p.ingredient_data_a)?;
                let mut ingredient_data_b = rv.enter(session, &p.ingredient_data_b)?;
                let mut ingredient_data_c = rv.enter(session, &p.ingredient_data_c)?;
                let mut combine_data_a = rv.enter(session, &p.combine_data_a)?;
                let mut combine_data_b = rv.enter(session, &p.combine_data_b)?;
                let mut combine_data_c = rv.enter(session, &p.combine_data_c)?;
                let base_file_a = rv.enter(session, &p.base_file_a)?;
                let base_file_b = rv.enter(session, &p.base_file_b)?;
                let base_file_c = rv.enter(session, &p.base_file_c)?;
                let ingredient_file_a = rv.enter(session, &p.ingredient_file_a)?;
                let ingredient_file_b = rv.enter(session, &p.ingredient_file_b)?;
                let ingredient_file_c = rv.enter(session, &p.ingredient_file_c)?;
                let combine_file_a = rv.enter(session, &p.combine_file_a)?;
                let combine_file_b = rv.enter(session, &p.combine_file_b)?;
                let combine_file_c = rv.enter(session, &p.combine_file_c)?;
                let a_vm = rv.enter(session, &p.a_vm)?;
                let b_vm = rv.enter(session, &p.b_vm)?;
                let c_vm = rv.enter(session, &p.c_vm)?;
                async {
                    file_metadata::run(&mut base_data_a, base_file_a, [FILES[0]]).await?;
                };
                async {
                    file_metadata::run(&mut base_data_b, base_file_b, [FILES[0]]).await?;
                };
                async {
                    file_metadata::run(&mut base_data_c, base_file_c, [FILES[0]]).await?;
                };
                async {
                    file_metadata::run(&mut ingredient_data_a, ingredient_file_a, [FILES[1]])
                        .await?;
                };
                async {
                    file_metadata::run(&mut ingredient_data_b, ingredient_file_b, [FILES[1]])
                        .await?;
                };
                async {
                    file_metadata::run(&mut ingredient_data_c, ingredient_file_c, [FILES[1]])
                        .await?;
                };
                async {
                    file_metadata::run(&mut combine_data_a, combine_file_a, [FILES[2]]).await?;
                };
                async {
                    file_metadata::run(&mut combine_data_b, combine_file_b, [FILES[2]]).await?;
                };
                async {
                    file_metadata::run(&mut combine_data_c, combine_file_c, [FILES[2]]).await?;
                };
                let (
                    (base_file_a, base_file_b, base_file_c),
                    (ingredient_file_a, ingredient_file_b, ingredient_file_c),
                    (combine_file_a, combine_file_b, combine_file_c),
                ) = async {
                    async {
                        let [mut a, mut b, mut c] = cooking.par()?;
                        async { Ok::<_, ShellError>(a.recv::<Loan<BASE_FILE_A>>().await?) };
                        async { Ok::<_, ShellError>(b.recv::<Loan<BASE_FILE_B>>().await?) };
                        async { Ok::<_, ShellError>(c.recv::<Loan<BASE_FILE_C>>().await?) };
                    };
                    async {
                        let [mut a, mut b, mut c] = ingredients.par()?;
                        async { Ok::<_, ShellError>(a.recv::<Loan<INGREDIENT_FILE_A>>().await?) };
                        async { Ok::<_, ShellError>(b.recv::<Loan<INGREDIENT_FILE_B>>().await?) };
                        async { Ok::<_, ShellError>(c.recv::<Loan<INGREDIENT_FILE_C>>().await?) };
                    };
                    async {
                        async { Ok::<_, ShellError>(a.recv::<Loan<COMBINE_FILE_A>>().await?) };
                        async { Ok::<_, ShellError>(b.recv::<Loan<COMBINE_FILE_B>>().await?) };
                        async { Ok::<_, ShellError>(c.recv::<Loan<COMBINE_FILE_C>>().await?) };
                    };
                }
                .await?;
                async {
                    localside::cooking::run(&mut cooking, (oven, stove)).await?;
                };
                async {
                    localside::ingredients::run(&mut ingredients, &simulator).await?;
                };
                async {
                    localside::finish::run(&mut finish, &simulator).await?;
                };
                async {
                    localside::chef_a::run(&mut a, &simulator).await?;
                };
                async {
                    localside::chef_b::run(&mut b, &simulator).await?;
                };
                async {
                    localside::chef_c::run(&mut c, &simulator).await?;
                };
                async {
                    private_io::run(&mut a_io, PrivateFs::new(a_terminal, bus, &FILES)).await?;
                };
                async {
                    private_io::run(&mut b_io, PrivateFs::new(b_terminal, bus, &FILES)).await?;
                };
                async {
                    private_io::run(&mut c_io, PrivateFs::new(c_terminal, bus, &FILES)).await?;
                };
                async {
                    let (mut base_request, mut ingredient_request, mut combine_request) = (
                        vec![0; crate::shell::GUEST_MEMORY_BYTES],
                        vec![0; crate::shell::GUEST_MEMORY_BYTES],
                        vec![0; crate::shell::GUEST_MEMORY_BYTES],
                    );
                    let (mut base_result, mut ingredient_result, mut dish_result) = (
                        vec![0; incoming_capacity],
                        vec![0; incoming_capacity],
                        vec![0; incoming_capacity],
                    );
                    let files = ChoreoFsFiles::new(a_vm)
                        .file(
                            ChoreoFsFile::new(FILES[0], base_file_a)
                                .write_as::<BaseIntentA>(&mut base_request)
                                .read_as::<BaseResultA>(&mut base_result),
                        )?
                        .file(
                            ChoreoFsFile::new(FILES[1], ingredient_file_a)
                                .write_as::<PrepareSauce>(&mut ingredient_request)
                                .read_as::<SaucePrepared>(&mut ingredient_result),
                        )?
                        .file(
                            ChoreoFsFile::new(FILES[2], combine_file_a)
                                .write_as::<CombineA>(&mut combine_request)
                                .read_as::<DishA>(&mut dish_result),
                        )?;
                    let mut memory = vec![0; crate::shell::GUEST_MEMORY_BYTES];
                    let mut storage = HibanaWasiGuestStorage::uninit();
                    let guest = storage.init(module, GuestMemory::new(&mut memory), files)?;
                    let (exit, files) = guest.run().await?;
                    let status = exit.status() as i32;
                    let (mut root, (base, (ingredient, (combine, ())))) = files.into_parts();
                    async {
                        async {
                            base.into_endpoint().send::<FileExit>(&status).await?;
                        };
                        async {
                            ingredient.into_endpoint().send::<FileExit>(&status).await?;
                        };
                        async {
                            combine.into_endpoint().send::<FileExit>(&status).await?;
                        };
                        Ok::<(), ShellError>(())
                    }
                    .await?;
                    root.send::<crate::shell::global::Exit>(&status).await?;
                };
                async {
                    let (mut base_request, mut ingredient_request, mut combine_request) = (
                        vec![0; crate::shell::GUEST_MEMORY_BYTES],
                        vec![0; crate::shell::GUEST_MEMORY_BYTES],
                        vec![0; crate::shell::GUEST_MEMORY_BYTES],
                    );
                    let (mut base_result, mut ingredient_result, mut dish_result) = (
                        vec![0; incoming_capacity],
                        vec![0; incoming_capacity],
                        vec![0; incoming_capacity],
                    );
                    let files = ChoreoFsFiles::new(b_vm)
                        .file(
                            ChoreoFsFile::new(FILES[0], base_file_b)
                                .write_as::<BaseIntentB>(&mut base_request)
                                .read_as::<BaseResultB>(&mut base_result),
                        )?
                        .file(
                            ChoreoFsFile::new(FILES[1], ingredient_file_b)
                                .write_as::<PrepareGarnish>(&mut ingredient_request)
                                .read_as::<GarnishPrepared>(&mut ingredient_result),
                        )?
                        .file(
                            ChoreoFsFile::new(FILES[2], combine_file_b)
                                .write_as::<CombineB>(&mut combine_request)
                                .read_as::<DishB>(&mut dish_result),
                        )?;
                    let mut memory = vec![0; crate::shell::GUEST_MEMORY_BYTES];
                    let mut storage = HibanaWasiGuestStorage::uninit();
                    let guest = storage.init(module, GuestMemory::new(&mut memory), files)?;
                    let (exit, files) = guest.run().await?;
                    let status = exit.status() as i32;
                    let (mut root, (base, (ingredient, (combine, ())))) = files.into_parts();
                    async {
                        async {
                            base.into_endpoint().send::<FileExit>(&status).await?;
                        };
                        async {
                            ingredient.into_endpoint().send::<FileExit>(&status).await?;
                        };
                        async {
                            combine.into_endpoint().send::<FileExit>(&status).await?;
                        };
                        Ok::<(), ShellError>(())
                    }
                    .await?;
                    root.send::<crate::shell::global::Exit>(&status).await?;
                };
                async {
                    let (mut base_request, mut ingredient_request, mut combine_request) = (
                        vec![0; crate::shell::GUEST_MEMORY_BYTES],
                        vec![0; crate::shell::GUEST_MEMORY_BYTES],
                        vec![0; crate::shell::GUEST_MEMORY_BYTES],
                    );
                    let (mut base_result, mut ingredient_result, mut dish_result) = (
                        vec![0; incoming_capacity],
                        vec![0; incoming_capacity],
                        vec![0; incoming_capacity],
                    );
                    let files = ChoreoFsFiles::new(c_vm)
                        .file(
                            ChoreoFsFile::new(FILES[0], base_file_c)
                                .write_as::<BaseIntentC>(&mut base_request)
                                .read_as::<BaseResultC>(&mut base_result),
                        )?
                        .file(
                            ChoreoFsFile::new(FILES[1], ingredient_file_c)
                                .write_as::<PrepareGlaze>(&mut ingredient_request)
                                .read_as::<GlazePrepared>(&mut ingredient_result),
                        )?
                        .file(
                            ChoreoFsFile::new(FILES[2], combine_file_c)
                                .write_as::<CombineC>(&mut combine_request)
                                .read_as::<DishC>(&mut dish_result),
                        )?;
                    let mut memory = vec![0; crate::shell::GUEST_MEMORY_BYTES];
                    let mut storage = HibanaWasiGuestStorage::uninit();
                    let guest = storage.init(module, GuestMemory::new(&mut memory), files)?;
                    let (exit, files) = guest.run().await?;
                    let status = exit.status() as i32;
                    let (mut root, (base, (ingredient, (combine, ())))) = files.into_parts();
                    async {
                        async {
                            base.into_endpoint().send::<FileExit>(&status).await?;
                        };
                        async {
                            ingredient.into_endpoint().send::<FileExit>(&status).await?;
                        };
                        async {
                            combine.into_endpoint().send::<FileExit>(&status).await?;
                        };
                        Ok::<(), ShellError>(())
                    }
                    .await?;
                    root.send::<crate::shell::global::Exit>(&status).await?;
                };
                Ok::<(), ShellError>(())
            }
            .await;
            bus.checkpoint(SESSION, observations.checkpoint());
            result
        },
        io.run(bus),
    )
    .await;
    match &result{
 Ok(())=>io.emit(bus,serde_json::json!({"type":"run_result","status":"complete"})).await?,
 Err(error)=>io.emit(bus,serde_json::json!({"type":"run_result","status":"failed","error":error.to_string()})).await?,
 }
    result
}
pub fn projection() -> serde_json::Value {
    global::programs().observation()
}
