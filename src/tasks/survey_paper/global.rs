//! The survey's four independently approved chapter rights meet at publication.
use super::{
    publication::Arguments,
    quality::{ChapterArguments, ChapterBytes},
};
use crate::shell::{
    global::{Exit, FileExit, Loan, file_metadata, private_io},
    imp::review_reply::ReviewReply,
};
use hibana::{Endpoint, g, runtime::program::Projectable};
use hibana_wasip1_runtime::{
    choreofs::{ChoreoFsObject, FdSpec, ObjectId},
    protocol::{FdBinding, FdReadRow, FdWriteRow},
};

pub const SECTION_A: u8 = 1;
pub const SECTION_B: u8 = 2;
pub const SECTION_C: u8 = 3;
pub const SECTION_D: u8 = 4;
pub const REVIEWER: u8 = 5;
pub const EDITOR: u8 = 6;
pub const EDITOR_FILE: u8 = 7;
pub const EDITOR_VM: u8 = 8;
pub const EDITOR_DATA: u8 = 9;
pub const EDITOR_IO: u8 = 10;
pub const AUTHOR_A: u8 = 11;
pub const AUTHOR_B: u8 = 12;
pub const AUTHOR_C: u8 = 13;
pub const AUTHOR_D: u8 = 14;
pub const AUTHOR_FILE_A: u8 = 15;
pub const AUTHOR_FILE_B: u8 = 16;
pub const AUTHOR_FILE_C: u8 = 17;
pub const AUTHOR_FILE_D: u8 = 18;
pub const REVIEW_FILE: u8 = 19;
pub const ANALYST: u8 = 21;
pub const ANALYST_FILE: u8 = 22;
pub const DATABASE: u8 = 23;
pub const FIGURES: u8 = 24;

pub const SESSION: u32 = 303;
pub const AUTHOR_VM_A: u8 = 25;
pub const AUTHOR_DATA_A: u8 = 26;
pub const AUTHOR_IO_A: u8 = 27;
pub const AUTHOR_VM_B: u8 = 28;
pub const AUTHOR_DATA_B: u8 = 29;
pub const AUTHOR_IO_B: u8 = 30;
pub const AUTHOR_VM_C: u8 = 31;
pub const AUTHOR_DATA_C: u8 = 32;
pub const AUTHOR_IO_C: u8 = 33;
pub const AUTHOR_VM_D: u8 = 34;
pub const AUTHOR_DATA_D: u8 = 35;
pub const AUTHOR_IO_D: u8 = 36;
pub const ANALYST_VM: u8 = 37;
pub const ANALYST_DATA: u8 = 38;
pub const ANALYST_IO: u8 = 39;
pub const REVIEW_VM: u8 = 40;
pub const REVIEW_DATA: u8 = 41;
pub const REVIEW_IO: u8 = 42;
pub fn wasi_task() -> impl Projectable {
    let loans = g::par(
        g::send::<AUTHOR_DATA_A, AUTHOR_A, Loan<AUTHOR_FILE_A>>(),
        g::par(
            g::send::<AUTHOR_DATA_B, AUTHOR_B, Loan<AUTHOR_FILE_B>>(),
            g::par(
                g::send::<AUTHOR_DATA_C, AUTHOR_C, Loan<AUTHOR_FILE_C>>(),
                g::par(
                    g::send::<AUTHOR_DATA_D, AUTHOR_D, Loan<AUTHOR_FILE_D>>(),
                    g::par(
                        g::send::<ANALYST_DATA, ANALYST, Loan<ANALYST_FILE>>(),
                        g::send::<REVIEW_DATA, REVIEWER, Loan<REVIEW_FILE>>(),
                    ),
                ),
            ),
        ),
    );
    let metadata = g::par(
        file_metadata::<AUTHOR_FILE_A, AUTHOR_DATA_A>().roll(),
        g::par(
            file_metadata::<AUTHOR_FILE_B, AUTHOR_DATA_B>().roll(),
            g::par(
                file_metadata::<AUTHOR_FILE_C, AUTHOR_DATA_C>().roll(),
                g::par(
                    file_metadata::<AUTHOR_FILE_D, AUTHOR_DATA_D>().roll(),
                    g::par(
                        file_metadata::<ANALYST_FILE, ANALYST_DATA>().roll(),
                        file_metadata::<REVIEW_FILE, REVIEW_DATA>().roll(),
                    ),
                ),
            ),
        ),
    );
    let file_exits = g::par(
        g::send::<AUTHOR_FILE_A, AUTHOR_DATA_A, FileExit>(),
        g::par(
            g::send::<AUTHOR_FILE_B, AUTHOR_DATA_B, FileExit>(),
            g::par(
                g::send::<AUTHOR_FILE_C, AUTHOR_DATA_C, FileExit>(),
                g::par(
                    g::send::<AUTHOR_FILE_D, AUTHOR_DATA_D, FileExit>(),
                    g::par(
                        g::send::<ANALYST_FILE, ANALYST_DATA, FileExit>(),
                        g::send::<REVIEW_FILE, REVIEW_DATA, FileExit>(),
                    ),
                ),
            ),
        ),
    );
    let io = g::par(
        g::seq(
            private_io::<AUTHOR_VM_A, AUTHOR_IO_A>(),
            g::send::<AUTHOR_VM_A, AUTHOR_IO_A, Exit>(),
        ),
        g::par(
            g::seq(
                private_io::<AUTHOR_VM_B, AUTHOR_IO_B>(),
                g::send::<AUTHOR_VM_B, AUTHOR_IO_B, Exit>(),
            ),
            g::par(
                g::seq(
                    private_io::<AUTHOR_VM_C, AUTHOR_IO_C>(),
                    g::send::<AUTHOR_VM_C, AUTHOR_IO_C, Exit>(),
                ),
                g::par(
                    g::seq(
                        private_io::<AUTHOR_VM_D, AUTHOR_IO_D>(),
                        g::send::<AUTHOR_VM_D, AUTHOR_IO_D, Exit>(),
                    ),
                    g::par(
                        g::seq(
                            private_io::<ANALYST_VM, ANALYST_IO>(),
                            g::send::<ANALYST_VM, ANALYST_IO, Exit>(),
                        ),
                        g::seq(
                            private_io::<REVIEW_VM, REVIEW_IO>(),
                            g::send::<REVIEW_VM, REVIEW_IO, Exit>(),
                        ),
                    ),
                ),
            ),
        ),
    );
    g::par(
        g::seq(
            loans,
            g::seq(g::par(wasi_publication(chapters()), metadata), file_exits),
        ),
        io,
    )
}

