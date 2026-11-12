//! Three affine chapter continuations and two unique physical I/O owners.
use super::imp::{Arguments, Reply, Verdict};
use hibana::{Endpoint, g, runtime::program::Projectable};

pub const AUTHOR_A: u8 = 0;
pub const AUTHOR_B: u8 = 1;
pub const AUTHOR_C: u8 = 2;
pub const FACT_CHECKER: u8 = 3;
pub const EDITOR: u8 = 4;
pub const RESEARCH_SERVICE: u8 = 7;
pub const DOCUMENT: u8 = 8;
pub const DATABASE: u8 = 9;
pub const SECTION_A: u8 = 10;
pub const SECTION_B: u8 = 11;
pub const SECTION_C: u8 = 12;
pub const REVIEW_SERVICE: u8 = 13;
pub const ARCHIVE: u8 = 14;
pub const SESSION: u32 = 324;
pub type Section<'r, const ROLE: u8> = g::Msg<1, Endpoint<'r, ROLE>>;

// These are static wire contracts for the three actual chapter participants.
// They carry no runtime state. The same chapter choreography is instantiated
// three times; distinct labels let a physical owner identify requests directly.
pub trait ChapterMessages<const AUTHOR: u8> {
    type Research: g::Message<Payload = Arguments<AUTHOR, 1>>;
    type Researched: g::Message<Payload = Reply<AUTHOR, 1>>;
    type ResearchForDocument: g::Message<Payload = Reply<AUTHOR, 1>>;
    type ResearchForDatabase: g::Message<Payload = Reply<AUTHOR, 1>>;
    type Write: g::Message<Payload = Arguments<AUTHOR, 2>>;
    type References: g::Message<Payload = Arguments<AUTHOR, 4>>;
    type Written: g::Message<Payload = Reply<AUTHOR, 2>>;
    type Referenced: g::Message<Payload = Reply<AUTHOR, 4>>;
    type MaterialForFact: g::Message<Payload = Reply<AUTHOR, 2>>;
    type ReferencesForFact: g::Message<Payload = Reply<AUTHOR, 4>>;
    type FactMaterial: g::Message<Payload = super::imp::Material>;
    type FactRequest: g::Message<Payload = Arguments<AUTHOR, 5>>;
    type FactPassed: g::Message<Payload = Verdict<AUTHOR, 5, true>>;
    type FactRejected: g::Message<Payload = Verdict<AUTHOR, 5, false>>;
    type FactsForEditor: g::Message<Payload = Verdict<AUTHOR, 5, true>>;
    type FactRejectionForAuthor: g::Message<Payload = Verdict<AUTHOR, 5, false>>;
    type Revise: g::Message<Payload = Arguments<AUTHOR, 3>>;
    type Revised: g::Message<Payload = Reply<AUTHOR, 3>>;
    type RevisedForFact: g::Message<Payload = Reply<AUTHOR, 3>>;
    type ReviewMaterial: g::Message<Payload = super::imp::Material>;
    type ReviewRequest: g::Message<Payload = Arguments<AUTHOR, 6>>;
    type ReviewPassed: g::Message<Payload = Verdict<AUTHOR, 6, true>>;
    type ReviewRejected: g::Message<Payload = Verdict<AUTHOR, 6, false>>;
    type ReviewRejectionForAuthor: g::Message<Payload = Verdict<AUTHOR, 6, false>>;
    type FinalForAuthor: g::Message<Payload = Verdict<AUTHOR, 6, true>>;
    type FinalForFact: g::Message<Payload = Verdict<AUTHOR, 6, true>>;
    type FinalForDocument: g::Message<Payload = Verdict<AUTHOR, 6, true>>;
    type FailedFactsForEditor: g::Message<Payload = Verdict<AUTHOR, 5, false>>;
}
macro_rules! chapter_messages {
    ($module:ident,$author:expr,$base:expr) => {
        pub mod $module {
            use super::*;
            pub type Research = g::Msg<{ $base + 2 }, Arguments<$author, 1>>;
            pub type Researched = g::Msg<{ $base + 3 }, Reply<$author, 1>>;
            pub type ResearchForDocument = g::Msg<{ $base + 4 }, Reply<$author, 1>>;
            pub type ResearchForDatabase = g::Msg<{ $base + 5 }, Reply<$author, 1>>;
            pub type Write = g::Msg<{ $base + 6 }, Arguments<$author, 2>>;
            pub type References = g::Msg<{ $base + 7 }, Arguments<$author, 4>>;
            pub type Written = g::Msg<{ $base + 11 }, Reply<$author, 2>>;
            pub type Referenced = g::Msg<{ $base + 12 }, Reply<$author, 4>>;
            pub type MaterialForFact = g::Msg<{ $base + 13 }, Reply<$author, 2>>;
            pub type ReferencesForFact = g::Msg<{ $base + 14 }, Reply<$author, 4>>;
            pub type FactMaterial = g::Msg<{ $base + 15 }, super::super::imp::Material>;
            pub type FactRequest = g::Msg<{ $base + 16 }, Arguments<$author, 5>>;
            pub type FactPassed = g::Msg<{ $base + 17 }, Verdict<$author, 5, true>>;
            pub type FactRejected = g::Msg<{ $base + 18 }, Verdict<$author, 5, false>>;
            pub type FactsForEditor = g::Msg<{ $base + 19 }, Verdict<$author, 5, true>>;
            pub type FactRejectionForAuthor = g::Msg<{ $base + 20 }, Verdict<$author, 5, false>>;
            pub type Revise = g::Msg<{ $base + 21 }, Arguments<$author, 3>>;
            pub type Revised = g::Msg<{ $base + 23 }, Reply<$author, 3>>;
            pub type RevisedForFact = g::Msg<{ $base + 24 }, Reply<$author, 3>>;
            pub type ReviewMaterial = g::Msg<{ $base + 25 }, super::super::imp::Material>;
            pub type ReviewRequest = g::Msg<{ $base + 26 }, Arguments<$author, 6>>;
            pub type ReviewPassed = g::Msg<{ $base + 27 }, Verdict<$author, 6, true>>;
            pub type ReviewRejected = g::Msg<{ $base + 28 }, Verdict<$author, 6, false>>;
            pub type ReviewRejectionForAuthor = g::Msg<{ $base + 29 }, Verdict<$author, 6, false>>;
            pub type FinalForAuthor = g::Msg<{ $base + 30 }, Verdict<$author, 6, true>>;
            pub type FinalForFact = g::Msg<{ $base + 31 }, Verdict<$author, 6, true>>;
            pub type FinalForDocument = g::Msg<{ $base + 32 }, Verdict<$author, 6, true>>;
            pub type FailedFactsForEditor = g::Msg<{ $base + 33 }, Verdict<$author, 5, false>>;
        }
        impl ChapterMessages<$author> for $module::Research {
            type Research = $module::Research;
            type Researched = $module::Researched;
            type ResearchForDocument = $module::ResearchForDocument;
            type ResearchForDatabase = $module::ResearchForDatabase;
            type Write = $module::Write;
            type References = $module::References;
            type Written = $module::Written;
            type Referenced = $module::Referenced;
            type MaterialForFact = $module::MaterialForFact;
            type ReferencesForFact = $module::ReferencesForFact;
            type FactMaterial = $module::FactMaterial;
            type FactRequest = $module::FactRequest;
            type FactPassed = $module::FactPassed;
            type FactRejected = $module::FactRejected;
            type FactsForEditor = $module::FactsForEditor;
            type FactRejectionForAuthor = $module::FactRejectionForAuthor;
            type Revise = $module::Revise;
            type Revised = $module::Revised;
            type RevisedForFact = $module::RevisedForFact;
            type ReviewMaterial = $module::ReviewMaterial;
            type ReviewRequest = $module::ReviewRequest;
            type ReviewPassed = $module::ReviewPassed;
            type ReviewRejected = $module::ReviewRejected;
            type ReviewRejectionForAuthor = $module::ReviewRejectionForAuthor;
            type FinalForAuthor = $module::FinalForAuthor;
            type FinalForFact = $module::FinalForFact;
            type FinalForDocument = $module::FinalForDocument;
            type FailedFactsForEditor = $module::FailedFactsForEditor;
        }
    };
}
chapter_messages!(a, AUTHOR_A, 0);
chapter_messages!(b, AUTHOR_B, 64);
chapter_messages!(c, AUTHOR_C, 128);

