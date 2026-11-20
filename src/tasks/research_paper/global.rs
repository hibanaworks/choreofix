//! Three affine chapter continuations and two unique physical I/O owners.
use super::imp::{Arguments, Reply, Verdict};
use hibana::{
    Endpoint, g,
    g::Message,
    runtime::program::{Projectable, RoleProgram, project},
};
use hibana_wasip1_runtime::{
    choreofs::{ChoreoFsObject, FdSpec, ObjectId},
    protocol,
};

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
pub const AUTHOR_A_FILE: u8 = 15;
pub const AUTHOR_B_FILE: u8 = 16;
pub const AUTHOR_C_FILE: u8 = 17;
pub const FACT_FILE: u8 = 18;
pub const EDITOR_FILE: u8 = 19;
pub const INIT: u8 = 20;
pub const AUTHOR_A_DATA: u8 = 21;
pub const AUTHOR_B_DATA: u8 = 22;
pub const AUTHOR_C_DATA: u8 = 23;
pub const FACT_DATA: u8 = 24;
pub const EDITOR_DATA: u8 = 25;
pub const AUTHOR_A_VM: u8 = 26;
pub const AUTHOR_B_VM: u8 = 27;
pub const AUTHOR_C_VM: u8 = 28;
pub const FACT_VM: u8 = 29;
pub const EDITOR_VM: u8 = 30;
pub const AUTHOR_A_IO: u8 = 31;
pub const AUTHOR_B_IO: u8 = 32;
pub const AUTHOR_C_IO: u8 = 33;
pub const FACT_IO: u8 = 34;
pub const EDITOR_IO: u8 = 35;
pub const SESSION: u32 = 324;
pub type Section<'r, const ROLE: u8> = g::Msg<1, Endpoint<'r, ROLE>>;
pub use crate::shell::global::{Exit, FileExit, Loan, file_metadata, private_io};