pub const PEER_REVIEW: u8 = 0;
pub const REVISION: u8 = 1;
pub const RESEARCH: u8 = 2;
pub const WRITE: u8 = 3;
pub const REFERENCES: u8 = 4;
pub const CREATE_FIGURE: u8 = 5;
pub const INSERT_FIGURE: u8 = 6;

const EDITOR_OPERATION: FdBinding = FdBinding {
    read: Some(FdReadRow::Base),
    write: Some(FdWriteRow::Object),
    readdir: None,
};
pub const EDITOR_OBJECTS: [ChoreoFsObject<'static>; 3] = [
    ChoreoFsObject::readable(
        b"service/accepted_sections",
        ObjectId(4),
        FdSpec::new(4, 2),
        b"",
        FdBinding::read(FdReadRow::Base),
    ),
    ChoreoFsObject::read_write(
        b"service/make_editorial_decision",
        ObjectId(5),
        FdSpec::new(5, 66),
        b"",
        EDITOR_OPERATION,
    ),
    ChoreoFsObject::read_write(
        b"service/combine_sections",
        ObjectId(6),
        FdSpec::new(6, 66),
        b"",
        EDITOR_OPERATION,
    ),
];

pub type Accepted<'r, const SECTION: u8> = g::Msg<SECTION, Endpoint<'r, SECTION>>;
pub type Reviewed<'a, const SECTION: u8> = g::Msg<SECTION, ReviewReply<'a, true>>;
pub type EditorialInput<'a> = g::Msg<10, &'a [u8]>;
pub type Editorial = g::Msg<11, Arguments<false>>;
pub type EditorialResult<'a> = g::Msg<12, &'a [u8]>;
pub type Combine = g::Msg<13, Arguments<true>>;
pub type Combined<'a> = g::Msg<14, &'a [u8]>;
pub type Draft<'a, const SECTION: u8> = g::Msg<20, ChapterBytes<'a, SECTION>>;
pub type ReviewInput<'a, const SECTION: u8> = g::Msg<21, ChapterBytes<'a, SECTION>>;
pub type Review<const SECTION: u8> = g::Msg<22, ChapterArguments<SECTION, PEER_REVIEW>>;
pub type ReviewResult<'a, const SECTION: u8> = g::Msg<23, ChapterBytes<'a, SECTION>>;
pub type PeerAccepted<'a> = g::Msg<24, ReviewReply<'a, true>>;
pub type PeerRejected<'a> = g::Msg<25, ReviewReply<'a, false>>;
pub type AuthorNotice<'a, const SECTION: u8> = g::Msg<26, ChapterBytes<'a, SECTION>>;
pub type Revise<const SECTION: u8> = g::Msg<27, ChapterArguments<SECTION, REVISION>>;
pub type Revised<'a, const SECTION: u8> = g::Msg<28, ChapterBytes<'a, SECTION>>;
pub type Research<const SECTION: u8> = g::Msg<30, ChapterArguments<SECTION, RESEARCH>>;
pub type Researched<'a, const SECTION: u8> = g::Msg<31, ChapterBytes<'a, SECTION>>;
pub type Write<const SECTION: u8> = g::Msg<32, ChapterArguments<SECTION, WRITE>>;
pub type Written<'a, const SECTION: u8> = g::Msg<33, ChapterBytes<'a, SECTION>>;
pub type References<const SECTION: u8> = g::Msg<SECTION, ChapterArguments<SECTION, REFERENCES>>;
pub type ReferenceSource<'a, const SECTION: u8> = g::Msg<34, ChapterBytes<'a, SECTION>>;
pub type Referenced<'a, const SECTION: u8> = g::Msg<35, ChapterBytes<'a, SECTION>>;
pub type ReferencesForChapter<'a, const SECTION: u8> = g::Msg<36, ChapterBytes<'a, SECTION>>;
pub type CreateFigure<const SECTION: u8> =
    g::Msg<SECTION, ChapterArguments<SECTION, CREATE_FIGURE>>;