pub type Combine = g::Msg<200, crate::shell::imp::report_arguments::ReportArguments<6>>;
pub type ReferencesForPublication = g::Msg<201, ()>;
pub type FrozenReferences<'a> = g::Msg<205, &'a [u8]>;
pub type FrozenReviews<'a> = g::Msg<206, &'a [u8]>;
pub type Published<'a> = g::Msg<209, &'a [u8]>;
pub type PublishedForDatabase<'a> = g::Msg<210, &'a [u8]>;

pub fn preparation<const AUTHOR: u8, const SECTION: u8, C: ChapterMessages<AUTHOR>>()
-> impl Projectable {
    const {
        assert!(AUTHOR <= AUTHOR_C && SECTION == SECTION_A + AUTHOR);
    }
    g::seq(
        g::send::<SECTION, RESEARCH_SERVICE, C::Research>(),
        g::seq(
            g::par(
                g::send::<RESEARCH_SERVICE, AUTHOR, C::Researched>(),
                g::par(
                    g::send::<RESEARCH_SERVICE, DOCUMENT, C::ResearchForDocument>(),
                    g::send::<RESEARCH_SERVICE, DATABASE, C::ResearchForDatabase>(),
                ),
            ),
            g::seq(
                g::par(
                    g::seq(
                        g::send::<SECTION, DOCUMENT, C::Write>(),
                        g::seq(
                            g::send::<DOCUMENT, SECTION, C::Written>(),
                            g::send::<DOCUMENT, FACT_CHECKER, C::MaterialForFact>(),
                        ),
                    ),
                    g::seq(
                        g::send::<SECTION, DATABASE, C::References>(),
                        g::seq(
                            g::send::<DATABASE, SECTION, C::Referenced>(),
                            g::send::<DATABASE, FACT_CHECKER, C::ReferencesForFact>(),
                        ),
                    ),
                ),
                g::send::<AUTHOR, FACT_CHECKER, Section<SECTION>>(),
            ),
        ),
    )
}