// These are static wire contracts for the three actual chapter participants.
// They carry no runtime state. The same chapter choreography is instantiated
// three times; distinct labels let a physical owner identify requests directly.
pub trait ChapterMessages<const AUTHOR: u8> {
    type Research: g::Message<Payload = Arguments<AUTHOR, 1>>;
    type Researched: g::Message<Payload = Reply<AUTHOR, 1>>;
    type Supplement: g::Message<Payload = Arguments<AUTHOR, 7>>;
    type Supplemented: g::Message<Payload = Reply<AUTHOR, 7>>;
    type ResearchDone: g::Message<Payload = i32>;
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
    type FactRejectionForDocument: g::Message<Payload = Verdict<AUTHOR, 5, false>>;
    type Revise: g::Message<Payload = Arguments<AUTHOR, 3>>;
    type Revised: g::Message<Payload = Reply<AUTHOR, 3>>;
    type RevisedForFact: g::Message<Payload = Reply<AUTHOR, 3>>;
    type ReviewMaterial: g::Message<Payload = super::imp::Material>;
    type ReviewRequest: g::Message<Payload = Arguments<AUTHOR, 6>>;
    type ReviewPassed: g::Message<Payload = Verdict<AUTHOR, 6, true>>;
    type ReviewRejected: g::Message<Payload = Verdict<AUTHOR, 6, false>>;
    type ReviewRejectionForAuthor: g::Message<Payload = Verdict<AUTHOR, 6, false>>;
    type ReviewRejectionForDocument: g::Message<Payload = Verdict<AUTHOR, 6, false>>;
    type FinalForAuthor: g::Message<Payload = Verdict<AUTHOR, 6, true>>;
    type FinalForDocument: g::Message<Payload = Verdict<AUTHOR, 6, true>>;
    type FinalForFact: g::Message<Payload = Verdict<AUTHOR, 6, true>>;
    type FailedFactsForEditor: g::Message<Payload = Verdict<AUTHOR, 5, false>>;
    type AuthorNotice: g::Message<Payload = super::imp::Material>;
    type FactInput: g::Message<Payload = super::imp::Material>;
    type FactFinished: g::Message<Payload = Verdict<AUTHOR, 6, true>>;
    type FactResult: g::Message<Payload = Reply<AUTHOR, 5>>;
    type ReviewInput: g::Message<Payload = super::imp::Material>;
    type ReviewResult: g::Message<Payload = Reply<AUTHOR, 6>>;
}
macro_rules! chapter_messages {
    ($module:ident,$author:expr,$base:expr) => {
        pub mod $module {
            use super::*;
            pub type Research = g::Msg<{ $base + 2 }, Arguments<$author, 1>>;
            pub type Researched = g::Msg<{ $base + 3 }, Reply<$author, 1>>;
            pub type Supplement = g::Msg<{ $base + 43 }, Arguments<$author, 7>>;
            pub type Supplemented = g::Msg<{ $base + 44 }, Reply<$author, 7>>;
            pub type ResearchDone = g::Msg<{ $base + 45 }, i32>;
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
            pub type FactRejectionForDocument = g::Msg<{ $base + 40 }, Verdict<$author, 5, false>>;
            pub type Revise = g::Msg<{ $base + 21 }, Arguments<$author, 3>>;
            pub type Revised = g::Msg<{ $base + 23 }, Reply<$author, 3>>;
            pub type RevisedForFact = g::Msg<{ $base + 24 }, Reply<$author, 3>>;
            pub type ReviewMaterial = g::Msg<{ $base + 25 }, super::super::imp::Material>;
            pub type ReviewRequest = g::Msg<{ $base + 26 }, Arguments<$author, 6>>;
            pub type ReviewPassed = g::Msg<{ $base + 27 }, Verdict<$author, 6, true>>;
            pub type ReviewRejected = g::Msg<{ $base + 28 }, Verdict<$author, 6, false>>;
            pub type ReviewRejectionForAuthor = g::Msg<{ $base + 29 }, Verdict<$author, 6, false>>;
            pub type ReviewRejectionForDocument =
                g::Msg<{ $base + 41 }, Verdict<$author, 6, false>>;
            pub type FinalForAuthor = g::Msg<{ $base + 30 }, Verdict<$author, 6, true>>;
            pub type FinalForDocument = g::Msg<{ $base + 42 }, Verdict<$author, 6, true>>;
            pub type FinalForFact = g::Msg<{ $base + 31 }, Verdict<$author, 6, true>>;
            pub type FailedFactsForEditor = g::Msg<{ $base + 33 }, Verdict<$author, 5, false>>;
            pub type AuthorNotice = g::Msg<{ $base + 34 }, super::super::imp::Material>;
            pub type FactInput = g::Msg<{ $base + 35 }, super::super::imp::Material>;
            pub type FactFinished = g::Msg<{ $base + 39 }, Verdict<$author, 6, true>>;
            pub type FactResult = g::Msg<{ $base + 36 }, Reply<$author, 5>>;
            pub type ReviewInput = g::Msg<{ $base + 37 }, super::super::imp::Material>;
            pub type ReviewResult = g::Msg<{ $base + 38 }, Reply<$author, 6>>;
            pub const AUDIT_LABELS: &[(&str, u8)] = &[
                ("Research", Research::LOGICAL_LABEL),
                ("Researched", Researched::LOGICAL_LABEL),
                ("Supplement", Supplement::LOGICAL_LABEL),
                ("Supplemented", Supplemented::LOGICAL_LABEL),
                ("ResearchDone", ResearchDone::LOGICAL_LABEL),
                ("Write", Write::LOGICAL_LABEL),
                ("Written", Written::LOGICAL_LABEL),
                ("References", References::LOGICAL_LABEL),
                ("Referenced", Referenced::LOGICAL_LABEL),
                ("Revise", Revise::LOGICAL_LABEL),
                ("Revised", Revised::LOGICAL_LABEL),
                ("FactRequest", FactRequest::LOGICAL_LABEL),
                ("FactResult", FactResult::LOGICAL_LABEL),
                ("FactInput", FactInput::LOGICAL_LABEL),
                ("FactFinished", FactFinished::LOGICAL_LABEL),
                ("ReviewRequest", ReviewRequest::LOGICAL_LABEL),
                ("ReviewResult", ReviewResult::LOGICAL_LABEL),
                ("ReviewInput", ReviewInput::LOGICAL_LABEL),
                ("AuthorNotice", AuthorNotice::LOGICAL_LABEL),
            ];
        }
        impl ChapterMessages<$author> for $module::Research {
            type Research = $module::Research;
            type Researched = $module::Researched;
            type Supplement = $module::Supplement;
            type Supplemented = $module::Supplemented;
            type ResearchDone = $module::ResearchDone;
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
            type FactRejectionForDocument = $module::FactRejectionForDocument;
            type Revise = $module::Revise;
            type Revised = $module::Revised;
            type RevisedForFact = $module::RevisedForFact;
            type ReviewMaterial = $module::ReviewMaterial;
            type ReviewRequest = $module::ReviewRequest;
            type ReviewPassed = $module::ReviewPassed;
            type ReviewRejected = $module::ReviewRejected;
            type ReviewRejectionForAuthor = $module::ReviewRejectionForAuthor;
            type ReviewRejectionForDocument = $module::ReviewRejectionForDocument;
            type FinalForAuthor = $module::FinalForAuthor;
            type FinalForDocument = $module::FinalForDocument;
            type FinalForFact = $module::FinalForFact;
            type FailedFactsForEditor = $module::FailedFactsForEditor;
            type AuthorNotice = $module::AuthorNotice;
            type FactInput = $module::FactInput;
            type FactFinished = $module::FactFinished;
            type FactResult = $module::FactResult;
            type ReviewInput = $module::ReviewInput;
            type ReviewResult = $module::ReviewResult;
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

fn preparation<const AUTHOR: u8, const SECTION: u8, const FILE: u8, C: ChapterMessages<AUTHOR>>()
-> impl Projectable {
    let research = g::seq(
        g::send::<FILE, RESEARCH_SERVICE, C::Research>(),
        g::par(
            g::send::<RESEARCH_SERVICE, FILE, C::Researched>(),
            g::par(
                g::send::<RESEARCH_SERVICE, DOCUMENT, C::ResearchForDocument>(),
                g::send::<RESEARCH_SERVICE, DATABASE, C::ResearchForDatabase>(),
            ),
        ),
    );
    let document = g::seq(
        g::send::<FILE, DOCUMENT, C::Write>(),
        g::seq(
            g::send::<DOCUMENT, FILE, C::Written>(),
            g::send::<DOCUMENT, FACT_CHECKER, C::MaterialForFact>(),
        ),
    );
    let database = g::seq(
        g::send::<FILE, DATABASE, C::References>(),
        g::seq(
            g::send::<DATABASE, FILE, C::Referenced>(),
            g::send::<DATABASE, FACT_CHECKER, C::ReferencesForFact>(),
        ),
    );
    g::seq(
        research,
        g::seq(
            g::par(document, database),
            g::send::<AUTHOR, FACT_CHECKER, Section<SECTION>>(),
        ),
    )
}

fn revision<const AUTHOR: u8, const SECTION: u8, const FILE: u8, C: ChapterMessages<AUTHOR>>()
-> impl Projectable {
    g::seq(
        g::send::<FILE, DOCUMENT, C::Revise>(),
        g::seq(
            g::send::<DOCUMENT, FILE, C::Revised>(),
            g::seq(
                g::send::<DOCUMENT, FACT_CHECKER, C::RevisedForFact>(),
                g::send::<AUTHOR, FACT_CHECKER, Section<SECTION>>(),
            ),
        ),
    )
}

fn quality<const AUTHOR: u8, const SECTION: u8, const FILE: u8, C: ChapterMessages<AUTHOR>>()
-> impl Projectable {
    let factual_retry = g::seq(
        g::send::<SECTION, FACT_CHECKER, C::FactRejected>(),
        g::par(
            g::send::<REVIEW_SERVICE, FACT_FILE, C::FactResult>(),
            g::seq(
                g::send::<SECTION, EDITOR, C::FailedFactsForEditor>(),
                g::seq(
                    g::seq(
                        g::send::<SECTION, AUTHOR, C::FactRejectionForAuthor>(),
                        g::send::<SECTION, DOCUMENT, C::FactRejectionForDocument>(),
                    ),
                    g::seq(
                        g::par(
                            g::send::<REVIEW_SERVICE, AUTHOR, Section<SECTION>>(),
                            g::send::<DOCUMENT, FILE, C::AuthorNotice>(),
                        ),
                        revision::<AUTHOR, SECTION, FILE, C>(),
                    ),
                ),
            ),
        ),
    );
    let editorial_retry = g::seq(
        g::send::<SECTION, EDITOR, C::ReviewRejected>(),
        g::par(
            g::send::<REVIEW_SERVICE, EDITOR_FILE, C::ReviewResult>(),
            g::seq(
                g::seq(
                    g::send::<SECTION, AUTHOR, C::ReviewRejectionForAuthor>(),
                    g::send::<SECTION, DOCUMENT, C::ReviewRejectionForDocument>(),
                ),
                g::seq(
                    g::par(
                        g::send::<REVIEW_SERVICE, AUTHOR, Section<SECTION>>(),
                        g::send::<DOCUMENT, FILE, C::AuthorNotice>(),
                    ),
                    revision::<AUTHOR, SECTION, FILE, C>(),
                ),
            ),
        ),
    );
    let approved = g::seq(
        g::send::<SECTION, EDITOR, C::ReviewPassed>(),
        g::par(
            g::send::<REVIEW_SERVICE, EDITOR_FILE, C::ReviewResult>(),
            g::seq(
                g::seq(
                    g::send::<SECTION, AUTHOR, C::FinalForAuthor>(),
                    g::send::<SECTION, DOCUMENT, C::FinalForDocument>(),
                ),
                g::seq(
                    g::send::<DOCUMENT, FILE, C::AuthorNotice>(),
                    g::send::<REVIEW_SERVICE, FACT_CHECKER, Section<SECTION>>(),
                ),
            ),
        ),
    );
    let editorial = g::seq(
        g::send::<REVIEW_SERVICE, EDITOR, Section<SECTION>>(),
        g::seq(
            g::send::<EDITOR, EDITOR_FILE, C::ReviewInput>(),
            g::seq(
                g::send::<EDITOR, REVIEW_SERVICE, C::ReviewMaterial>(),
                g::seq(
                    g::send::<EDITOR, REVIEW_SERVICE, Section<SECTION>>(),
                    g::send::<EDITOR_FILE, REVIEW_SERVICE, C::ReviewRequest>(),
                ),
            ),
        ),
    );
    let factual = g::seq(
        g::send::<FACT_CHECKER, FACT_FILE, C::FactInput>(),
        g::seq(
            g::send::<FACT_CHECKER, REVIEW_SERVICE, C::FactMaterial>(),
            g::seq(
                g::send::<FACT_CHECKER, REVIEW_SERVICE, Section<SECTION>>(),
                g::send::<FACT_FILE, REVIEW_SERVICE, C::FactRequest>(),
            ),
        ),
    );
    let factual_approval = g::seq(
        g::send::<SECTION, FACT_CHECKER, C::FactPassed>(),
        g::seq(
            g::par(
                g::send::<REVIEW_SERVICE, FACT_FILE, C::FactResult>(),
                g::send::<SECTION, EDITOR, C::FactsForEditor>(),
            ),
            g::seq(editorial, g::route(editorial_retry, approved)),
        ),
    );
    g::seq(
        g::seq(factual, g::route(factual_retry, factual_approval)).roll(),
        g::seq(
            g::send::<REVIEW_SERVICE, FACT_CHECKER, C::FinalForFact>(),
            g::par(
                g::send::<FACT_CHECKER, ARCHIVE, Section<SECTION>>(),
                g::send::<FACT_CHECKER, FACT_FILE, C::FactFinished>(),
            ),
        ),
    )
}

pub fn publication() -> impl Projectable {
    g::seq(
        g::send::<EDITOR_FILE, DOCUMENT, Combine>(),
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
                g::send::<DOCUMENT, EDITOR_FILE, Published>(),
                g::send::<DOCUMENT, DATABASE, PublishedForDatabase>(),
            ),
        ),
    )
}

fn chapters() -> impl Projectable {
    g::seq(
        g::par(
            g::seq(
                preparation::<AUTHOR_A, SECTION_A, AUTHOR_A_FILE, a::Research>(),
                quality::<AUTHOR_A, SECTION_A, AUTHOR_A_FILE, a::Research>(),
            ),
            g::par(
                g::seq(
                    preparation::<AUTHOR_B, SECTION_B, AUTHOR_B_FILE, b::Research>(),
                    quality::<AUTHOR_B, SECTION_B, AUTHOR_B_FILE, b::Research>(),
                ),
                g::seq(
                    preparation::<AUTHOR_C, SECTION_C, AUTHOR_C_FILE, c::Research>(),
                    quality::<AUTHOR_C, SECTION_C, AUTHOR_C_FILE, c::Research>(),
                ),
            ),
        ),
        publication(),
    )
}

fn open_files() -> impl Projectable {
    g::par(
        g::send::<AUTHOR_A_DATA, AUTHOR_A, Loan<AUTHOR_A_FILE>>(),
        g::par(
            g::send::<AUTHOR_B_DATA, AUTHOR_B, Loan<AUTHOR_B_FILE>>(),
            g::par(
                g::send::<AUTHOR_C_DATA, AUTHOR_C, Loan<AUTHOR_C_FILE>>(),
                g::par(
                    g::send::<FACT_DATA, FACT_CHECKER, Loan<FACT_FILE>>(),
                    g::send::<EDITOR_DATA, EDITOR, Loan<EDITOR_FILE>>(),
                ),
            ),
        ),
    )
}

#[cfg(test)]
pub fn source() -> impl Projectable {
    g::seq(open_files(), chapters())
}

fn supplemental_research<const AUTHOR: u8, const FILE: u8, C: ChapterMessages<AUTHOR>>()
-> impl Projectable {
    let done = g::send::<FILE, RESEARCH_SERVICE, C::ResearchDone>();
    let query = g::seq(
        g::send::<FILE, RESEARCH_SERVICE, C::Supplement>(),
        g::send::<RESEARCH_SERVICE, FILE, C::Supplemented>(),
    );
    g::route(done, g::seq(query.roll(), done))
}

fn wasi_source() -> impl Projectable {
    let guests = g::par(
        g::send::<INIT, AUTHOR_A, Loan<AUTHOR_A_VM>>(),
        g::par(
            g::send::<INIT, AUTHOR_B, Loan<AUTHOR_B_VM>>(),
            g::par(
                g::send::<INIT, AUTHOR_C, Loan<AUTHOR_C_VM>>(),
                g::par(
                    g::send::<INIT, FACT_CHECKER, Loan<FACT_VM>>(),
                    g::send::<INIT, EDITOR, Loan<EDITOR_VM>>(),
                ),
            ),
        ),
    );
    let supplemental = g::par(
        supplemental_research::<AUTHOR_A, AUTHOR_A_FILE, a::Research>(),
        g::par(
            supplemental_research::<AUTHOR_B, AUTHOR_B_FILE, b::Research>(),
            supplemental_research::<AUTHOR_C, AUTHOR_C_FILE, c::Research>(),
        ),
    );
    let metadata = g::par(
        file_metadata::<AUTHOR_A_FILE, AUTHOR_A_DATA>().roll(),
        g::par(
            file_metadata::<AUTHOR_B_FILE, AUTHOR_B_DATA>().roll(),
            g::par(
                file_metadata::<AUTHOR_C_FILE, AUTHOR_C_DATA>().roll(),
                g::par(
                    file_metadata::<FACT_FILE, FACT_DATA>().roll(),
                    file_metadata::<EDITOR_FILE, EDITOR_DATA>().roll(),
                ),
            ),
        ),
    );
    let exits = g::par(
        g::send::<AUTHOR_A_FILE, AUTHOR_A_DATA, FileExit>(),
        g::par(
            g::send::<AUTHOR_B_FILE, AUTHOR_B_DATA, FileExit>(),
            g::par(
                g::send::<AUTHOR_C_FILE, AUTHOR_C_DATA, FileExit>(),
                g::par(
                    g::send::<FACT_FILE, FACT_DATA, FileExit>(),
                    g::send::<EDITOR_FILE, EDITOR_DATA, FileExit>(),
                ),
            ),
        ),
    );
    let io = g::par(
        g::seq(
            private_io::<AUTHOR_A_VM, AUTHOR_A_IO>(),
            g::send::<AUTHOR_A_VM, AUTHOR_A_IO, Exit>(),
        ),
        g::par(
            g::seq(
                private_io::<AUTHOR_B_VM, AUTHOR_B_IO>(),
                g::send::<AUTHOR_B_VM, AUTHOR_B_IO, Exit>(),
            ),
            g::par(
                g::seq(
                    private_io::<AUTHOR_C_VM, AUTHOR_C_IO>(),
                    g::send::<AUTHOR_C_VM, AUTHOR_C_IO, Exit>(),
                ),
                g::par(
                    g::seq(
                        private_io::<FACT_VM, FACT_IO>(),
                        g::send::<FACT_VM, FACT_IO, Exit>(),
                    ),
                    g::seq(
                        private_io::<EDITOR_VM, EDITOR_IO>(),
                        g::send::<EDITOR_VM, EDITOR_IO, Exit>(),
                    ),
                ),
            ),
        ),
    );
    g::seq(
        g::par(open_files(), guests),
        g::par(
            g::seq(g::par(g::par(chapters(), supplemental), metadata), exits),
            io,
        ),
    )
}

const fn request_file(path: &'static [u8], fd: u8) -> ChoreoFsObject<'static> {
    ChoreoFsObject::read_write(
        path,
        ObjectId(fd as u32),
        FdSpec::new(fd, 66),
        b"",
        protocol::FdBinding {
            read: Some(protocol::FdReadRow::Base),
            write: Some(protocol::FdWriteRow::Object),
            readdir: None,
        },
    )
}

const fn incoming(path: &'static [u8], fd: u8) -> ChoreoFsObject<'static> {
    ChoreoFsObject::readable(
        path,
        ObjectId(fd as u32),
        FdSpec::new(fd, 2),
        b"",
        protocol::FdBinding::read(protocol::FdReadRow::Base),
    )
}

pub const AUTHOR_FILES: [ChoreoFsObject<'static>; 6] = [
    request_file(b"service/research_topic", 4),
    request_file(b"service/write_section", 5),
    request_file(b"service/update_references", 6),
    request_file(b"service/revise_section", 7),
    incoming(b"service/review", 8),
    request_file(b"service/supplemental_research", 9),
];

pub const FACT_FILES: [ChoreoFsObject<'static>; 9] = [
    incoming(b"service/section/section_a", 4),
    request_file(b"service/fact_check/section_a", 5),
    incoming(b"service/approved/section_a", 6),
    incoming(b"service/section/section_b", 7),
    request_file(b"service/fact_check/section_b", 8),
    incoming(b"service/approved/section_b", 9),
    incoming(b"service/section/section_c", 10),
    request_file(b"service/fact_check/section_c", 11),
    incoming(b"service/approved/section_c", 12),
];

pub const EDITOR_FILES: [ChoreoFsObject<'static>; 7] = [
    incoming(b"service/section/section_a", 4),
    request_file(b"service/review_sections/section_a", 5),
    incoming(b"service/section/section_b", 6),
    request_file(b"service/review_sections/section_b", 7),
    incoming(b"service/section/section_c", 8),
    request_file(b"service/review_sections/section_c", 9),
    request_file(b"service/combine_sections", 10),
];

pub struct Programs {
    pub author_a: RoleProgram<AUTHOR_A>,
    pub author_b: RoleProgram<AUTHOR_B>,
    pub author_c: RoleProgram<AUTHOR_C>,
    pub fact: RoleProgram<FACT_CHECKER>,
    pub editor: RoleProgram<EDITOR>,
    pub research_service: RoleProgram<RESEARCH_SERVICE>,
    pub document: RoleProgram<DOCUMENT>,
    pub database: RoleProgram<DATABASE>,
    pub section_a: RoleProgram<SECTION_A>,
    pub section_b: RoleProgram<SECTION_B>,
    pub section_c: RoleProgram<SECTION_C>,
    pub review_service: RoleProgram<REVIEW_SERVICE>,
    pub archive: RoleProgram<ARCHIVE>,
    pub init: RoleProgram<INIT>,
    pub author_a_file: RoleProgram<AUTHOR_A_FILE>,
    pub author_a_data: RoleProgram<AUTHOR_A_DATA>,
    pub author_a_vm: RoleProgram<AUTHOR_A_VM>,
    pub author_a_io: RoleProgram<AUTHOR_A_IO>,
    pub author_b_file: RoleProgram<AUTHOR_B_FILE>,
    pub author_b_data: RoleProgram<AUTHOR_B_DATA>,
    pub author_b_vm: RoleProgram<AUTHOR_B_VM>,
    pub author_b_io: RoleProgram<AUTHOR_B_IO>,
    pub author_c_file: RoleProgram<AUTHOR_C_FILE>,
    pub author_c_data: RoleProgram<AUTHOR_C_DATA>,
    pub author_c_vm: RoleProgram<AUTHOR_C_VM>,
    pub author_c_io: RoleProgram<AUTHOR_C_IO>,
    pub fact_file: RoleProgram<FACT_FILE>,
    pub fact_data: RoleProgram<FACT_DATA>,
    pub fact_vm: RoleProgram<FACT_VM>,
    pub fact_io: RoleProgram<FACT_IO>,
    pub editor_file: RoleProgram<EDITOR_FILE>,
    pub editor_data: RoleProgram<EDITOR_DATA>,
    pub editor_vm: RoleProgram<EDITOR_VM>,
    pub editor_io: RoleProgram<EDITOR_IO>,
}

pub fn programs() -> Programs {
    let source = wasi_source();
    Programs {
        author_a: project(&source),
        author_b: project(&source),
        author_c: project(&source),
        fact: project(&source),
        editor: project(&source),
        research_service: project(&source),
        document: project(&source),
        database: project(&source),
        section_a: project(&source),
        section_b: project(&source),
        section_c: project(&source),
        review_service: project(&source),
        archive: project(&source),
        init: project(&source),
        author_a_file: project(&source),
        author_a_data: project(&source),
        author_a_vm: project(&source),
        author_a_io: project(&source),
        author_b_file: project(&source),
        author_b_data: project(&source),
        author_b_vm: project(&source),
        author_b_io: project(&source),
        author_c_file: project(&source),
        author_c_data: project(&source),
        author_c_vm: project(&source),
        author_c_io: project(&source),
        fact_file: project(&source),
        fact_data: project(&source),
        fact_vm: project(&source),
        fact_io: project(&source),
        editor_file: project(&source),
        editor_data: project(&source),
        editor_vm: project(&source),
        editor_io: project(&source),
    }
}

impl Programs {
    pub fn observation(&self) -> serde_json::Value {
        use crate::imp::projection::rows;
        let roles = [
            ("RESEARCHER_A", AUTHOR_A),
            ("RESEARCHER_B", AUTHOR_B),
            ("RESEARCHER_C", AUTHOR_C),
            ("FACT_CHECKER", FACT_CHECKER),
            ("EDITOR", EDITOR),
            ("RESEARCH_SERVICE", RESEARCH_SERVICE),
            ("DOCUMENT", DOCUMENT),
            ("DATABASE", DATABASE),
            ("SECTION_A", SECTION_A),
            ("SECTION_B", SECTION_B),
            ("SECTION_C", SECTION_C),
            ("REVIEW_SERVICE", REVIEW_SERVICE),
            ("ARCHIVE", ARCHIVE),
            ("INIT", INIT),
            ("AUTHOR_A_FILE", AUTHOR_A_FILE),
            ("AUTHOR_A_DATA", AUTHOR_A_DATA),
            ("AUTHOR_A_VM", AUTHOR_A_VM),
            ("AUTHOR_A_IO", AUTHOR_A_IO),
            ("AUTHOR_B_FILE", AUTHOR_B_FILE),
            ("AUTHOR_B_DATA", AUTHOR_B_DATA),
            ("AUTHOR_B_VM", AUTHOR_B_VM),
            ("AUTHOR_B_IO", AUTHOR_B_IO),
            ("AUTHOR_C_FILE", AUTHOR_C_FILE),
            ("AUTHOR_C_DATA", AUTHOR_C_DATA),
            ("AUTHOR_C_VM", AUTHOR_C_VM),
            ("AUTHOR_C_IO", AUTHOR_C_IO),
            ("FACT_FILE", FACT_FILE),
            ("FACT_DATA", FACT_DATA),
            ("FACT_VM", FACT_VM),
            ("FACT_IO", FACT_IO),
            ("EDITOR_FILE", EDITOR_FILE),
            ("EDITOR_DATA", EDITOR_DATA),
            ("EDITOR_VM", EDITOR_VM),
            ("EDITOR_IO", EDITOR_IO),
        ]
        .into_iter()
        .collect::<std::collections::BTreeMap<_, _>>();
        let programs = [
            rows(&self.author_a),
            rows(&self.author_b),
            rows(&self.author_c),
            rows(&self.fact),
            rows(&self.editor),
            rows(&self.research_service),
            rows(&self.document),
            rows(&self.database),
            rows(&self.section_a),
            rows(&self.section_b),
            rows(&self.section_c),
            rows(&self.review_service),
            rows(&self.archive),
            rows(&self.init),
            rows(&self.author_a_file),
            rows(&self.author_a_data),
            rows(&self.author_a_vm),
            rows(&self.author_a_io),
            rows(&self.author_b_file),
            rows(&self.author_b_data),
            rows(&self.author_b_vm),
            rows(&self.author_b_io),
            rows(&self.author_c_file),
            rows(&self.author_c_data),
            rows(&self.author_c_vm),
            rows(&self.author_c_io),
            rows(&self.fact_file),
            rows(&self.fact_data),
            rows(&self.fact_vm),
            rows(&self.fact_io),
            rows(&self.editor_file),
            rows(&self.editor_data),
            rows(&self.editor_vm),
            rows(&self.editor_io),
        ];
        let chapter_labels = [a::AUDIT_LABELS, b::AUDIT_LABELS, c::AUDIT_LABELS].map(|labels| {
            labels
                .iter()
                .copied()
                .collect::<std::collections::BTreeMap<_, _>>()
        });
        serde_json::json!({
            "type":"compiled_projection", "format_version":1, "task":"3M", "session":SESSION,
            "roles":roles, "programs":programs.as_slice(),
            "chapter_labels":chapter_labels,
            "labels":{
                "Combine":Combine::LOGICAL_LABEL,"Published":Published::LOGICAL_LABEL,
                "FrozenReviews":FrozenReviews::LOGICAL_LABEL,
                "FrozenReferences":FrozenReferences::LOGICAL_LABEL,
                "Section":Section::<SECTION_A>::LOGICAL_LABEL,
            },
        })
    }
}