pub type FigureSource<'a, const SECTION: u8> = g::Msg<37, ChapterBytes<'a, SECTION>>;
pub type FigureCreated<'a, const SECTION: u8> = g::Msg<38, ChapterBytes<'a, SECTION>>;
pub type FigureForChapter<'a, const SECTION: u8> = g::Msg<39, ChapterBytes<'a, SECTION>>;
pub type Prepared<'a, const SECTION: u8> = g::Msg<40, ChapterBytes<'a, SECTION>>;
pub type InsertFigure<const SECTION: u8> = g::Msg<41, ChapterArguments<SECTION, INSERT_FIGURE>>;
pub type Inserted<'a, const SECTION: u8> = g::Msg<42, ChapterBytes<'a, SECTION>>;
pub type ResearchForAnalyst<'a, const SECTION: u8> = g::Msg<43, ChapterBytes<'a, SECTION>>;
pub type InsertionInput<'a, const SECTION: u8> = g::Msg<44, ChapterBytes<'a, SECTION>>;

pub fn approved_chapter<const SECTION: u8>() -> impl Projectable {
    g::seq(
        g::send::<SECTION, EDITOR, Reviewed<SECTION>>(),
        g::send::<REVIEWER, EDITOR, Accepted<SECTION>>(),
    )
}

fn editorial() -> impl Projectable {
    g::seq(
        g::send::<EDITOR, EDITOR_FILE, EditorialInput>(),
        g::seq(
            g::send::<EDITOR_FILE, EDITOR, Editorial>(),
            g::seq(
                g::send::<EDITOR, EDITOR_FILE, EditorialResult>(),
                g::seq(
                    g::send::<EDITOR_FILE, EDITOR, Combine>(),
                    g::send::<EDITOR, EDITOR_FILE, Combined>(),
                ),
            ),
        ),
    )
}

pub fn wasi_publication(chapters: impl Projectable) -> impl Projectable {
    g::par(
        g::seq(
            g::send::<EDITOR_DATA, EDITOR, Loan<EDITOR_FILE>>(),
            g::seq(
                g::par(
                    g::seq(chapters, editorial()),
                    file_metadata::<EDITOR_FILE, EDITOR_DATA>().roll(),
                ),
                g::send::<EDITOR_FILE, EDITOR_DATA, FileExit>(),
            ),
        ),
        g::seq(
            private_io::<EDITOR_VM, EDITOR_IO>(),
            g::send::<EDITOR_VM, EDITOR_IO, Exit>(),
        ),
    )
}