fn revision<const AUTHOR: u8, const SECTION: u8, C: ChapterMessages<AUTHOR>>() -> impl Projectable {
    g::seq(
        g::send::<SECTION, DOCUMENT, C::Revise>(),
        g::seq(
            g::send::<DOCUMENT, SECTION, C::Revised>(),
            g::seq(
                g::send::<DOCUMENT, FACT_CHECKER, C::RevisedForFact>(),
                g::send::<AUTHOR, FACT_CHECKER, Section<SECTION>>(),
            ),
        ),
    )
}

pub fn quality<const AUTHOR: u8, const SECTION: u8, C: ChapterMessages<AUTHOR>>() -> impl Projectable
{
    let factual_retry = g::seq(
        g::send::<SECTION, FACT_CHECKER, C::FactRejected>(),
        g::par(
            g::send::<REVIEW_SERVICE, EDITOR, C::FailedFactsForEditor>(),
            g::seq(
                g::send::<SECTION, AUTHOR, C::FactRejectionForAuthor>(),
                g::seq(
                    g::send::<REVIEW_SERVICE, AUTHOR, Section<SECTION>>(),
                    revision::<AUTHOR, SECTION, C>(),
                ),
            ),
        ),
    );
    let editorial_retry = g::seq(
        g::send::<SECTION, EDITOR, C::ReviewRejected>(),
        g::seq(
            g::send::<SECTION, AUTHOR, C::ReviewRejectionForAuthor>(),
            g::seq(
                g::send::<REVIEW_SERVICE, AUTHOR, Section<SECTION>>(),
                revision::<AUTHOR, SECTION, C>(),
            ),
        ),
    );
    let approved = g::seq(
        g::send::<SECTION, EDITOR, C::ReviewPassed>(),
        g::seq(
            g::send::<SECTION, AUTHOR, C::FinalForAuthor>(),
            g::seq(
                g::send::<SECTION, DOCUMENT, C::FinalForDocument>(),
                g::send::<REVIEW_SERVICE, FACT_CHECKER, Section<SECTION>>(),
            ),
        ),
    );
    let attempt = g::seq(
        g::send::<FACT_CHECKER, REVIEW_SERVICE, C::FactMaterial>(),
        g::seq(
            g::send::<SECTION, REVIEW_SERVICE, C::FactRequest>(),
            g::seq(
                g::send::<FACT_CHECKER, REVIEW_SERVICE, Section<SECTION>>(),
                g::route(
                    factual_retry,
                    g::seq(
                        g::send::<SECTION, FACT_CHECKER, C::FactPassed>(),
                        g::seq(
                            g::send::<SECTION, EDITOR, C::FactsForEditor>(),
                            g::seq(
                                g::send::<REVIEW_SERVICE, EDITOR, Section<SECTION>>(),
                                g::seq(
                                    g::send::<EDITOR, REVIEW_SERVICE, C::ReviewMaterial>(),
                                    g::seq(
                                        g::send::<SECTION, REVIEW_SERVICE, C::ReviewRequest>(),
                                        g::seq(
                                            g::send::<EDITOR, REVIEW_SERVICE, Section<SECTION>>(),
                                            g::route(editorial_retry, approved),
                                        ),
                                    ),
                                ),
                            ),
                        ),
                    ),
                ),
            ),
        ),
    );
    g::seq(
        attempt.roll(),
        g::seq(
            g::send::<REVIEW_SERVICE, FACT_CHECKER, C::FinalForFact>(),
            g::send::<FACT_CHECKER, ARCHIVE, Section<SECTION>>(),
        ),
    )
}

