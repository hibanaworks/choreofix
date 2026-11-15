//! Actual file requests choose equipment order; preparation and per-dish finish stay independent.
use super::imp::Arguments;
use hibana::{
    Endpoint, g,
    runtime::program::{Projectable, RoleProgram, project},
};
pub const A: u8 = 0;
pub const B: u8 = 1;
pub const C: u8 = 2;
pub const COOKING: u8 = 3;
pub const INGREDIENTS: u8 = 4;
pub const FINISH: u8 = 5;
pub const OVEN: u8 = 6;
pub const STOVE: u8 = 7;
pub const A_IO: u8 = 8;
pub const B_IO: u8 = 9;
pub const C_IO: u8 = 10;
pub const BASE_FILE_A: u8 = 16;
pub const BASE_FILE_B: u8 = 17;
pub const BASE_FILE_C: u8 = 18;
pub const INGREDIENT_FILE_A: u8 = 19;
pub const INGREDIENT_FILE_B: u8 = 20;
pub const INGREDIENT_FILE_C: u8 = 21;
pub const COMBINE_FILE_A: u8 = 22;
pub const COMBINE_FILE_B: u8 = 23;
pub const COMBINE_FILE_C: u8 = 24;
pub const A_VM: u8 = 32;
pub const B_VM: u8 = 33;
pub const C_VM: u8 = 34;
pub const BASE_DATA_A: u8 = 48;
pub const BASE_DATA_B: u8 = 49;
pub const BASE_DATA_C: u8 = 50;
pub const INGREDIENT_DATA_A: u8 = 51;
pub const INGREDIENT_DATA_B: u8 = 52;
pub const INGREDIENT_DATA_C: u8 = 53;
pub const COMBINE_DATA_A: u8 = 54;
pub const COMBINE_DATA_B: u8 = 55;
pub const COMBINE_DATA_C: u8 = 56;
pub const SESSION: u32 = 120;
pub use crate::shell::global::{FileExit, Loan};
pub type BaseIntentA = g::Msg<1, Arguments<0>>;
pub type BaseIntentB = g::Msg<2, Arguments<1>>;
pub type BaseIntentC = g::Msg<3, Arguments<2>>;
pub type OvenLoanA<'r> = g::Msg<4, Endpoint<'r, OVEN>>;
pub type StoveLoanA<'r> = g::Msg<5, Endpoint<'r, STOVE>>;
pub type OvenLoanB<'r> = g::Msg<6, Endpoint<'r, OVEN>>;
pub type StoveLoanB<'r> = g::Msg<7, Endpoint<'r, STOVE>>;
pub type OvenLoanC<'r> = g::Msg<8, Endpoint<'r, OVEN>>;
pub type OvenRequestA = g::Msg<9, Arguments<0>>;
pub type StoveRequestA = g::Msg<10, Arguments<0>>;
pub type OvenRequestB = g::Msg<11, Arguments<1>>;
pub type StoveRequestB = g::Msg<12, Arguments<1>>;
pub type OvenRequestC = g::Msg<13, Arguments<2>>;
pub type OvenResultA<'a> = g::Msg<14, &'a [u8]>;
pub type StoveResultA<'a> = g::Msg<15, &'a [u8]>;
pub type OvenResultB<'a> = g::Msg<16, &'a [u8]>;
pub type StoveResultB<'a> = g::Msg<17, &'a [u8]>;
pub type OvenResultC<'a> = g::Msg<18, &'a [u8]>;
pub type ReturnOvenA<'r> = g::Msg<19, Endpoint<'r, OVEN>>;
pub type ReturnStoveA<'r> = g::Msg<20, Endpoint<'r, STOVE>>;
pub type ReturnOvenB<'r> = g::Msg<21, Endpoint<'r, OVEN>>;
pub type ReturnStoveB<'r> = g::Msg<22, Endpoint<'r, STOVE>>;
pub type ReturnOvenC<'r> = g::Msg<23, Endpoint<'r, OVEN>>;
pub type BaseResultA<'a> = g::Msg<24, &'a [u8]>;
pub type BaseResultB<'a> = g::Msg<25, &'a [u8]>;
pub type BaseResultC<'a> = g::Msg<26, &'a [u8]>;
pub type PrepareSauce = g::Msg<27, Arguments<3>>;
pub type PrepareGarnish = g::Msg<28, Arguments<4>>;
pub type PrepareGlaze = g::Msg<29, Arguments<5>>;
pub type SaucePrepared<'a> = g::Msg<30, &'a [u8]>;
pub type GarnishPrepared<'a> = g::Msg<31, &'a [u8]>;
pub type GlazePrepared<'a> = g::Msg<32, &'a [u8]>;
pub type Sauce<'a> = g::Msg<33, &'a [u8]>;
pub type Garnish<'a> = g::Msg<34, &'a [u8]>;
pub type Glaze<'a> = g::Msg<35, &'a [u8]>;
pub type CombineA = g::Msg<36, Arguments<6>>;
pub type CombineB = g::Msg<37, Arguments<7>>;
pub type CombineC = g::Msg<38, Arguments<8>>;
pub type DishA<'a> = g::Msg<39, &'a [u8]>;
pub type DishB<'a> = g::Msg<40, &'a [u8]>;
pub type DishC<'a> = g::Msg<41, &'a [u8]>;
const fn object(
    path: &'static [u8],
    fd: u8,
) -> hibana_wasip1_runtime::choreofs::ChoreoFsObject<'static> {
    use hibana_wasip1_runtime::{
        choreofs::{ChoreoFsObject, FdSpec, ObjectId},
        protocol,
    };
    ChoreoFsObject::read_write(
        path,
        ObjectId(fd as u32),
        FdSpec::new(fd, 66, 1),
        b"",
        protocol::FdBinding {
            read: Some(protocol::FdReadRow::Base),
            write: Some(protocol::FdWriteRow::Object),
            readdir: None,
        },
    )
}
pub const FILES: [hibana_wasip1_runtime::choreofs::ChoreoFsObject<'static>; 3] = [
    object(b"service/prepare_base_dish", 4),
    object(b"service/prepare_ingredient", 5),
    object(b"service/combine_dish", 6),
];
pub fn source() -> impl Projectable {
    let head_a = g::seq(
        g::send::<BASE_FILE_A, A, BaseIntentA>(),
        g::seq(
            g::par(
                g::send::<COOKING, A, OvenLoanA>(),
                g::send::<COOKING, A, StoveLoanA>(),
            ),
            g::seq(
                g::par(
                    g::send::<OVEN, A, OvenRequestA>(),
                    g::send::<STOVE, A, StoveRequestA>(),
                ),
                g::par(
                    g::send::<A, OVEN, OvenResultA>(),
                    g::send::<A, STOVE, StoveResultA>(),
                ),
            ),
        ),
    );
    let reply_a = g::send::<A, BASE_FILE_A, BaseResultA>();
    let returns_a = g::par(
        g::send::<A, COOKING, ReturnOvenA>(),
        g::send::<A, COOKING, ReturnStoveA>(),
    );
    let last_a = g::seq(head_a, g::par(reply_a, returns_a));
    let head_b = g::seq(
        g::send::<BASE_FILE_B, B, BaseIntentB>(),
        g::seq(
            g::par(
                g::send::<COOKING, B, OvenLoanB>(),
                g::send::<COOKING, B, StoveLoanB>(),
            ),
            g::seq(
                g::par(
                    g::send::<OVEN, B, OvenRequestB>(),
                    g::send::<STOVE, B, StoveRequestB>(),
                ),
                g::par(
                    g::send::<B, OVEN, OvenResultB>(),
                    g::send::<B, STOVE, StoveResultB>(),
                ),
            ),
        ),
    );
    let reply_b = g::send::<B, BASE_FILE_B, BaseResultB>();
    let returns_b = g::par(
        g::send::<B, COOKING, ReturnOvenB>(),
        g::send::<B, COOKING, ReturnStoveB>(),
    );
    let last_b = g::seq(head_b, g::par(reply_b, returns_b));
    let head_c = g::seq(
        g::send::<BASE_FILE_C, C, BaseIntentC>(),
        g::seq(
            g::send::<COOKING, C, OvenLoanC>(),
            g::seq(
                g::send::<OVEN, C, OvenRequestC>(),
                g::send::<C, OVEN, OvenResultC>(),
            ),
        ),
    );
    let reply_c = g::send::<C, BASE_FILE_C, BaseResultC>();
    let returns_c = g::send::<C, COOKING, ReturnOvenC>();
    let last_c = g::seq(head_c, g::par(reply_c, returns_c));
    // Replies can remain unread while returned equipment serves another chef.
    let equipment = g::route(
        g::seq(
            head_a,
            g::par(
                reply_a,
                g::seq(
                    returns_a,
                    g::route(
                        g::seq(head_b, g::par(reply_b, g::seq(returns_b, last_c))),
                        g::seq(head_c, g::par(reply_c, g::seq(returns_c, last_b))),
                    ),
                ),
            ),
        ),
        g::route(
            g::seq(
                head_b,
                g::par(
                    reply_b,
                    g::seq(
                        returns_b,
                        g::route(
                            g::seq(head_a, g::par(reply_a, g::seq(returns_a, last_c))),
                            g::seq(head_c, g::par(reply_c, g::seq(returns_c, last_a))),
                        ),
                    ),
                ),
            ),
            g::seq(
                head_c,
                g::par(
                    reply_c,
                    g::seq(
                        returns_c,
                        g::route(
                            g::seq(head_a, g::par(reply_a, g::seq(returns_a, last_b))),
                            g::seq(head_b, g::par(reply_b, g::seq(returns_b, last_a))),
                        ),
                    ),
                ),
            ),
        ),
    );
    let ingredients = g::par(
        g::seq(
            g::send::<INGREDIENT_FILE_A, INGREDIENTS, PrepareSauce>(),
            g::par(
                g::send::<INGREDIENTS, INGREDIENT_FILE_A, SaucePrepared>(),
                g::send::<INGREDIENTS, C, Sauce>(),
            ),
        ),
        g::par(
            g::seq(
                g::send::<INGREDIENT_FILE_B, INGREDIENTS, PrepareGarnish>(),
                g::par(
                    g::send::<INGREDIENTS, INGREDIENT_FILE_B, GarnishPrepared>(),
                    g::send::<INGREDIENTS, A, Garnish>(),
                ),
            ),
            g::seq(
                g::send::<INGREDIENT_FILE_C, INGREDIENTS, PrepareGlaze>(),
                g::par(
                    g::send::<INGREDIENTS, INGREDIENT_FILE_C, GlazePrepared>(),
                    g::send::<INGREDIENTS, B, Glaze>(),
                ),
            ),
        ),
    );
    let dishes = g::par(
        g::seq(
            g::send::<COMBINE_FILE_A, FINISH, CombineA>(),
            g::send::<FINISH, COMBINE_FILE_A, DishA>(),
        ),
        g::par(
            g::seq(
                g::send::<COMBINE_FILE_B, FINISH, CombineB>(),
                g::send::<FINISH, COMBINE_FILE_B, DishB>(),
            ),
            g::seq(
                g::send::<COMBINE_FILE_C, FINISH, CombineC>(),
                g::send::<FINISH, COMBINE_FILE_C, DishC>(),
            ),
        ),
    );
    let bootstrap = g::par(
        g::par(
            g::par(
                g::send::<BASE_DATA_A, COOKING, Loan<BASE_FILE_A>>(),
                g::send::<BASE_DATA_B, COOKING, Loan<BASE_FILE_B>>(),
            ),
            g::par(
                g::send::<BASE_DATA_C, COOKING, Loan<BASE_FILE_C>>(),
                g::send::<INGREDIENT_DATA_A, INGREDIENTS, Loan<INGREDIENT_FILE_A>>(),
            ),
        ),
        g::par(
            g::par(
                g::send::<INGREDIENT_DATA_B, INGREDIENTS, Loan<INGREDIENT_FILE_B>>(),
                g::send::<INGREDIENT_DATA_C, INGREDIENTS, Loan<INGREDIENT_FILE_C>>(),
            ),
            g::par(
                g::send::<COMBINE_DATA_A, A, Loan<COMBINE_FILE_A>>(),
                g::par(
                    g::send::<COMBINE_DATA_B, B, Loan<COMBINE_FILE_B>>(),
                    g::send::<COMBINE_DATA_C, C, Loan<COMBINE_FILE_C>>(),
                ),
            ),
        ),
    );
    let private = g::par(
        g::seq(
            crate::shell::global::private_io::<A_VM, A_IO>(),
            g::send::<A_VM, A_IO, crate::shell::global::Exit>(),
        ),
        g::par(
            g::seq(
                crate::shell::global::private_io::<B_VM, B_IO>(),
                g::send::<B_VM, B_IO, crate::shell::global::Exit>(),
            ),
            g::seq(
                crate::shell::global::private_io::<C_VM, C_IO>(),
                g::send::<C_VM, C_IO, crate::shell::global::Exit>(),
            ),
        ),
    );
    let metadata = g::par(
        g::par(
            g::par(
                g::seq(
                    crate::shell::global::file_metadata::<BASE_FILE_A, BASE_DATA_A>().roll(),
                    g::send::<BASE_FILE_A, BASE_DATA_A, FileExit>(),
                ),
                g::seq(
                    crate::shell::global::file_metadata::<BASE_FILE_B, BASE_DATA_B>().roll(),
                    g::send::<BASE_FILE_B, BASE_DATA_B, FileExit>(),
                ),
            ),
            g::par(
                g::seq(
                    crate::shell::global::file_metadata::<BASE_FILE_C, BASE_DATA_C>().roll(),
                    g::send::<BASE_FILE_C, BASE_DATA_C, FileExit>(),
                ),
                g::seq(
                    crate::shell::global::file_metadata::<INGREDIENT_FILE_A, INGREDIENT_DATA_A>()
                        .roll(),
                    g::send::<INGREDIENT_FILE_A, INGREDIENT_DATA_A, FileExit>(),
                ),
            ),
        ),
        g::par(
            g::par(
                g::seq(
                    crate::shell::global::file_metadata::<INGREDIENT_FILE_B, INGREDIENT_DATA_B>()
                        .roll(),
                    g::send::<INGREDIENT_FILE_B, INGREDIENT_DATA_B, FileExit>(),
                ),
                g::seq(
                    crate::shell::global::file_metadata::<INGREDIENT_FILE_C, INGREDIENT_DATA_C>()
                        .roll(),
                    g::send::<INGREDIENT_FILE_C, INGREDIENT_DATA_C, FileExit>(),
                ),
            ),
            g::par(
                g::seq(
                    crate::shell::global::file_metadata::<COMBINE_FILE_A, COMBINE_DATA_A>().roll(),
                    g::send::<COMBINE_FILE_A, COMBINE_DATA_A, FileExit>(),
                ),
                g::par(
                    g::seq(
                        crate::shell::global::file_metadata::<COMBINE_FILE_B, COMBINE_DATA_B>()
                            .roll(),
                        g::send::<COMBINE_FILE_B, COMBINE_DATA_B, FileExit>(),
                    ),
                    g::seq(
                        crate::shell::global::file_metadata::<COMBINE_FILE_C, COMBINE_DATA_C>()
                            .roll(),
                        g::send::<COMBINE_FILE_C, COMBINE_DATA_C, FileExit>(),
                    ),
                ),
            ),
        ),
    );
    g::seq(
        bootstrap,
        g::par(
            g::seq(g::par(equipment, ingredients), dishes),
            g::par(private, metadata),
        ),
    )
}
pub struct Programs {
    pub a: RoleProgram<A>,
    pub b: RoleProgram<B>,
    pub c: RoleProgram<C>,
    pub cooking: RoleProgram<COOKING>,
    pub ingredients: RoleProgram<INGREDIENTS>,
    pub finish: RoleProgram<FINISH>,
    pub oven: RoleProgram<OVEN>,
    pub stove: RoleProgram<STOVE>,
    pub a_io: RoleProgram<A_IO>,
    pub b_io: RoleProgram<B_IO>,
    pub c_io: RoleProgram<C_IO>,
    pub base_file_a: RoleProgram<BASE_FILE_A>,
    pub base_file_b: RoleProgram<BASE_FILE_B>,
    pub base_file_c: RoleProgram<BASE_FILE_C>,
    pub ingredient_file_a: RoleProgram<INGREDIENT_FILE_A>,
    pub ingredient_file_b: RoleProgram<INGREDIENT_FILE_B>,
    pub ingredient_file_c: RoleProgram<INGREDIENT_FILE_C>,
    pub combine_file_a: RoleProgram<COMBINE_FILE_A>,
    pub combine_file_b: RoleProgram<COMBINE_FILE_B>,
    pub combine_file_c: RoleProgram<COMBINE_FILE_C>,
    pub a_vm: RoleProgram<A_VM>,
    pub b_vm: RoleProgram<B_VM>,
    pub c_vm: RoleProgram<C_VM>,
    pub base_data_a: RoleProgram<BASE_DATA_A>,
    pub base_data_b: RoleProgram<BASE_DATA_B>,
    pub base_data_c: RoleProgram<BASE_DATA_C>,
    pub ingredient_data_a: RoleProgram<INGREDIENT_DATA_A>,
    pub ingredient_data_b: RoleProgram<INGREDIENT_DATA_B>,
    pub ingredient_data_c: RoleProgram<INGREDIENT_DATA_C>,
    pub combine_data_a: RoleProgram<COMBINE_DATA_A>,
    pub combine_data_b: RoleProgram<COMBINE_DATA_B>,
    pub combine_data_c: RoleProgram<COMBINE_DATA_C>,
}
pub fn programs() -> Programs {
    let source = source();
    Programs {
        a: project(&source),
        b: project(&source),
        c: project(&source),
        cooking: project(&source),
        ingredients: project(&source),
        finish: project(&source),
        oven: project(&source),
        stove: project(&source),
        a_io: project(&source),
        b_io: project(&source),
        c_io: project(&source),
        base_file_a: project(&source),
        base_file_b: project(&source),
        base_file_c: project(&source),
        ingredient_file_a: project(&source),
        ingredient_file_b: project(&source),
        ingredient_file_c: project(&source),
        combine_file_a: project(&source),
        combine_file_b: project(&source),
        combine_file_c: project(&source),
        a_vm: project(&source),
        b_vm: project(&source),
        c_vm: project(&source),
        base_data_a: project(&source),
        base_data_b: project(&source),
        base_data_c: project(&source),
        ingredient_data_a: project(&source),
        ingredient_data_b: project(&source),
        ingredient_data_c: project(&source),
        combine_data_a: project(&source),
        combine_data_b: project(&source),
        combine_data_c: project(&source),
    }
}
impl Programs {
    pub fn observation(&self) -> serde_json::Value {
        use crate::imp::projection::rows;
        use hibana::g::Message;
        let roles = serde_json::json!({
            "CHEF_A":A,
            "CHEF_B":B,
            "CHEF_C":C,
            "COOKING":COOKING,
            "INGREDIENTS":INGREDIENTS,
            "FINISH":FINISH,
            "OVEN":OVEN,
            "STOVETOP":STOVE,
            "A_IO":A_IO,
            "B_IO":B_IO,
            "C_IO":C_IO,
            "BASE_FILE_A":BASE_FILE_A,
            "BASE_FILE_B":BASE_FILE_B,
            "BASE_FILE_C":BASE_FILE_C,
            "INGREDIENT_FILE_A":INGREDIENT_FILE_A,
            "INGREDIENT_FILE_B":INGREDIENT_FILE_B,
            "INGREDIENT_FILE_C":INGREDIENT_FILE_C,
            "COMBINE_FILE_A":COMBINE_FILE_A,
            "COMBINE_FILE_B":COMBINE_FILE_B,
            "COMBINE_FILE_C":COMBINE_FILE_C,
            "A_VM":A_VM,
            "B_VM":B_VM,
            "C_VM":C_VM,
            "BASE_DATA_A":BASE_DATA_A,
            "BASE_DATA_B":BASE_DATA_B,
            "BASE_DATA_C":BASE_DATA_C,
            "INGREDIENT_DATA_A":INGREDIENT_DATA_A,
            "INGREDIENT_DATA_B":INGREDIENT_DATA_B,
            "INGREDIENT_DATA_C":INGREDIENT_DATA_C,
            "COMBINE_DATA_A":COMBINE_DATA_A,
            "COMBINE_DATA_B":COMBINE_DATA_B,
            "COMBINE_DATA_C":COMBINE_DATA_C
        });
        let labels = serde_json::json!({
            "BaseIntentA":BaseIntentA::LOGICAL_LABEL,
            "BaseIntentB":BaseIntentB::LOGICAL_LABEL,
            "BaseIntentC":BaseIntentC::LOGICAL_LABEL,
            "OvenLoanA":OvenLoanA::LOGICAL_LABEL,
            "StoveLoanA":StoveLoanA::LOGICAL_LABEL,
            "OvenLoanB":OvenLoanB::LOGICAL_LABEL,
            "StoveLoanB":StoveLoanB::LOGICAL_LABEL,
            "OvenLoanC":OvenLoanC::LOGICAL_LABEL,
            "OvenRequestA":OvenRequestA::LOGICAL_LABEL,
            "StoveRequestA":StoveRequestA::LOGICAL_LABEL,
            "OvenRequestB":OvenRequestB::LOGICAL_LABEL,
            "StoveRequestB":StoveRequestB::LOGICAL_LABEL,
            "OvenRequestC":OvenRequestC::LOGICAL_LABEL,
            "OvenResultA":OvenResultA::LOGICAL_LABEL,
            "StoveResultA":StoveResultA::LOGICAL_LABEL,
            "OvenResultB":OvenResultB::LOGICAL_LABEL,
            "StoveResultB":StoveResultB::LOGICAL_LABEL,
            "OvenResultC":OvenResultC::LOGICAL_LABEL,
            "ReturnOvenA":ReturnOvenA::LOGICAL_LABEL,
            "ReturnStoveA":ReturnStoveA::LOGICAL_LABEL,
            "ReturnOvenB":ReturnOvenB::LOGICAL_LABEL,
            "ReturnStoveB":ReturnStoveB::LOGICAL_LABEL,
            "ReturnOvenC":ReturnOvenC::LOGICAL_LABEL,
            "BaseResultA":BaseResultA::LOGICAL_LABEL,
            "BaseResultB":BaseResultB::LOGICAL_LABEL,
            "BaseResultC":BaseResultC::LOGICAL_LABEL,
            "PrepareSauce":PrepareSauce::LOGICAL_LABEL,
            "PrepareGarnish":PrepareGarnish::LOGICAL_LABEL,
            "PrepareGlaze":PrepareGlaze::LOGICAL_LABEL,
            "SaucePrepared":SaucePrepared::LOGICAL_LABEL,
            "GarnishPrepared":GarnishPrepared::LOGICAL_LABEL,
            "GlazePrepared":GlazePrepared::LOGICAL_LABEL,
            "Sauce":Sauce::LOGICAL_LABEL,
            "Garnish":Garnish::LOGICAL_LABEL,
            "Glaze":Glaze::LOGICAL_LABEL,
            "CombineA":CombineA::LOGICAL_LABEL,
            "CombineB":CombineB::LOGICAL_LABEL,
            "CombineC":CombineC::LOGICAL_LABEL,
            "DishA":DishA::LOGICAL_LABEL,
            "DishB":DishB::LOGICAL_LABEL,
            "DishC":DishC::LOGICAL_LABEL
        });
        let programs = [
            rows(&self.a),
            rows(&self.b),
            rows(&self.c),
            rows(&self.cooking),
            rows(&self.ingredients),
            rows(&self.finish),
            rows(&self.oven),
            rows(&self.stove),
            rows(&self.a_io),
            rows(&self.b_io),
            rows(&self.c_io),
            rows(&self.base_file_a),
            rows(&self.base_file_b),
            rows(&self.base_file_c),
            rows(&self.ingredient_file_a),
            rows(&self.ingredient_file_b),
            rows(&self.ingredient_file_c),
            rows(&self.combine_file_a),
            rows(&self.combine_file_b),
            rows(&self.combine_file_c),
            rows(&self.a_vm),
            rows(&self.b_vm),
            rows(&self.c_vm),
            rows(&self.base_data_a),
            rows(&self.base_data_b),
            rows(&self.base_data_c),
            rows(&self.ingredient_data_a),
            rows(&self.ingredient_data_b),
            rows(&self.ingredient_data_c),
            rows(&self.combine_data_a),
            rows(&self.combine_data_b),
            rows(&self.combine_data_c),
        ];
        serde_json::json!({
            "type": "compiled_projection",
            "format_version": 1,
            "task": "12E",
            "session": SESSION,
            "roles": roles,
            "labels": labels,
            "programs": programs
        })
    }
}
