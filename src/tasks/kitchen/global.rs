//! Exclusive equipment, independent ingredients and per-dish completion.
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
pub const FIRST_A: u16 = 960;
pub const FIRST_B: u16 = 961;
pub const NEXT: u16 = 962;
pub const SESSION: u32 = 120;

pub type BaseIntentA = g::Msg<1, Arguments<0>>;
pub type BaseIntentB = g::Msg<2, Arguments<1>>;
pub type BaseIntentC = g::Msg<3, Arguments<2>>;
pub type OvenLoan<'r> = g::Msg<4, Endpoint<'r, OVEN>>;
pub type StoveLoan<'r> = g::Msg<5, Endpoint<'r, STOVE>>;
pub type OvenRequestA = g::Msg<6, Arguments<0>>;
pub type OvenRequestB = g::Msg<7, Arguments<1>>;
pub type OvenRequestC = g::Msg<8, Arguments<2>>;
pub type StoveRequestA = g::Msg<9, Arguments<0>>;
pub type StoveRequestB = g::Msg<10, Arguments<1>>;
pub type OvenResult<'a> = g::Msg<11, &'a [u8]>;
pub type StoveResult<'a> = g::Msg<12, &'a [u8]>;
pub type ReturnOven<'r> = g::Msg<13, Endpoint<'r, OVEN>>;
pub type ReturnStove<'r> = g::Msg<14, Endpoint<'r, STOVE>>;
pub type PrepareSauce = g::Msg<15, Arguments<3>>;
pub type PrepareGarnish = g::Msg<16, Arguments<4>>;
pub type PrepareGlaze = g::Msg<17, Arguments<5>>;
pub type SaucePrepared<'a> = g::Msg<18, &'a [u8]>;
pub type GarnishPrepared<'a> = g::Msg<19, &'a [u8]>;
pub type GlazePrepared<'a> = g::Msg<20, &'a [u8]>;
pub type Sauce<'a> = g::Msg<21, &'a [u8]>;
pub type Garnish<'a> = g::Msg<22, &'a [u8]>;
pub type Glaze<'a> = g::Msg<23, &'a [u8]>;
pub type CombineA = g::Msg<24, Arguments<6>>;
pub type CombineB = g::Msg<25, Arguments<7>>;
pub type CombineC = g::Msg<26, Arguments<8>>;
pub type DishA<'a> = g::Msg<27, &'a [u8]>;
pub type DishB<'a> = g::Msg<28, &'a [u8]>;
pub type DishC<'a> = g::Msg<29, &'a [u8]>;

fn two_resources<const CHEF: u8, O: g::Message, S: g::Message>() -> impl Projectable {
    g::seq(
        g::par(
            g::send::<COOKING, CHEF, OvenLoan>(),
            g::send::<COOKING, CHEF, StoveLoan>(),
        ),
        g::seq(
            g::par(
                g::send::<OVEN, COOKING, O>(),
                g::send::<STOVE, COOKING, S>(),
            ),
            g::seq(
                g::par(
                    g::send::<COOKING, OVEN, OvenResult>(),
                    g::send::<COOKING, STOVE, StoveResult>(),
                ),
                g::par(
                    g::send::<CHEF, COOKING, ReturnOven>(),
                    g::send::<CHEF, COOKING, ReturnStove>(),
                ),
            ),
        ),
    )
}
fn a() -> impl Projectable {
    two_resources::<A, OvenRequestA, StoveRequestA>()
}
fn b() -> impl Projectable {
    two_resources::<B, OvenRequestB, StoveRequestB>()
}
fn c() -> impl Projectable {
    g::seq(
        g::send::<COOKING, C, OvenLoan>(),
        g::seq(
            g::send::<OVEN, COOKING, OvenRequestC>(),
            g::seq(
                g::send::<COOKING, OVEN, OvenResult>(),
                g::send::<C, COOKING, ReturnOven>(),
            ),
        ),
    )
}