pub fn publication() -> impl Projectable {
    g::seq(
        g::send::<EDITOR, DOCUMENT, Combine>(),
        g::seq(
            g::par(
                g::par(
                    g::send::<ARCHIVE, DOCUMENT, Section<SECTION_A>>(),
                    g::par(
                        g::send::<ARCHIVE, DOCUMENT, Section<SECTION_B>>(),
                        g::send::<ARCHIVE, DOCUMENT, Section<SECTION_C>>(),
                    ),
                ),
                g::par(
                    g::seq(
                        g::send::<DOCUMENT, DATABASE, ReferencesForPublication>(),
                        g::send::<DATABASE, DOCUMENT, FrozenReferences>(),
                    ),
                    g::send::<REVIEW_SERVICE, DOCUMENT, FrozenReviews>(),
                ),
            ),
            g::par(
                g::send::<DOCUMENT, EDITOR, Published>(),
                g::send::<DOCUMENT, DATABASE, PublishedForDatabase>(),
            ),
        ),
    )
}

pub fn source() -> impl Projectable {
    g::seq(
        g::par(
            g::seq(
                preparation::<AUTHOR_A, SECTION_A, a::Research>(),
                quality::<AUTHOR_A, SECTION_A, a::Research>(),
            ),
            g::par(
                g::seq(
                    preparation::<AUTHOR_B, SECTION_B, b::Research>(),
                    quality::<AUTHOR_B, SECTION_B, b::Research>(),
                ),
                g::seq(
                    preparation::<AUTHOR_C, SECTION_C, c::Research>(),
                    quality::<AUTHOR_C, SECTION_C, c::Research>(),
                ),
            ),
        ),
        publication(),
    )
}