/// The reviewing section's authority travels back to its own author on rejection.
pub fn peer_review<const SECTION: u8, const AUTHOR: u8, const FILE: u8>() -> impl Projectable {
    let review = || {
        g::seq(
            g::send::<SECTION, REVIEW_FILE, ReviewInput<SECTION>>(),
            g::send::<REVIEW_FILE, SECTION, Review<SECTION>>(),
        )
    };
    let publish_review = || {
        g::par(
            g::send::<SECTION, REVIEW_FILE, ReviewResult<SECTION>>(),
            g::send::<AUTHOR, FILE, AuthorNotice<SECTION>>(),
        )
    };
    let approved = || {
        g::seq(
            g::send::<SECTION, AUTHOR, PeerAccepted>(),
            g::seq(publish_review(), approved_chapter::<SECTION>()),
        )
    };
    let repair = g::seq(
        g::send::<SECTION, AUTHOR, PeerRejected>(),
        g::seq(
            publish_review(),
            g::seq(
                g::send::<REVIEWER, AUTHOR, Accepted<SECTION>>(),
                g::seq(
                    g::send::<FILE, SECTION, Revise<SECTION>>(),
                    g::seq(
                        g::send::<SECTION, FILE, Revised<SECTION>>(),
                        g::seq(
                            g::send::<SECTION, REVIEWER, Draft<SECTION>>(),
                            g::send::<AUTHOR, REVIEWER, Accepted<SECTION>>(),
                        ),
                    ),
                ),
            ),
        ),
    );
    g::seq(
        review(),
        g::route(
            approved(),
            g::seq(g::seq(repair, review()).roll(), approved()),
        ),
    )
}

fn chapter<const SECTION: u8, const AUTHOR: u8, const FILE: u8>() -> impl Projectable {
    let write = g::seq(
        g::send::<FILE, SECTION, Write<SECTION>>(),
        g::send::<SECTION, FILE, Written<SECTION>>(),
    );
    let references = g::seq(
        g::send::<FILE, DATABASE, References<SECTION>>(),
        g::seq(
            g::send::<SECTION, DATABASE, ReferenceSource<SECTION>>(),
            g::par(
                g::send::<DATABASE, FILE, Referenced<SECTION>>(),
                g::send::<DATABASE, SECTION, ReferencesForChapter<SECTION>>(),
            ),
        ),
    );
    let figure = g::seq(
        g::send::<ANALYST_FILE, FIGURES, CreateFigure<SECTION>>(),
        g::seq(
            g::send::<SECTION, FIGURES, FigureSource<SECTION>>(),
            g::par(
                g::send::<FIGURES, ANALYST_FILE, FigureCreated<SECTION>>(),
                g::send::<FIGURES, SECTION, FigureForChapter<SECTION>>(),
            ),
        ),
    );
    let insert = g::seq(
        g::send::<SECTION, ANALYST, Prepared<SECTION>>(),
        g::seq(
            g::send::<AUTHOR, ANALYST, Accepted<SECTION>>(),
            g::seq(
                g::send::<SECTION, ANALYST_FILE, InsertionInput<SECTION>>(),
                g::seq(
                    g::send::<ANALYST_FILE, SECTION, InsertFigure<SECTION>>(),
                    g::seq(
                        g::send::<SECTION, ANALYST_FILE, Inserted<SECTION>>(),
                        g::seq(
                            g::send::<SECTION, REVIEWER, Draft<SECTION>>(),
                            g::send::<ANALYST, REVIEWER, Accepted<SECTION>>(),
                        ),
                    ),
                ),
            ),
        ),
    );
    g::seq(
        g::send::<FILE, SECTION, Research<SECTION>>(),
        g::seq(
            g::par(
                g::send::<SECTION, FILE, Researched<SECTION>>(),
                g::send::<SECTION, ANALYST_FILE, ResearchForAnalyst<SECTION>>(),
            ),
            g::seq(
                g::par(write, g::par(references, figure)),
                g::seq(insert, peer_review::<SECTION, AUTHOR, FILE>()),
            ),
        ),
    )
}

pub fn chapters() -> impl Projectable {
    g::par(
        chapter::<SECTION_A, AUTHOR_A, AUTHOR_FILE_A>(),
        g::par(
            chapter::<SECTION_B, AUTHOR_B, AUTHOR_FILE_B>(),
            g::par(
                chapter::<SECTION_C, AUTHOR_C, AUTHOR_FILE_C>(),
                chapter::<SECTION_D, AUTHOR_D, AUTHOR_FILE_D>(),
            ),
        ),
    )
}