pub fn source() -> impl Projectable {
    let intents = g::par(
        g::send::<A, COOKING, BaseIntentA>(),
        g::par(
            g::send::<B, COOKING, BaseIntentB>(),
            g::send::<C, COOKING, BaseIntentC>(),
        ),
    );
    let equipment = g::route(
        g::seq(
            a(),
            g::route(g::seq(b(), c()), g::seq(c(), b())).resolve::<NEXT>(),
        ),
        g::route(
            g::seq(
                b(),
                g::route(g::seq(a(), c()), g::seq(c(), a())).resolve::<NEXT>(),
            ),
            g::seq(
                c(),
                g::route(g::seq(a(), b()), g::seq(b(), a())).resolve::<NEXT>(),
            ),
        )
        .resolve::<FIRST_B>(),
    )
    .resolve::<FIRST_A>();
    // The actual ingredient goes directly to its consumer as soon as it exists;
    // that transmission does not wait for the producer's oven turn.
    let sauce = g::seq(
        g::send::<A, INGREDIENTS, PrepareSauce>(),
        g::par(
            g::send::<INGREDIENTS, A, SaucePrepared>(),
            g::send::<INGREDIENTS, C, Sauce>(),
        ),
    );
    let garnish = g::seq(
        g::send::<B, INGREDIENTS, PrepareGarnish>(),
        g::par(
            g::send::<INGREDIENTS, B, GarnishPrepared>(),
            g::send::<INGREDIENTS, A, Garnish>(),
        ),
    );
    let glaze = g::seq(
        g::send::<C, INGREDIENTS, PrepareGlaze>(),
        g::par(
            g::send::<INGREDIENTS, C, GlazePrepared>(),
            g::send::<INGREDIENTS, B, Glaze>(),
        ),
    );
    let preparation = g::par(
        g::par(intents, equipment),
        g::par(sauce, g::par(garnish, glaze)),
    );
    let dishes = g::par(
        g::seq(
            g::send::<A, FINISH, CombineA>(),
            g::send::<FINISH, A, DishA>(),
        ),
        g::par(
            g::seq(
                g::send::<B, FINISH, CombineB>(),
                g::send::<FINISH, B, DishB>(),
            ),
            g::seq(
                g::send::<C, FINISH, CombineC>(),
                g::send::<FINISH, C, DishC>(),
            ),
        ),
    );
    g::par(
        g::seq(preparation, dishes),
        g::par(
            crate::shell::global::private_io::<A, A_IO>(),
            g::par(
                crate::shell::global::private_io::<B, B_IO>(),
                crate::shell::global::private_io::<C, C_IO>(),
            ),
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
    }
}

impl Programs {
    pub(crate) fn observation(&self) -> serde_json::Value {
        use crate::imp::projection::rows;
        use hibana::g::Message;
        serde_json::json!({"type":"compiled_projection","format_version":1,"task":"12E","session":SESSION,
            "roles":{"CHEF_A":A,"CHEF_B":B,"CHEF_C":C,"COOKING":COOKING,"INGREDIENTS":INGREDIENTS,"FINISH":FINISH,"OVEN":OVEN,"STOVETOP":STOVE,"A_IO":A_IO,"B_IO":B_IO,"C_IO":C_IO},
            "labels":{"BaseIntentA":BaseIntentA::LOGICAL_LABEL,"BaseIntentB":BaseIntentB::LOGICAL_LABEL,"BaseIntentC":BaseIntentC::LOGICAL_LABEL,"OvenLoan":OvenLoan::LOGICAL_LABEL,"StoveLoan":StoveLoan::LOGICAL_LABEL,"OvenRequestA":OvenRequestA::LOGICAL_LABEL,"OvenRequestB":OvenRequestB::LOGICAL_LABEL,"OvenRequestC":OvenRequestC::LOGICAL_LABEL,"StoveRequestA":StoveRequestA::LOGICAL_LABEL,"StoveRequestB":StoveRequestB::LOGICAL_LABEL,"OvenResult":OvenResult::LOGICAL_LABEL,"StoveResult":StoveResult::LOGICAL_LABEL,"ReturnOven":ReturnOven::LOGICAL_LABEL,"ReturnStove":ReturnStove::LOGICAL_LABEL,"PrepareSauce":PrepareSauce::LOGICAL_LABEL,"PrepareGarnish":PrepareGarnish::LOGICAL_LABEL,"PrepareGlaze":PrepareGlaze::LOGICAL_LABEL,"SaucePrepared":SaucePrepared::LOGICAL_LABEL,"GarnishPrepared":GarnishPrepared::LOGICAL_LABEL,"GlazePrepared":GlazePrepared::LOGICAL_LABEL,"Sauce":Sauce::LOGICAL_LABEL,"Garnish":Garnish::LOGICAL_LABEL,"Glaze":Glaze::LOGICAL_LABEL,"CombineA":CombineA::LOGICAL_LABEL,"CombineB":CombineB::LOGICAL_LABEL,"CombineC":CombineC::LOGICAL_LABEL,"DishA":DishA::LOGICAL_LABEL,"DishB":DishB::LOGICAL_LABEL,"DishC":DishC::LOGICAL_LABEL},
            "programs":[rows(&self.a),rows(&self.b),rows(&self.c),rows(&self.cooking),rows(&self.ingredients),rows(&self.finish),rows(&self.oven),rows(&self.stove),rows(&self.a_io),rows(&self.b_io),rows(&self.c_io)]})
    }
}
