use choreofix::{
    imp::json::{decode, encode},
    imp::projection::rows,
    shell::{
        GUEST_MEMORY_BYTES, ShellError, ShellResult,
        global::{Exit, FileExit, Loan, file_metadata, private_io},
        imp::{
            bus::Bus,
            private_fs::PrivateFs,
            review_reply::ReviewReply,
            service::{PairedEffects, ReceivedEffects, SimulatorOperation},
        },
        localside,
    },
};
use hibana::{
    g::{self, Message},
    runtime::{
        SessionKitStorage,
        ids::SessionId,
        program::{Projectable, project},
    },
};
use hibana_wasip1_runtime::{
    ChoreoFsFile, ChoreoFsFiles, GuestMemory, HibanaWasiGuestStorage,
    choreofs::{ChoreoFsObject, FdSpec, ObjectId},
    protocol::{FdBinding, FdReadRow, FdWriteRow},
};
use serde_json::json;

#[path = "../tasks/wafer_fabrication/instruments.rs"]
mod instruments;
use instruments::*;

const TERMINAL: u8 = 5;
const SESSION: u32 = 150;
const LITHO: u8 = 9;
const LITHO_INBOX: u8 = 10;
const LITHO_FILES: u8 = 11;
const LITHO_TERM: u8 = 12;
const DEP: u8 = 13;
const DEP_INBOX: u8 = 14;
const DEP_FILES: u8 = 15;
const DEP_TERM: u8 = 16;
const ETCH: u8 = 17;
const ETCH_INBOX: u8 = 18;
const ETCH_FILES: u8 = 19;
const ETCH_TERM: u8 = 20;
const CONTROL: u8 = 22;
const CONTROL_FILE: u8 = 23;
const CONTROL_FILES: u8 = 24;
const CONTROL_TERM: u8 = 25;
const CONTROL_OPERATOR: u8 = 26;
const MONITOR_STATION: u8 = 27;
const MEASURER: u8 = 28;
const MEASURE_FILE: u8 = 29;
const MEASURE_FILES: u8 = 30;
const MEASURE_TERM: u8 = 31;
const ETCH_OPERATOR: u8 = 33;
const ETCH_STATION: u8 = 34;
const SENSOR: u8 = 37;
const METER: u8 = 38;
const MONITOR_INPUT: u8 = 40;
const RIE: u8 = 44;
const ETCH_POWER: u8 = 45;
const PATTERN_WAFER: u8 = 46;
const PATTERN_OPERATOR: u8 = 47;
const PATTERN_METER: u8 = 48;
const PHOTO_METER: u8 = 41;
const ETCH_METER: u8 = 42;
const PROBE: u8 = 35;
const READOUT: u8 = 36;
type SampleResult<'a> = g::Msg<74, &'a [u8]>;
type FabReady<'a, const FAB: u8> = g::Msg<FAB, &'a [u8]>;
type Confirmation<'a, const LABEL: u8> = g::Msg<LABEL, WaferReading<'a, MONITOR>>;
const CONFIRM_LITHO: u8 = 40;
const CONFIRM_DEP: u8 = 41;
const CONFIRM_ETCH: u8 = 42;
const ALL_VALIDATED: u8 = 43;
type Disposition<'a, const INBOX: u8> = g::Msg<INBOX, &'a [u8]>;

const INBOX_FILE: ChoreoFsObject<'static> = ChoreoFsObject::readable(
    b"service/lot_disposition",
    ObjectId(4),
    FdSpec::new(4, 2),
    b"",
    FdBinding::read(FdReadRow::Base),
);

const VALIDATION_FILE: ChoreoFsObject<'static> = ChoreoFsObject::readable(
    b"service/process_validation",
    ObjectId(5),
    FdSpec::new(5, 2),
    b"",
    FdBinding::read(FdReadRow::Base),
);

const READY_FILE: ChoreoFsObject<'static> = ChoreoFsObject::writable(
    b"service/fab_ready",
    ObjectId(6),
    FdSpec::new(6, 64),
    FdBinding::write(FdWriteRow::Object),
);

fn measure_wafer<const FAB: u8, const KIND: u8>() -> impl Projectable + Copy
where
    InstrumentRequest<KIND>: SimulatorOperation,
{
    g::seq(
        g::send::<FAB, MEASURE_FILE, FabReady<FAB>>(),
        g::seq(
            g::seq(
                g::send::<MEASURE_FILE, METROLOGY, Dimension<KIND>>(),
                g::send::<METROLOGY, MEASURE_FILE, Reading<KIND>>(),
            )
            .roll(),
            g::send::<MEASURE_FILE, CONTROL_FILE, MeasurementReport<KIND>>(),
        ),
    )
}

fn recipient_io<const VM: u8, const INBOX: u8, const DATA: u8, const TERM: u8>()
-> impl Projectable + Copy {
    g::par(
        g::seq(
            file_metadata::<INBOX, DATA>().roll(),
            g::send::<INBOX, DATA, FileExit>(),
        ),
        g::seq(private_io::<VM, TERM>(), g::send::<VM, TERM, Exit>()),
    )
}

#[hibana_runtime::main]
async fn main() -> ShellResult<()> {
    let path = std::env::args()
        .nth(1)
        .ok_or_else(|| ShellError::message("usage: wafer-wasi GUEST.wasm"))?;
    let probe = g::send::<WAFER, METROLOGY, Measure>();
    let reading = g::send::<STEPPER, WAFER, Measured>();
    let failed = g::seq(
        g::send::<STAGE, STEPPER, OverlayFailed>(),
        g::seq(reading, probe),
    );
    let passed = g::seq(
        g::send::<STAGE, STEPPER, OverlayPassed>(),
        g::seq(g::send::<METROLOGY, STEPPER, Loan<STAGE>>(), reading),
    );
    let alignment = g::seq(
        g::send::<WAFER, STAGE, Mount>(),
        g::seq(
            g::send::<STEPPER, METROLOGY, Loan<STAGE>>(),
            g::par(
                g::send::<STEPPER, WAFER, Mounted>(),
                g::seq(probe, g::route(passed, g::seq(failed.roll(), passed))),
            ),
        ),
    );
    let check = g::seq(
        g::send::<CONTROL_FILE, MONITOR, CrossReference>(),
        g::seq(
            g::send::<MONITOR, MONITOR_INPUT, CrossReference>(),
            g::send::<MONITOR_INPUT, MONITOR, CrossReferenceResult>(),
        ),
    );
    let checked = g::send::<MONITOR_INPUT, CONTROL_FILE, CrossReferenceResult>();
    let released = g::seq(
        g::send::<MONITOR, MONITOR_INPUT, CrossReferencePassed>(),
        g::seq(
            g::send::<CONTROL_OPERATOR, MONITOR_STATION, Loan<MONITOR>>(),
            checked,
        ),
    );
    let validation = g::seq(
        check,
        g::route(
            released,
            g::seq(
                g::seq(
                    g::send::<MONITOR, MONITOR_INPUT, CrossReferenceFailed>(),
                    g::seq(checked, check),
                )
                .roll(),
                released,
            ),
        ),
    );
    let rate = g::seq(
        g::send::<DEP_INBOX, CVD_STAGE, Rate>(),
        g::seq(
            g::send::<CVD_STAGE, MONITOR_INPUT, MonitorRequest<DEPOSITION_RATE>>(),
            g::send::<MONITOR_INPUT, CVD_STAGE, MonitorReading<DEPOSITION_RATE>>(),
        ),
    );
    let rate_result = g::send::<MONITOR_INPUT, DEP_INBOX, RateResult>();
    let stable = g::seq(
        g::send::<CVD_STAGE, MONITOR_INPUT, RatePassed>(),
        g::seq(
            g::send::<MONITOR, DEPOSITION_CHAMBER, Loan<CVD_STAGE>>(),
            rate_result,
        ),
    );
    let recheck = g::seq(
        g::send::<CVD_STAGE, MONITOR_INPUT, RateFailed>(),
        g::seq(rate_result, rate),
    );
    let growth = g::seq(
        g::send::<DEP_INBOX, CVD_STAGE, Growth>(),
        g::seq(
            g::send::<DEPOSITION_CHAMBER, MONITOR, Loan<CVD_STAGE>>(),
            g::par(
                g::send::<DEPOSITION_CHAMBER, DEP_INBOX, Grown>(),
                g::seq(rate, g::route(stable, g::seq(recheck.roll(), stable))),
            ),
        ),
    );
    let dose = g::seq(
        g::send::<LITHO_INBOX, STAGE, MonitorRequest<EXPOSURE_DOSE>>(),
        g::seq(
            g::send::<STAGE, MONITOR_INPUT, MonitorRequest<EXPOSURE_DOSE>>(),
            g::send::<MONITOR_INPUT, STAGE, MonitorReading<EXPOSURE_DOSE>>(),
        ),
    );
    let dose_result = g::send::<MONITOR_INPUT, LITHO_INBOX, MonitorReading<EXPOSURE_DOSE>>();
    let dosed = g::seq(
        g::send::<STAGE, MONITOR_INPUT, DosePassed>(),
        g::seq(g::send::<PHOTO_METER, STEPPER, Loan<STAGE>>(), dose_result),
    );
    let retry_dose = g::seq(
        g::send::<STAGE, MONITOR_INPUT, DoseFailed>(),
        g::seq(dose_result, dose),
    );
    let exposure = g::seq(
        g::send::<LITHO_INBOX, STAGE, Expose>(),
        g::seq(
            g::send::<STEPPER, PHOTO_METER, Loan<STAGE>>(),
            g::par(
                g::send::<STEPPER, LITHO_INBOX, Exposed>(),
                g::seq(dose, g::route(dosed, g::seq(retry_dose.roll(), dosed))),
            ),
        ),
    );
    let etch_request = g::seq(
        g::send::<ETCH_INBOX, ETCH_CHAMBER, ReactiveEtch>(),
        g::seq(
            g::send::<ETCH_CHAMBER, PLASMA_INPUT, ReactiveEtch>(),
            g::send::<PLASMA_INPUT, ETCH_CHAMBER, EtchResult>(),
        ),
    );
    let etch_result = g::send::<PLASMA_INPUT, ETCH_INBOX, EtchResult>();
    let loaded = g::seq(
        g::send::<ETCH_CHAMBER, PLASMA_INPUT, EtchLoaded>(),
        g::seq(
            g::send::<ETCH_OPERATOR, ETCH_METER, Loan<ETCH_CHAMBER>>(),
            etch_result,
        ),
    );
    let retry_etch = g::seq(
        g::send::<ETCH_CHAMBER, PLASMA_INPUT, EtchFailed>(),
        g::seq(etch_result, etch_request),
    );
    let reactive = g::seq(
        etch_request,
        g::route(loaded, g::seq(retry_etch.roll(), loaded)),
    );
    let rate_check = g::seq(
        g::send::<ETCH_INBOX, ETCH_CHAMBER, MonitorRequest<ETCH_RATE>>(),
        g::seq(
            g::send::<ETCH_CHAMBER, MONITOR_INPUT, MonitorRequest<ETCH_RATE>>(),
            g::send::<MONITOR_INPUT, ETCH_CHAMBER, MonitorReading<ETCH_RATE>>(),
        ),
    );
    let etch_rate_result = g::send::<MONITOR_INPUT, ETCH_INBOX, MonitorReading<ETCH_RATE>>();
    let inspected = g::seq(
        g::send::<ETCH_CHAMBER, MONITOR_INPUT, EtchRatePassed>(),
        g::seq(
            g::send::<ETCH_CHAMBER, PLASMA_INPUT, EtchRatePassed>(),
            g::seq(
                g::send::<ETCH_METER, ETCH_STATION, Loan<ETCH_CHAMBER>>(),
                etch_rate_result,
            ),
        ),
    );
    let retry_rate = g::seq(
        g::send::<ETCH_CHAMBER, MONITOR_INPUT, EtchRateFailed>(),
        g::seq(etch_rate_result, rate_check),
    );
    let reactive = g::seq(
        reactive,
        g::seq(
            rate_check,
            g::route(inspected, g::seq(retry_rate.roll(), inspected)),
        ),
    );
    let pattern_request = g::seq(
        g::send::<LITHO_INBOX, PATTERN_WAFER, PatternEtch>(),
        g::seq(
            g::send::<PATTERN_WAFER, PLASMA_INPUT, PatternEtch>(),
            g::send::<PLASMA_INPUT, PATTERN_WAFER, PatternResult>(),
        ),
    );
    let pattern_result = g::send::<PLASMA_INPUT, LITHO_INBOX, PatternResult>();
    let patterned = g::seq(
        g::send::<PATTERN_WAFER, PLASMA_INPUT, PatternLoaded>(),
        g::seq(
            g::send::<PATTERN_OPERATOR, PATTERN_METER, Loan<PATTERN_WAFER>>(),
            pattern_result,
        ),
    );
    let retry_pattern = g::seq(
        g::send::<PATTERN_WAFER, PLASMA_INPUT, PatternFailed>(),
        g::seq(pattern_result, pattern_request),
    );
    let pattern = g::seq(
        pattern_request,
        g::route(patterned, g::seq(retry_pattern.roll(), patterned)),
    );
    let endpoint_check = g::seq(
        g::send::<LITHO_INBOX, PATTERN_WAFER, MonitorRequest<ETCH_ENDPOINT>>(),
        g::seq(
            g::send::<PATTERN_WAFER, MONITOR_INPUT, MonitorRequest<ETCH_ENDPOINT>>(),
            g::send::<MONITOR_INPUT, PATTERN_WAFER, MonitorReading<ETCH_ENDPOINT>>(),
        ),
    );
    let endpoint_result = g::send::<MONITOR_INPUT, LITHO_INBOX, MonitorReading<ETCH_ENDPOINT>>();
    let cleared = g::seq(
        g::send::<PATTERN_WAFER, MONITOR_INPUT, EndpointPassed>(),
        g::seq(
            g::send::<PATTERN_WAFER, PLASMA_INPUT, EndpointPassed>(),
            g::seq(
                g::send::<PATTERN_METER, PATTERN_OPERATOR, Loan<PATTERN_WAFER>>(),
                endpoint_result,
            ),
        ),
    );
    let recheck_endpoint = g::seq(
        g::send::<PATTERN_WAFER, MONITOR_INPUT, EndpointFailed>(),
        g::seq(endpoint_result, endpoint_check),
    );
    let pattern = g::seq(
        pattern,
        g::seq(
            endpoint_check,
            g::route(cleared, g::seq(recheck_endpoint.roll(), cleared)),
        ),
    );
    let checks = {
        let film = g::seq(
            g::send::<WAFER, DEPOSITION_CHAMBER, Film>(),
            g::send::<DEPOSITION_CHAMBER, WAFER, FilmResult>(),
        );
        let etch = g::seq(
            g::send::<WAFER, ETCH_CHAMBER, Uniformity>(),
            g::seq(
                g::send::<ETCH_CHAMBER, PLASMA_INPUT, Uniformity>(),
                g::seq(
                    g::send::<PLASMA_INPUT, ETCH_CHAMBER, UniformityResult>(),
                    g::send::<ETCH_CHAMBER, WAFER, UniformityResult>(),
                ),
            ),
        );
        g::route(g::seq(film, etch), g::seq(etch, film))
    };
    let fabricator_confirmations = g::par(
        g::par(
            g::seq(
                g::send::<CONTROL_FILE, LITHO_INBOX, Confirmation<CONFIRM_LITHO>>(),
                g::seq(
                    g::send::<LITHO_INBOX, METROLOGY, Dimension<ALIGNMENT_CHECK>>(),
                    g::send::<METROLOGY, LITHO_INBOX, Reading<ALIGNMENT_CHECK>>(),
                )
                .roll(),
            ),
            g::seq(
                g::send::<CONTROL_FILE, DEP_INBOX, Confirmation<CONFIRM_DEP>>(),
                g::seq(
                    g::send::<DEP_INBOX, METROLOGY, Dimension<THICKNESS_VERIFICATION>>(),
                    g::send::<METROLOGY, DEP_INBOX, Reading<THICKNESS_VERIFICATION>>(),
                )
                .roll(),
            ),
        ),
        g::send::<CONTROL_FILE, ETCH_INBOX, Confirmation<CONFIRM_ETCH>>(),
    );
    let notices = g::par(
        g::send::<WAFER, LITHO_INBOX, Disposition<LITHO_INBOX>>(),
        g::par(
            g::send::<WAFER, DEP_INBOX, Disposition<DEP_INBOX>>(),
            g::send::<WAFER, ETCH_INBOX, Disposition<ETCH_INBOX>>(),
        ),
    );
    let bootstrap = g::par(
        g::send::<ALIGNER, ENGINEER, Loan<WAFER>>(),
        g::par(
            g::send::<LITHO_FILES, LITHO, Loan<LITHO_INBOX>>(),
            g::par(
                g::send::<DEP_FILES, DEP, Loan<DEP_INBOX>>(),
                g::par(
                    g::send::<ETCH_FILES, ETCH, Loan<ETCH_INBOX>>(),
                    g::par(
                        g::send::<CONTROL_FILES, CONTROL, Loan<CONTROL_FILE>>(),
                        g::send::<MEASURE_FILES, MEASURER, Loan<MEASURE_FILE>>(),
                    ),
                ),
            ),
        ),
    );
    let services = g::par(
        recipient_io::<LITHO, LITHO_INBOX, LITHO_FILES, LITHO_TERM>(),
        g::par(
            recipient_io::<DEP, DEP_INBOX, DEP_FILES, DEP_TERM>(),
            g::par(
                recipient_io::<ETCH, ETCH_INBOX, ETCH_FILES, ETCH_TERM>(),
                g::par(
                    recipient_io::<CONTROL, CONTROL_FILE, CONTROL_FILES, CONTROL_TERM>(),
                    recipient_io::<MEASURER, MEASURE_FILE, MEASURE_FILES, MEASURE_TERM>(),
                ),
            ),
        ),
    );
    let source = g::seq(
        bootstrap,
        g::par(
            services,
            g::par(
                g::seq(
                    g::par(
                        g::seq(
                            g::seq(
                                g::par(
                                    g::seq(
                                        g::seq(exposure, pattern),
                                        measure_wafer::<LITHO_INBOX, CD>(),
                                    ),
                                    g::par(
                                        g::seq(growth, measure_wafer::<DEP_INBOX, THICKNESS>()),
                                        g::seq(reactive, measure_wafer::<ETCH_INBOX, PROFILE>()),
                                    ),
                                ),
                                validation,
                            ),
                            g::par(
                                fabricator_confirmations,
                                g::seq(
                                    g::send::<CONTROL_FILE, WAFER, Confirmation<ALL_VALIDATED>>(),
                                    g::seq(alignment, g::seq(checks, notices)),
                                ),
                            ),
                        ),
                        g::seq(
                            file_metadata::<WAFER, ALIGNER>().roll(),
                            g::send::<WAFER, ALIGNER, FileExit>(),
                        ),
                    ),
                    g::send::<ENGINEER, ALIGNER, Loan<WAFER>>(),
                ),
                g::seq(
                    private_io::<ENGINEER, TERMINAL>(),
                    g::send::<ENGINEER, TERMINAL, Exit>(),
                ),
            ),
        ),
    );
    // Every measurement, including measurement with a borrowed stage, uses this
    // one probe. Its next request follows the electronics' actual reply.
    let sample = g::route(
        g::route(
            g::send::<PROBE, READOUT, Dimension<CD>>(),
            g::route(
                g::send::<PROBE, READOUT, Dimension<THICKNESS>>(),
                g::send::<PROBE, READOUT, Dimension<PROFILE>>(),
            ),
        ),
        g::route(
            g::send::<PROBE, READOUT, Measure>(),
            g::route(
                g::send::<PROBE, READOUT, Dimension<ALIGNMENT_CHECK>>(),
                g::send::<PROBE, READOUT, Dimension<THICKNESS_VERIFICATION>>(),
            ),
        ),
    );
    let source = g::seq(
        g::send::<READOUT, METROLOGY, Loan<PROBE>>(),
        g::par(
            source,
            g::seq(sample, g::send::<READOUT, PROBE, SampleResult>()).roll(),
        ),
    );
    let mode = g::route(
        g::seq(
            g::send::<METER, SENSOR, MonitorRequest<DEPOSITION_RATE>>(),
            g::send::<SENSOR, METER, SensorPermit<DEPOSITION_RATE>>(),
        ),
        g::route(
            g::seq(
                g::send::<METER, SENSOR, MonitorRequest<EXPOSURE_DOSE>>(),
                g::send::<SENSOR, METER, SensorPermit<EXPOSURE_DOSE>>(),
            ),
            g::seq(
                g::send::<METER, SENSOR, MonitorRequest<ETCH_RATE>>(),
                g::send::<SENSOR, METER, SensorPermit<ETCH_RATE>>(),
            ),
        ),
    );
    let mode = g::route(
        mode,
        g::seq(
            g::send::<METER, SENSOR, CrossReference>(),
            g::send::<SENSOR, METER, SensorPermit<MONITOR>>(),
        ),
    );
    let mode = g::route(
        mode,
        g::seq(
            g::send::<METER, SENSOR, MonitorRequest<ETCH_ENDPOINT>>(),
            g::send::<SENSOR, METER, SensorPermit<ETCH_ENDPOINT>>(),
        ),
    );
    let source = g::seq(
        g::send::<SENSOR, MONITOR_INPUT, Loan<METER>>(),
        g::par(
            source,
            g::seq(mode, g::send::<METER, SENSOR, SampleResult>()).roll(),
        ),
    );
    let wet = g::route(
        g::send::<RIE, ETCH_POWER, LoadFailed>(),
        g::seq(
            g::send::<RIE, ETCH_POWER, LoadPassed>(),
            g::send::<RIE, ETCH_POWER, EtchRatePassed>(),
        ),
    );
    let patterned = g::route(
        g::send::<RIE, ETCH_POWER, LoadFailed>(),
        g::seq(
            g::send::<RIE, ETCH_POWER, LoadPassed>(),
            g::send::<RIE, ETCH_POWER, EndpointPassed>(),
        ),
    );
    let etching = g::route(
        g::seq(
            g::send::<RIE, ETCH_POWER, ReactiveEtch>(),
            g::seq(
                g::send::<ETCH_POWER, RIE, ChamberPermit<REACTIVE_ION_ETCH>>(),
                wet,
            ),
        ),
        g::route(
            g::seq(
                g::send::<RIE, ETCH_POWER, PatternEtch>(),
                g::seq(
                    g::send::<ETCH_POWER, RIE, ChamberPermit<PATTERN_ETCH>>(),
                    patterned,
                ),
            ),
            g::seq(
                g::send::<RIE, ETCH_POWER, Uniformity>(),
                g::seq(
                    g::send::<ETCH_POWER, RIE, ChamberPermit<ETCH_CHAMBER>>(),
                    g::send::<RIE, ETCH_POWER, UniformityResult>(),
                ),
            ),
        ),
    )
    .roll();
    let source = g::seq(
        g::send::<ETCH_POWER, PLASMA_INPUT, Loan<RIE>>(),
        g::par(source, etching),
    );
    let p_rie = project::<RIE>(&source);
    let p_power = project::<ETCH_POWER>(&source);
    let p_plasma = project::<PLASMA_INPUT>(&source);
    let p_pattern = project::<PATTERN_WAFER>(&source);
    let p_pattern_operator = project::<PATTERN_OPERATOR>(&source);
    let p_pattern_meter = project::<PATTERN_METER>(&source);
    let p_sensor = project::<SENSOR>(&source);
    let p_meter = project::<METER>(&source);
    let p_input = project::<MONITOR_INPUT>(&source);
    let p_photo = project::<PHOTO_METER>(&source);
    let p_etch_meter = project::<ETCH_METER>(&source);
    let p_probe = project::<PROBE>(&source);
    let p_readout = project::<READOUT>(&source);
    let aligner = project::<ALIGNER>(&source);
    let engineer = project::<ENGINEER>(&source);
    let wafer = project::<WAFER>(&source);
    let deposition_chamber = project::<DEPOSITION_CHAMBER>(&source);
    let etch_chamber = project::<ETCH_CHAMBER>(&source);
    let etch_operator = project::<ETCH_OPERATOR>(&source);
    let etch_station = project::<ETCH_STATION>(&source);
    let stepper = project::<STEPPER>(&source);
    let metrology = project::<METROLOGY>(&source);
    let stage = project::<STAGE>(&source);
    let cvd_stage = project::<CVD_STAGE>(&source);
    let terminal = project::<TERMINAL>(&source);
    let p_litho = project::<LITHO>(&source);
    let p_litho_inbox = project::<LITHO_INBOX>(&source);
    let p_litho_files = project::<LITHO_FILES>(&source);
    let p_litho_term = project::<LITHO_TERM>(&source);
    let p_dep = project::<DEP>(&source);
    let p_dep_inbox = project::<DEP_INBOX>(&source);
    let p_dep_files = project::<DEP_FILES>(&source);
    let p_dep_term = project::<DEP_TERM>(&source);
    let p_etch = project::<ETCH>(&source);
    let p_etch_inbox = project::<ETCH_INBOX>(&source);
    let p_etch_files = project::<ETCH_FILES>(&source);
    let p_etch_term = project::<ETCH_TERM>(&source);
    let p_monitor = project::<MONITOR>(&source);
    let p_control_operator = project::<CONTROL_OPERATOR>(&source);
    let p_monitor_station = project::<MONITOR_STATION>(&source);
    let p_control = project::<CONTROL>(&source);
    let p_control_file = project::<CONTROL_FILE>(&source);
    let p_control_files = project::<CONTROL_FILES>(&source);
    let p_control_term = project::<CONTROL_TERM>(&source);
    let p_measurer = project::<MEASURER>(&source);
    let p_measure_file = project::<MEASURE_FILE>(&source);
    let p_measure_files = project::<MEASURE_FILES>(&source);
    let p_measure_term = project::<MEASURE_TERM>(&source);
    let roles = std::collections::BTreeMap::from([
        ("RIE", RIE),
        ("ETCH_POWER", ETCH_POWER),
        ("PLASMA_INPUT", PLASMA_INPUT),
        ("PATTERN_WAFER", PATTERN_WAFER),
        ("PATTERN_OPERATOR", PATTERN_OPERATOR),
        ("PATTERN_METER", PATTERN_METER),
        ("METER", METER),
        ("SENSOR", SENSOR),
        ("MONITOR_INPUT", MONITOR_INPUT),
        ("PHOTO_METER", PHOTO_METER),
        ("ETCH_METER", ETCH_METER),
        ("PROBE", PROBE),
        ("READOUT", READOUT),
        ("ALIGNER", ALIGNER),
        ("ENGINEER", ENGINEER),
        ("WAFER", WAFER),
        ("TERMINAL", TERMINAL),
        ("DEPOSITION_CHAMBER", DEPOSITION_CHAMBER),
        ("ETCH_CHAMBER", ETCH_CHAMBER),
        ("STEPPER", STEPPER),
        ("METROLOGY", METROLOGY),
        ("STAGE", STAGE),
        ("CVD_STAGE", CVD_STAGE),
        ("LITHO", LITHO),
        ("LITHO_INBOX", LITHO_INBOX),
        ("LITHO_FILES", LITHO_FILES),
        ("LITHO_TERM", LITHO_TERM),
        ("DEP", DEP),
        ("DEP_INBOX", DEP_INBOX),
        ("DEP_FILES", DEP_FILES),
        ("DEP_TERM", DEP_TERM),
        ("ETCH", ETCH),
        ("ETCH_INBOX", ETCH_INBOX),
        ("ETCH_FILES", ETCH_FILES),
        ("ETCH_TERM", ETCH_TERM),
        ("ETCH_OPERATOR", ETCH_OPERATOR),
        ("ETCH_STATION", ETCH_STATION),
        ("MONITOR", MONITOR),
        ("CONTROL_OPERATOR", CONTROL_OPERATOR),
        ("MONITOR_STATION", MONITOR_STATION),
        ("CONTROL", CONTROL),
        ("CONTROL_FILE", CONTROL_FILE),
        ("CONTROL_FILES", CONTROL_FILES),
        ("CONTROL_TERM", CONTROL_TERM),
        ("MEASURER", MEASURER),
        ("MEASURE_FILE", MEASURE_FILE),
        ("MEASURE_FILES", MEASURE_FILES),
        ("MEASURE_TERM", MEASURE_TERM),
    ]);
    let projection = json!({
        "type":"compiled_projection", "format_version":1, "task":"15E", "session":SESSION,
        "roles":roles, "labels":std::collections::BTreeMap::from([
            ("Expose", Expose::LOGICAL_LABEL),
            ("Exposed", Exposed::LOGICAL_LABEL),
            ("PatternEtch", PatternEtch::LOGICAL_LABEL),
            ("PatternResult", PatternResult::LOGICAL_LABEL),
            ("ReactiveEtch", ReactiveEtch::LOGICAL_LABEL),
            ("EtchResult", EtchResult::LOGICAL_LABEL),
            ("Growth", Growth::LOGICAL_LABEL),
            ("Grown", Grown::LOGICAL_LABEL),
            ("MonitorRequest_EXPOSURE_DOSE", MonitorRequest::<EXPOSURE_DOSE>::LOGICAL_LABEL),
            ("MonitorReading_EXPOSURE_DOSE", MonitorReading::<EXPOSURE_DOSE>::LOGICAL_LABEL),
            ("Rate", Rate::LOGICAL_LABEL),
            ("RateResult", RateResult::LOGICAL_LABEL),
            ("CrossReference", CrossReference::LOGICAL_LABEL),
            ("CrossReferenceResult", CrossReferenceResult::LOGICAL_LABEL),
            ("Uniformity", Uniformity::LOGICAL_LABEL),
            ("UniformityResult", UniformityResult::LOGICAL_LABEL),
            ("Film", Film::LOGICAL_LABEL),
            ("FilmResult", FilmResult::LOGICAL_LABEL),
            ("Mount", Mount::LOGICAL_LABEL),
            ("Mounted", Mounted::LOGICAL_LABEL),
            ("Measure", Measure::LOGICAL_LABEL),
            ("Measured", Measured::LOGICAL_LABEL),
            ("SensorPermit_MONITOR", SensorPermit::<MONITOR>::LOGICAL_LABEL),
            ("ChamberPermit_ETCH_CHAMBER", ChamberPermit::<ETCH_CHAMBER>::LOGICAL_LABEL),
            ("LoadPassed", LoadPassed::LOGICAL_LABEL),
            ("LoadFailed", LoadFailed::LOGICAL_LABEL),
            ("DosePassed", DosePassed::LOGICAL_LABEL),
            ("DoseFailed", DoseFailed::LOGICAL_LABEL),
            ("EndpointPassed", EndpointPassed::LOGICAL_LABEL),
            ("EndpointFailed", EndpointFailed::LOGICAL_LABEL),
            ("EtchRatePassed", EtchRatePassed::LOGICAL_LABEL),
            ("EtchRateFailed", EtchRateFailed::LOGICAL_LABEL),
            ("RatePassed", RatePassed::LOGICAL_LABEL),
            ("RateFailed", RateFailed::LOGICAL_LABEL),
            ("CrossReferencePassed", CrossReferencePassed::LOGICAL_LABEL),
            ("CrossReferenceFailed", CrossReferenceFailed::LOGICAL_LABEL),
            ("PatternLoaded", PatternLoaded::LOGICAL_LABEL),
            ("PatternFailed", PatternFailed::LOGICAL_LABEL),
            ("EtchLoaded", EtchLoaded::LOGICAL_LABEL),
            ("EtchFailed", EtchFailed::LOGICAL_LABEL),
            ("etch_endpoint", ETCH_ENDPOINT),
            ("etch_rate", ETCH_RATE),
            ("cd", CD),
            ("thickness", THICKNESS),
            ("profile", PROFILE),
            ("alignment_check", ALIGNMENT_CHECK),
            ("thickness_verification", THICKNESS_VERIFICATION),
            ("reactive_ion_etch", REACTIVE_ION_ETCH),
            ("pattern_etch", PATTERN_ETCH),
            ("SampleResult", SampleResult::LOGICAL_LABEL),
            ("EtchLoan", Loan::<ETCH_CHAMBER>::LOGICAL_LABEL),
            ("ExposureDose", EXPOSURE_DOSE),
            ("OverlayPassed", OverlayPassed::LOGICAL_LABEL),
            ("OverlayFailed", OverlayFailed::LOGICAL_LABEL),
        ]),
        "programs":[
            rows(&p_rie),rows(&p_power),rows(&p_plasma),rows(&p_pattern),rows(&p_pattern_operator),rows(&p_pattern_meter),rows(&p_meter),rows(&p_sensor),rows(&p_input),rows(&p_photo),rows(&p_etch_meter),rows(&p_probe), rows(&p_readout), rows(&aligner), rows(&engineer), rows(&wafer), rows(&deposition_chamber),
            rows(&etch_chamber), rows(&etch_operator), rows(&etch_station), rows(&terminal), rows(&stepper), rows(&metrology), rows(&stage),
            rows(&p_litho), rows(&p_litho_inbox), rows(&p_litho_files), rows(&p_litho_term),
            rows(&p_dep), rows(&p_dep_inbox), rows(&p_dep_files), rows(&p_dep_term),
            rows(&p_etch), rows(&p_etch_inbox), rows(&p_etch_files), rows(&p_etch_term),rows(&p_monitor),rows(&p_control_operator),rows(&p_monitor_station),rows(&p_control),rows(&p_control_file),rows(&p_control_files),rows(&p_control_term),
            rows(&p_measurer), rows(&p_measure_file), rows(&p_measure_files), rows(&p_measure_term), rows(&cvd_stage)
        ]
    });
    if path == "--projection" {
        println!("{projection}");
        return Ok(());
    }
    let module = std::fs::read(path)?;
    let session = SessionId::new(SESSION);
    let (
        mut connection,
        mut io,
        [
            shell,
            litho_shell,
            dep_shell,
            etch_shell,
            control_shell,
            measure_shell,
        ],
    ) = Bus::open(
        [
            "INTEGRATION_ENGINEER",
            "LITHOGRAPHER",
            "DEPOSITOR",
            "ETCHER",
            "PROCESS_CONTROLLER",
            "METROLOGY_ENGINEER",
        ],
        [
            (session, RIE),
            (session, PATTERN_WAFER),
            (session, METER),
            (session, PROBE),
            (session, WAFER),
            (session, STAGE),
            (session, CVD_STAGE),
            (session, LITHO_INBOX),
            (session, DEP_INBOX),
            (session, ETCH_INBOX),
            (session, ETCH_CHAMBER),
            (session, CONTROL_FILE),
            (session, MONITOR),
            (session, MEASURE_FILE),
        ],
    );
    let (bus, effects) = connection.split();
    bus.projected(projection);
    let instrument = |path, fd: u8| {
        ChoreoFsObject::read_write(
            path,
            ObjectId(fd.into()),
            FdSpec::new(fd, 66),
            b"",
            FdBinding {
                read: Some(FdReadRow::Base),
                write: Some(FdWriteRow::Object),
                readdir: None,
            },
        )
    };
    let notice = |path, fd: u8| {
        ChoreoFsObject::writable(
            path,
            ObjectId(fd.into()),
            FdSpec::new(fd, 64),
            FdBinding::write(FdWriteRow::Object),
        )
    };
    let readable = |path, fd: u8| {
        ChoreoFsObject::readable(
            path,
            ObjectId(fd.into()),
            FdSpec::new(fd, 2),
            b"",
            FdBinding::read(FdReadRow::Base),
        )
    };
    let litho_files = [
        INBOX_FILE,
        VALIDATION_FILE,
        READY_FILE,
        instrument(b"service/run_stepper_exposure", 7),
        instrument(b"service/run_metrology_measurement", 8),
        instrument(b"service/check_monitoring/exposure_dose", 9),
        instrument(b"service/run_etch_process", 10),
        instrument(b"service/check_monitoring/etch_endpoint", 11),
    ];
    let depositor_files = [
        INBOX_FILE,
        VALIDATION_FILE,
        READY_FILE,
        instrument(b"service/run_deposition", 7),
        instrument(b"service/check_monitoring", 8),
        instrument(b"service/run_metrology_measurement", 9),
    ];
    let etcher_files = [
        INBOX_FILE,
        VALIDATION_FILE,
        READY_FILE,
        instrument(b"service/run_etch_process", 7),
        instrument(b"service/check_monitoring", 8),
    ];
    let measure_files = [
        instrument(b"service/pattern", 4),
        instrument(b"service/film", 5),
        instrument(b"service/etch", 6),
        instrument(b"service/cd_measurement", 7),
        instrument(b"service/thickness_measurement", 8),
        instrument(b"service/profile_measurement", 9),
    ];
    let control_files = [
        instrument(b"service/check_monitoring", 4),
        notice(b"service/validated_lithographer", 5),
        notice(b"service/validated_depositor", 6),
        notice(b"service/validated_etcher", 7),
        notice(b"service/all_validated", 8),
        readable(b"service/pattern_report", 9),
        readable(b"service/film_report", 10),
        readable(b"service/etch_report", 11),
    ];
    let files = [
        instrument(b"service/run_deposition", 4),
        instrument(b"service/run_etch_process", 5),
        instrument(b"service/run_stepper_exposure", 6),
        instrument(b"service/run_metrology_measurement", 7),
        notice(b"service/lot_lithographer", 8),
        notice(b"service/lot_depositor", 9),
        notice(b"service/lot_etcher", 10),
        ChoreoFsObject::readable(
            b"service/all_validated",
            ObjectId(11),
            FdSpec::new(11, 2),
            b"",
            FdBinding::read(FdReadRow::Base),
        ),
    ];
    let capture = |row| bus.capture(row);
    let mut slab = vec![0; 1024 * 1024];
    let mut storage = SessionKitStorage::uninit();
    let rv = storage.init().rendezvous(&mut slab, &bus.transport)?;
    rv.observe(&capture);
    let tap = rv.tap();
    let trial = async {
        let mut aligner = rv.enter(session, &aligner)?;
        let mut engineer = rv.enter(session, &engineer)?;
        let wafer = rv.enter(session, &wafer)?;
        let mut dep = rv.enter(session, &deposition_chamber)?;
        let mut etch_operator = rv.enter(session, &etch_operator)?;
        let mut etch = rv.enter(session, &etch_chamber)?;
        let mut etch_station = rv.enter(session, &etch_station)?;
        let mut terminal = rv.enter(session, &terminal)?;
        let mut stepper = rv.enter(session, &stepper)?;
        let mut stage = rv.enter(session, &stage)?;
        let mut cvd_stage = rv.enter(session, &cvd_stage)?;
        let mut operator = rv.enter(session, &p_control_operator)?;
        let mut monitor = rv.enter(session, &p_monitor)?;
        let mut station = rv.enter(session, &p_monitor_station)?;
        let litho_ep = rv.enter(session, &p_litho)?;
        let mut litho_files_ep = rv.enter(session, &p_litho_files)?;
        let mut litho_term_ep = rv.enter(session, &p_litho_term)?;
        let dep_ep = rv.enter(session, &p_dep)?;
        let mut dep_files_ep = rv.enter(session, &p_dep_files)?;
        let mut dep_term_ep = rv.enter(session, &p_dep_term)?;
        let mut etch_ep = rv.enter(session, &p_etch)?;
        let mut etch_files_ep = rv.enter(session, &p_etch_files)?;
        let mut etch_term_ep = rv.enter(session, &p_etch_term)?;
        let litho_inbox_ep = rv.enter(session, &p_litho_inbox)?;
        let dep_inbox_ep = rv.enter(session, &p_dep_inbox)?;
        let etch_inbox_ep = rv.enter(session, &p_etch_inbox)?;
        async {
            let mut vm = litho_ep;
            let inbox = vm.recv::<Loan<LITHO_INBOX>>().await?;
            let [
                mut disposition,
                mut validation,
                mut ready,
                mut expose,
                mut exposed,
                mut alignment_request,
                mut alignment_reading,
                mut dose,
                mut dose_reply,
                mut pattern,
                mut pattern_result,
                mut endpoint,
                mut endpoint_result,
            ] = [(); 13].map(|_| vec![0; GUEST_MEMORY_BYTES]);
            let file = ChoreoFsFile::new(INBOX_FILE, inbox)
                .read_as::<Disposition<LITHO_INBOX>>(&mut disposition)
                .file(VALIDATION_FILE)?
                .read_as::<Confirmation<CONFIRM_LITHO>>(&mut validation)
                .file(READY_FILE)?
                .write_as::<FabReady<LITHO_INBOX>>(&mut ready)
                .file(litho_files[3])?
                .write_as::<Expose>(&mut expose)
                .read_as::<Exposed>(&mut exposed)
                .file(litho_files[4])?
                .write_as::<Dimension<ALIGNMENT_CHECK>>(&mut alignment_request)
                .read_as::<Reading<ALIGNMENT_CHECK>>(&mut alignment_reading)
                .file(litho_files[5])?
                .write_as::<MonitorRequest<EXPOSURE_DOSE>>(&mut dose)
                .read_as::<MonitorReading<EXPOSURE_DOSE>>(&mut dose_reply)
                .file(litho_files[6])?
                .write_as::<PatternEtch>(&mut pattern)
                .read_as::<PatternResult>(&mut pattern_result)
                .file(litho_files[7])?
                .write_as::<MonitorRequest<ETCH_ENDPOINT>>(&mut endpoint)
                .read_as::<MonitorReading<ETCH_ENDPOINT>>(&mut endpoint_result);
            let files = ChoreoFsFiles::new(vm).file(file)?;
            let mut memory = vec![0; GUEST_MEMORY_BYTES];
            let mut storage = Box::new(HibanaWasiGuestStorage::uninit());
            let guest = storage.init(&module, GuestMemory::new(&mut memory), files)?;
            let (exit, files) = guest.run().await?;
            let status = exit.status() as i32;
            let (mut vm, (file, ())) = files.into_parts();
            file.into_endpoint().send::<FileExit>(&status).await?;
            vm.send::<Exit>(&status).await?;
            Ok(())
        };
        async {
            localside::file_metadata::run(&mut litho_files_ep, litho_inbox_ep, litho_files).await
        };
        async {
            localside::private_io::run(
                &mut litho_term_ep,
                PrivateFs::new(litho_shell, bus, &litho_files),
            )
            .await
        };
        async {
            let mut vm = dep_ep;
            let inbox = vm.recv::<Loan<DEP_INBOX>>().await?;
            let [
                mut disposition,
                mut validation,
                mut ready,
                mut grow,
                mut grown,
                mut rate,
                mut checked,
                mut thickness_request,
                mut thickness_reading,
            ] = [(); 9].map(|_| vec![0; GUEST_MEMORY_BYTES]);
            let file = ChoreoFsFile::new(INBOX_FILE, inbox)
                .read_as::<Disposition<DEP_INBOX>>(&mut disposition)
                .file(VALIDATION_FILE)?
                .read_as::<Confirmation<CONFIRM_DEP>>(&mut validation)
                .file(READY_FILE)?
                .write_as::<FabReady<DEP_INBOX>>(&mut ready)
                .file(depositor_files[3])?
                .write_as::<Growth>(&mut grow)
                .read_as::<Grown>(&mut grown)
                .file(depositor_files[4])?
                .write_as::<Rate>(&mut rate)
                .read_as::<RateResult>(&mut checked)
                .file(depositor_files[5])?
                .write_as::<Dimension<THICKNESS_VERIFICATION>>(&mut thickness_request)
                .read_as::<Reading<THICKNESS_VERIFICATION>>(&mut thickness_reading);
            let files = ChoreoFsFiles::new(vm).file(file)?;
            let mut memory = vec![0; GUEST_MEMORY_BYTES];
            let mut storage = Box::new(HibanaWasiGuestStorage::uninit());
            let guest = storage.init(&module, GuestMemory::new(&mut memory), files)?;
            let (exit, files) = guest.run().await?;
            let status = exit.status() as i32;
            let (mut vm, (file, ())) = files.into_parts();
            file.into_endpoint().send::<FileExit>(&status).await?;
            vm.send::<Exit>(&status).await?;
            Ok(())
        };
        async {
            localside::file_metadata::run(&mut dep_files_ep, dep_inbox_ep, depositor_files).await
        };
        async {
            localside::private_io::run(
                &mut dep_term_ep,
                PrivateFs::new(dep_shell, bus, &depositor_files),
            )
            .await
        };
        async {
            let inbox = etch_ep.recv::<Loan<ETCH_INBOX>>().await?;
            let mut buffer = vec![0; GUEST_MEMORY_BYTES];
            let mut validation = vec![0; GUEST_MEMORY_BYTES];
            let mut ready = vec![0; GUEST_MEMORY_BYTES];
            let mut rate = vec![0; GUEST_MEMORY_BYTES];
            let mut rate_reply = vec![0; GUEST_MEMORY_BYTES];
            let mut process = vec![0; GUEST_MEMORY_BYTES];
            let mut result = vec![0; GUEST_MEMORY_BYTES];
            let file = ChoreoFsFile::new(INBOX_FILE, inbox)
                .read_as::<Disposition<ETCH_INBOX>>(&mut buffer)
                .file(VALIDATION_FILE)?
                .read_as::<Confirmation<CONFIRM_ETCH>>(&mut validation)
                .file(READY_FILE)?
                .write_as::<FabReady<ETCH_INBOX>>(&mut ready)
                .file(etcher_files[3])?
                .write_as::<ReactiveEtch>(&mut process)
                .read_as::<EtchResult>(&mut result)
                .file(etcher_files[4])?
                .write_as::<MonitorRequest<ETCH_RATE>>(&mut rate)
                .read_as::<MonitorReading<ETCH_RATE>>(&mut rate_reply);
            let files = ChoreoFsFiles::new(etch_ep).file(file)?;
            let mut memory = vec![0; GUEST_MEMORY_BYTES];
            let mut storage = Box::new(HibanaWasiGuestStorage::uninit());
            let guest = storage.init(&module, GuestMemory::new(&mut memory), files)?;
            let (exit, files) = guest.run().await?;
            let status = exit.status() as i32;
            let (mut vm, (file, ())) = files.into_parts();
            let mut inbox = file.into_endpoint();
            inbox.send::<FileExit>(&status).await?;
            vm.send::<Exit>(&status).await?;
            Ok(())
        };
        async {
            localside::file_metadata::run(&mut etch_files_ep, etch_inbox_ep, etcher_files).await
        };
        async {
            localside::private_io::run(
                &mut etch_term_ep,
                PrivateFs::new(etch_shell, bus, &etcher_files),
            )
            .await
        };
        let mut measurer = rv.enter(session, &p_measurer)?;
        let mut measure_data = rv.enter(session, &p_measure_files)?;
        let measure_file = rv.enter(session, &p_measure_file)?;
        let mut measure_term = rv.enter(session, &p_measure_term)?;
        async {
            localside::file_metadata::run(&mut measure_data, measure_file, measure_files).await
        };
        async {
            localside::private_io::run(
                &mut measure_term,
                PrivateFs::new(measure_shell, bus, &measure_files),
            )
            .await
        };
        async {
            let file = measurer.recv::<Loan<MEASURE_FILE>>().await?;
            let [
                mut pattern,
                mut film,
                mut etch,
                mut pattern_report,
                mut film_report,
                mut etch_report,
            ] = [(); 6].map(|_| vec![0; GUEST_MEMORY_BYTES]);
            let [
                mut cd,
                mut thickness,
                mut profile,
                mut cd_reply,
                mut thickness_reply,
                mut profile_reply,
            ] = [(); 6].map(|_| vec![0; GUEST_MEMORY_BYTES]);
            let file = ChoreoFsFile::new(measure_files[0], file)
                .read_as::<FabReady<LITHO_INBOX>>(&mut pattern)
                .write_as::<MeasurementReport<CD>>(&mut pattern_report)
                .file(measure_files[1])?
                .read_as::<FabReady<DEP_INBOX>>(&mut film)
                .write_as::<MeasurementReport<THICKNESS>>(&mut film_report)
                .file(measure_files[2])?
                .read_as::<FabReady<ETCH_INBOX>>(&mut etch)
                .write_as::<MeasurementReport<PROFILE>>(&mut etch_report)
                .file(measure_files[3])?
                .write_as::<Dimension<CD>>(&mut cd)
                .read_as::<Reading<CD>>(&mut cd_reply)
                .file(measure_files[4])?
                .write_as::<Dimension<THICKNESS>>(&mut thickness)
                .read_as::<Reading<THICKNESS>>(&mut thickness_reply)
                .file(measure_files[5])?
                .write_as::<Dimension<PROFILE>>(&mut profile)
                .read_as::<Reading<PROFILE>>(&mut profile_reply);
            let files = ChoreoFsFiles::new(measurer).file(file)?;
            let mut memory = vec![0; GUEST_MEMORY_BYTES];
            let mut storage = Box::new(HibanaWasiGuestStorage::uninit());
            let guest = storage.init(&module, GuestMemory::new(&mut memory), files)?;
            let (exit, files) = guest.run().await?;
            let status = exit.status() as i32;
            let (mut measurer, (file, ())) = files.into_parts();
            file.into_endpoint().send::<FileExit>(&status).await?;
            measurer.send::<Exit>(&status).await?;
            Ok(())
        };
        let mut control = rv.enter(session, &p_control)?;
        let mut control_data = rv.enter(session, &p_control_files)?;
        let control_file = rv.enter(session, &p_control_file)?;
        let mut control_term = rv.enter(session, &p_control_term)?;
        async {
            localside::file_metadata::run(&mut control_data, control_file, control_files).await
        };
        async {
            localside::private_io::run(
                &mut control_term,
                PrivateFs::new(control_shell, bus, &control_files),
            )
            .await
        };
        async {
            let file = control.recv::<Loan<CONTROL_FILE>>().await?;
            let [mut check, mut result, mut litho, mut dep, mut etch, mut all] =
                [(); 6].map(|_| vec![0; GUEST_MEMORY_BYTES]);
            let [mut pattern, mut film, mut etch_report] =
                [(); 3].map(|_| vec![0; GUEST_MEMORY_BYTES]);
            let file = ChoreoFsFile::new(control_files[0], file)
                .write_as::<CrossReference>(&mut check)
                .read_as::<CrossReferenceResult>(&mut result)
                .file(control_files[1])?
                .write_as::<Confirmation<CONFIRM_LITHO>>(&mut litho)
                .file(control_files[2])?
                .write_as::<Confirmation<CONFIRM_DEP>>(&mut dep)
                .file(control_files[3])?
                .write_as::<Confirmation<CONFIRM_ETCH>>(&mut etch)
                .file(control_files[4])?
                .write_as::<Confirmation<ALL_VALIDATED>>(&mut all)
                .file(control_files[5])?
                .read_as::<MeasurementReport<CD>>(&mut pattern)
                .file(control_files[6])?
                .read_as::<MeasurementReport<THICKNESS>>(&mut film)
                .file(control_files[7])?
                .read_as::<MeasurementReport<PROFILE>>(&mut etch_report);
            let files = ChoreoFsFiles::new(control).file(file)?;
            let mut memory = vec![0; GUEST_MEMORY_BYTES];
            let mut storage = Box::new(HibanaWasiGuestStorage::uninit());
            let guest = storage.init(&module, GuestMemory::new(&mut memory), files)?;
            let (exit, files) = guest.run().await?;
            let status = exit.status() as i32;
            let (mut control, (file, ())) = files.into_parts();
            file.into_endpoint().send::<FileExit>(&status).await?;
            control.send::<Exit>(&status).await?;
            Ok(())
        };
        async {
            let mut loaded = monitor.recv::<Loan<CVD_STAGE>>().await?;
            loop {
                let input = InstrumentRequest::<DEPOSITION_RATE>::try_from(
                    loaded.recv::<Rate>().await?.as_bytes(),
                )?;
                loaded
                    .send::<MonitorRequest<DEPOSITION_RATE>>(&input)
                    .await?;
                let bytes = loaded
                    .recv::<MonitorReading<DEPOSITION_RATE>>()
                    .await?
                    .to_vec();
                if decode(&bytes)?["success"] == true {
                    loaded
                        .send::<RatePassed>(&ReviewReply::from_bytes(&bytes)?)
                        .await?;
                    monitor.send::<Loan<CVD_STAGE>>(loaded).await?;
                    break;
                }

                loaded
                    .send::<RateFailed>(&ReviewReply::from_bytes(&bytes)?)
                    .await?;
            }
            loop {
                let check = InstrumentRequest::<MONITOR>::try_from(
                    monitor.recv::<CrossReference>().await?.as_bytes(),
                )?;
                monitor.send::<CrossReference>(&check).await?;
                let bytes = monitor.recv::<CrossReferenceResult>().await?.to_vec();
                if decode(&bytes)?["success"] == true {
                    monitor
                        .send::<CrossReferencePassed>(&ReviewReply::from_bytes(&bytes)?)
                        .await?;
                    operator.send::<Loan<MONITOR>>(monitor).await?;
                    return Ok(());
                }
                monitor
                    .send::<CrossReferenceFailed>(&ReviewReply::from_bytes(&bytes)?)
                    .await?;
            }
        };
        async {
            let _finished = station.recv::<Loan<MONITOR>>().await?;
            Ok(())
        };
        async {
            let mut holder = rv.enter(session, &p_photo)?;
            let mut loaded = holder.recv::<Loan<STAGE>>().await?;
            loop {
                let input = InstrumentRequest::<EXPOSURE_DOSE>::try_from(
                    loaded
                        .recv::<MonitorRequest<EXPOSURE_DOSE>>()
                        .await?
                        .as_bytes(),
                )?;
                loaded.send::<MonitorRequest<EXPOSURE_DOSE>>(&input).await?;
                let bytes = loaded
                    .recv::<MonitorReading<EXPOSURE_DOSE>>()
                    .await?
                    .to_vec();
                if decode(&bytes)?["success"] == true {
                    loaded
                        .send::<DosePassed>(&ReviewReply::from_bytes(&bytes)?)
                        .await?;
                    holder.send::<Loan<STAGE>>(loaded).await?;
                    return Ok(());
                }

                loaded
                    .send::<DoseFailed>(&ReviewReply::from_bytes(&bytes)?)
                    .await?;
            }
        };
        async {
            let mut holder = rv.enter(session, &p_pattern_meter)?;
            let mut loaded = holder.recv::<Loan<PATTERN_WAFER>>().await?;
            loop {
                let input = InstrumentRequest::<ETCH_ENDPOINT>::try_from(
                    loaded
                        .recv::<MonitorRequest<ETCH_ENDPOINT>>()
                        .await?
                        .as_bytes(),
                )?;
                loaded.send::<MonitorRequest<ETCH_ENDPOINT>>(&input).await?;
                let bytes = loaded
                    .recv::<MonitorReading<ETCH_ENDPOINT>>()
                    .await?
                    .to_vec();
                if decode(&bytes)?["success"] == true {
                    loaded
                        .send::<EndpointPassed>(&ReviewReply::from_bytes(&bytes)?)
                        .await?;
                    loaded
                        .send::<EndpointPassed>(&ReviewReply::from_bytes(&bytes)?)
                        .await?;
                    holder.send::<Loan<PATTERN_WAFER>>(loaded).await?;
                    return Ok(());
                }

                loaded
                    .send::<EndpointFailed>(&ReviewReply::from_bytes(&bytes)?)
                    .await?;
            }
        };
        async {
            let mut holder = rv.enter(session, &p_etch_meter)?;
            let mut loaded = holder.recv::<Loan<ETCH_CHAMBER>>().await?;
            loop {
                let input = InstrumentRequest::<ETCH_RATE>::try_from(
                    loaded.recv::<MonitorRequest<ETCH_RATE>>().await?.as_bytes(),
                )?;
                loaded.send::<MonitorRequest<ETCH_RATE>>(&input).await?;
                let bytes = loaded.recv::<MonitorReading<ETCH_RATE>>().await?.to_vec();
                if decode(&bytes)?["success"] == true {
                    loaded
                        .send::<EtchRatePassed>(&ReviewReply::from_bytes(&bytes)?)
                        .await?;
                    loaded
                        .send::<EtchRatePassed>(&ReviewReply::from_bytes(&bytes)?)
                        .await?;
                    holder.send::<Loan<ETCH_CHAMBER>>(loaded).await?;
                    return Ok(());
                }

                loaded
                    .send::<EtchRateFailed>(&ReviewReply::from_bytes(&bytes)?)
                    .await?;
            }
        };
        async {
            let exposed = effects.submit(stage.recv::<Expose>().await?)?;
            stepper.send::<Loan<STAGE>>(stage).await?;
            let mut stage = async {
                let [mut exposure, mut release] = stepper.par()?;
                async {
                    exposure
                        .send::<Exposed>(&encode(&exposed.await?)?.as_slice())
                        .await?;
                };
                let stage = release.recv::<Loan<STAGE>>().await?;
                Ok(stage)
            }
            .await?;
            let mounted = effects.submit(stage.recv::<Mount>().await?)?;
            stepper.send::<Loan<STAGE>>(stage).await?;
            let [mut load, mut release] = stepper.par()?;
            async {
                let result = mounted.await?;
                load.send::<Mounted>(&encode(&result)?.as_slice()).await?;
                Ok(())
            };
            async {
                loop {
                    let report = release.offer().await?;
                    match report.label() {
                        OverlayFailed::LOGICAL_LABEL => {
                            let bytes = report.recv::<OverlayFailed>().await?.to_vec();
                            release.send::<Measured>(&bytes.as_slice()).await?;
                        }
                        OverlayPassed::LOGICAL_LABEL => {
                            let bytes = report.recv::<OverlayPassed>().await?.to_vec();
                            let _returned = release.recv::<Loan<STAGE>>().await?;
                            release.send::<Measured>(&bytes.as_slice()).await?;
                            return Ok(());
                        }
                        label => {
                            return Err(ShellError::message(format!(
                                "unexpected overlay result {label}"
                            )));
                        }
                    }
                }
            };
            Ok(())
        };
        async {
            localside::file_metadata::run(&mut aligner, wafer, files).await?;
            let _returned = aligner.recv::<Loan<WAFER>>().await?;
            Ok(())
        };
        async {
            let growth = effects.submit(cvd_stage.recv::<Growth>().await?)?;
            dep.send::<Loan<CVD_STAGE>>(cvd_stage).await?;
            async {
                let [mut load, mut release] = dep.par()?;
                async {
                    load.send::<Grown>(&encode(&growth.await?)?.as_slice())
                        .await?;
                    Ok(())
                };
                async {
                    let _stage = release.recv::<Loan<CVD_STAGE>>().await?;
                    Ok(())
                };
                Ok(())
            }
            .await?;
            let actual = effects.apply(dep.recv::<Film>().await?).await?;
            dep.send::<FilmResult>(&encode(&actual)?.as_slice()).await?;
            Ok(())
        };
        async {
            let mut holder = rv.enter(session, &p_pattern_operator)?;
            let mut wafer = rv.enter(session, &p_pattern)?;
            loop {
                let args = InstrumentRequest::<PATTERN_ETCH>::try_from(
                    wafer.recv::<PatternEtch>().await?.as_bytes(),
                )?;
                wafer.send::<PatternEtch>(&args).await?;
                let bytes = wafer.recv::<PatternResult>().await?.to_vec();
                if decode(&bytes)?["success"] == true {
                    wafer
                        .send::<PatternLoaded>(&ReviewReply::from_bytes(&bytes)?)
                        .await?;
                    holder.send::<Loan<PATTERN_WAFER>>(wafer).await?;
                    let _unloaded = holder.recv::<Loan<PATTERN_WAFER>>().await?;
                    return Ok(());
                }
                wafer
                    .send::<PatternFailed>(&ReviewReply::from_bytes(&bytes)?)
                    .await?;
            }
        };
        async {
            loop {
                let args = InstrumentRequest::<REACTIVE_ION_ETCH>::try_from(
                    etch.recv::<ReactiveEtch>().await?.as_bytes(),
                )?;
                etch.send::<ReactiveEtch>(&args).await?;
                let bytes = etch.recv::<EtchResult>().await?.to_vec();
                if decode(&bytes)?["success"] == true {
                    etch.send::<EtchLoaded>(&ReviewReply::from_bytes(&bytes)?)
                        .await?;
                    etch_operator.send::<Loan<ETCH_CHAMBER>>(etch).await?;
                    return Ok(());
                }
                etch.send::<EtchFailed>(&ReviewReply::from_bytes(&bytes)?)
                    .await?;
            }
        };
        async {
            let mut chamber = etch_station.recv::<Loan<ETCH_CHAMBER>>().await?;
            let args = InstrumentRequest::<ETCH_CHAMBER>::try_from(
                chamber.recv::<Uniformity>().await?.as_bytes(),
            )?;
            chamber.send::<Uniformity>(&args).await?;
            let bytes = chamber.recv::<UniformityResult>().await?.to_vec();
            chamber.send::<UniformityResult>(&bytes.as_slice()).await?;
            Ok(())
        };
        async {
            localside::private_io::run(&mut terminal, PrivateFs::new(shell, bus, &files)).await
        };
        async {
            let wafer = engineer.recv::<Loan<WAFER>>().await?;
            let [
                mut film,
                mut film_result,
                mut etch,
                mut etch_result,
                mut mount,
                mut mounted,
                mut measure,
                mut measured,
                mut litho_note,
                mut dep_note,
                mut etch_note,
                mut validated,
            ] = [(); 12].map(|_| vec![0; GUEST_MEMORY_BYTES]);
            let wafer = ChoreoFsFile::new(files[0], wafer)
                .write_as::<Film>(&mut film)
                .read_as::<FilmResult>(&mut film_result)
                .file(files[1])?
                .write_as::<Uniformity>(&mut etch)
                .read_as::<UniformityResult>(&mut etch_result)
                .file(files[2])?
                .write_as::<Mount>(&mut mount)
                .read_as::<Mounted>(&mut mounted)
                .file(files[3])?
                .write_as::<Measure>(&mut measure)
                .read_as::<Measured>(&mut measured)
                .file(files[4])?
                .write_as::<Disposition<LITHO_INBOX>>(&mut litho_note)
                .file(files[5])?
                .write_as::<Disposition<DEP_INBOX>>(&mut dep_note)
                .file(files[6])?
                .write_as::<Disposition<ETCH_INBOX>>(&mut etch_note)
                .file(files[7])?
                .read_as::<Confirmation<ALL_VALIDATED>>(&mut validated);
            let bound = ChoreoFsFiles::new(engineer).file(wafer)?;
            let mut memory = vec![0; GUEST_MEMORY_BYTES];
            let mut storage = Box::new(HibanaWasiGuestStorage::uninit());
            let guest = storage.init(&module, GuestMemory::new(&mut memory), bound)?;
            let (exit, files) = guest.run().await?;
            let status = exit.status() as i32;
            let (mut engineer, (file, ())) = files.into_parts();
            let mut wafer = file.into_endpoint();
            wafer.send::<FileExit>(&status).await?;
            engineer.send::<Loan<WAFER>>(wafer).await?;
            engineer.send::<Exit>(&status).await?;
            Ok(())
        };
        Ok(())
    };
    let instruments = async {
        async {
            let mut power = rv.enter(session, &p_power)?;
            let chamber = rv.enter(session, &p_rie)?;
            power.send::<Loan<RIE>>(chamber).await?;
            loop {
                let load = power.offer().await?;
                match load.label() {
                    ReactiveEtch::LOGICAL_LABEL => {
                        let args = InstrumentRequest::<REACTIVE_ION_ETCH>::try_from(
                            load.recv::<ReactiveEtch>().await?.as_bytes(),
                        )?;
                        power
                            .send::<ChamberPermit<REACTIVE_ION_ETCH>>(&args)
                            .await?;
                        let outcome = power.offer().await?;
                        if outcome.label() == LoadFailed::LOGICAL_LABEL {
                            outcome.recv::<LoadFailed>().await?;
                            continue;
                        }
                        let wafer = decode(outcome.recv::<LoadPassed>().await?)?;
                        let readout = decode(power.recv::<EtchRatePassed>().await?)?;
                        if wafer["data"]["receipt"].as_u64().is_none()
                            || readout["data"]["inputs"] != json!([wafer["data"]["receipt"]])
                            || readout["data"]["check_type"] != "etch_rate"
                            || readout["data"]["engineer"] != "ETCHER"
                        {
                            return Err(ShellError::message(
                                "etch release does not identify the loaded wafer",
                            ));
                        }
                    }
                    PatternEtch::LOGICAL_LABEL => {
                        let args = InstrumentRequest::<PATTERN_ETCH>::try_from(
                            load.recv::<PatternEtch>().await?.as_bytes(),
                        )?;
                        power.send::<ChamberPermit<PATTERN_ETCH>>(&args).await?;
                        let outcome = power.offer().await?;
                        if outcome.label() == LoadFailed::LOGICAL_LABEL {
                            outcome.recv::<LoadFailed>().await?;
                            continue;
                        }
                        let wafer = decode(outcome.recv::<LoadPassed>().await?)?;
                        let readout = decode(power.recv::<EndpointPassed>().await?)?;
                        if wafer["data"]["receipt"].as_u64().is_none()
                            || readout["data"]["inputs"] != json!([wafer["data"]["receipt"]])
                            || readout["data"]["check_type"] != "etch_endpoint"
                            || readout["data"]["engineer"] != "LITHOGRAPHER"
                        {
                            return Err(ShellError::message(
                                "etch release does not identify the loaded wafer",
                            ));
                        }
                    }
                    Uniformity::LOGICAL_LABEL => {
                        let args = InstrumentRequest::<ETCH_CHAMBER>::try_from(
                            load.recv::<Uniformity>().await?.as_bytes(),
                        )?;
                        power.send::<ChamberPermit<ETCH_CHAMBER>>(&args).await?;
                        power.recv::<UniformityResult>().await?;
                    }
                    label => {
                        return Err(ShellError::message(format!(
                            "unexpected chamber mode {label}"
                        )));
                    }
                }
            }
        };
        async {
            let mut station = rv.enter(session, &p_plasma)?;
            let mut chamber = station.recv::<Loan<RIE>>().await?;
            loop {
                let load = station.offer().await?;
                match load.label() {
                    ReactiveEtch::LOGICAL_LABEL => {
                        let sample = load.recv::<ReactiveEtch>().await?;
                        let args =
                            InstrumentRequest::<REACTIVE_ION_ETCH>::try_from(sample.as_bytes())?;
                        chamber.send::<ReactiveEtch>(&args).await?;
                        let permit = chamber.recv::<ChamberPermit<REACTIVE_ION_ETCH>>().await?;
                        let result = effects.apply_pair(sample, permit).await?;
                        let bytes = encode(&result)?;
                        if result["success"] == true {
                            chamber
                                .send::<LoadPassed>(&ReviewReply::from_bytes(&bytes)?)
                                .await?;
                        } else {
                            chamber
                                .send::<LoadFailed>(&ReviewReply::from_bytes(&bytes)?)
                                .await?;
                        }
                        station.send::<EtchResult>(&bytes.as_slice()).await?;
                        if result["success"] == true {
                            let reported = station.recv::<EtchLoaded>().await?.to_vec();
                            station.send::<EtchResult>(&reported.as_slice()).await?;
                            let checked = station.recv::<EtchRatePassed>().await?.to_vec();
                            chamber
                                .send::<EtchRatePassed>(&ReviewReply::from_bytes(&checked)?)
                                .await?;
                        } else {
                            let reported = station.recv::<EtchFailed>().await?.to_vec();
                            station.send::<EtchResult>(&reported.as_slice()).await?;
                        }
                    }
                    PatternEtch::LOGICAL_LABEL => {
                        let sample = load.recv::<PatternEtch>().await?;
                        let args = InstrumentRequest::<PATTERN_ETCH>::try_from(sample.as_bytes())?;
                        chamber.send::<PatternEtch>(&args).await?;
                        let permit = chamber.recv::<ChamberPermit<PATTERN_ETCH>>().await?;
                        let result = effects.apply_pair(sample, permit).await?;
                        let bytes = encode(&result)?;
                        if result["success"] == true {
                            chamber
                                .send::<LoadPassed>(&ReviewReply::from_bytes(&bytes)?)
                                .await?;
                        } else {
                            chamber
                                .send::<LoadFailed>(&ReviewReply::from_bytes(&bytes)?)
                                .await?;
                        }
                        station.send::<PatternResult>(&bytes.as_slice()).await?;
                        if result["success"] == true {
                            let reported = station.recv::<PatternLoaded>().await?.to_vec();
                            station.send::<PatternResult>(&reported.as_slice()).await?;
                            let checked = station.recv::<EndpointPassed>().await?.to_vec();
                            chamber
                                .send::<EndpointPassed>(&ReviewReply::from_bytes(&checked)?)
                                .await?;
                        } else {
                            let reported = station.recv::<PatternFailed>().await?.to_vec();
                            station.send::<PatternResult>(&reported.as_slice()).await?;
                        }
                    }
                    Uniformity::LOGICAL_LABEL => {
                        let sample = load.recv::<Uniformity>().await?;
                        let args = InstrumentRequest::<ETCH_CHAMBER>::try_from(sample.as_bytes())?;
                        chamber.send::<Uniformity>(&args).await?;
                        let permit = chamber.recv::<ChamberPermit<ETCH_CHAMBER>>().await?;
                        let result = encode(&effects.apply_pair(sample, permit).await?)?;
                        chamber.send::<UniformityResult>(&result.as_slice()).await?;
                        station.send::<UniformityResult>(&result.as_slice()).await?;
                    }
                    label => {
                        return Err(ShellError::message(format!(
                            "unexpected plasma request {label}"
                        )));
                    }
                }
            }
        };
        async {
            let mut sensor = rv.enter(session, &p_sensor)?;
            let meter = rv.enter(session, &p_meter)?;
            sensor.send::<Loan<METER>>(meter).await?;
            loop {
                let mode = sensor.offer().await?;
                match mode.label() {
                    DEPOSITION_RATE => {
                        let request = InstrumentRequest::<DEPOSITION_RATE>::try_from(
                            mode.recv::<MonitorRequest<DEPOSITION_RATE>>()
                                .await?
                                .as_bytes(),
                        )?;
                        sensor
                            .send::<SensorPermit<DEPOSITION_RATE>>(&request)
                            .await?;
                    }
                    EXPOSURE_DOSE => {
                        let request = InstrumentRequest::<EXPOSURE_DOSE>::try_from(
                            mode.recv::<MonitorRequest<EXPOSURE_DOSE>>()
                                .await?
                                .as_bytes(),
                        )?;
                        sensor.send::<SensorPermit<EXPOSURE_DOSE>>(&request).await?;
                    }
                    ETCH_RATE => {
                        let request = InstrumentRequest::<ETCH_RATE>::try_from(
                            mode.recv::<MonitorRequest<ETCH_RATE>>().await?.as_bytes(),
                        )?;
                        sensor.send::<SensorPermit<ETCH_RATE>>(&request).await?;
                    }
                    ETCH_ENDPOINT => {
                        let request = InstrumentRequest::<ETCH_ENDPOINT>::try_from(
                            mode.recv::<MonitorRequest<ETCH_ENDPOINT>>()
                                .await?
                                .as_bytes(),
                        )?;
                        sensor.send::<SensorPermit<ETCH_ENDPOINT>>(&request).await?;
                    }
                    CrossReference::LOGICAL_LABEL => {
                        let check = InstrumentRequest::<MONITOR>::try_from(
                            mode.recv::<CrossReference>().await?.as_bytes(),
                        )?;
                        sensor.send::<SensorPermit<MONITOR>>(&check).await?;
                    }
                    label => {
                        return Err(ShellError::message(format!(
                            "unexpected monitoring mode {label}"
                        )));
                    }
                }
                sensor.recv::<SampleResult>().await?;
            }
        };
        async {
            let mut input = rv.enter(session, &p_input)?;
            let mut meter = input.recv::<Loan<METER>>().await?;
            loop {
                let mode = input.offer().await?;
                match mode.label() {
                    DEPOSITION_RATE => {
                        let sample = mode.recv::<MonitorRequest<DEPOSITION_RATE>>().await?;
                        let args =
                            InstrumentRequest::<DEPOSITION_RATE>::try_from(sample.as_bytes())?;
                        meter.send::<MonitorRequest<DEPOSITION_RATE>>(&args).await?;
                        let permit = meter.recv::<SensorPermit<DEPOSITION_RATE>>().await?;
                        let result = effects.apply_pair(sample, permit).await?;
                        let bytes = encode(&result)?;
                        meter.send::<SampleResult>(&bytes.as_slice()).await?;
                        input
                            .send::<MonitorReading<DEPOSITION_RATE>>(&bytes.as_slice())
                            .await?;
                        let reported = if result["success"] == true {
                            input.recv::<RatePassed>().await?.to_vec()
                        } else {
                            input.recv::<RateFailed>().await?.to_vec()
                        };
                        input.send::<RateResult>(&reported.as_slice()).await?;
                    }
                    EXPOSURE_DOSE => {
                        let sample = mode.recv::<MonitorRequest<EXPOSURE_DOSE>>().await?;
                        let args = InstrumentRequest::<EXPOSURE_DOSE>::try_from(sample.as_bytes())?;
                        meter.send::<MonitorRequest<EXPOSURE_DOSE>>(&args).await?;
                        let permit = meter.recv::<SensorPermit<EXPOSURE_DOSE>>().await?;
                        let result = effects.apply_pair(sample, permit).await?;
                        let bytes = encode(&result)?;
                        meter.send::<SampleResult>(&bytes.as_slice()).await?;
                        input
                            .send::<MonitorReading<EXPOSURE_DOSE>>(&bytes.as_slice())
                            .await?;
                        let reported = if result["success"] == true {
                            input.recv::<DosePassed>().await?.to_vec()
                        } else {
                            input.recv::<DoseFailed>().await?.to_vec()
                        };
                        input
                            .send::<MonitorReading<EXPOSURE_DOSE>>(&reported.as_slice())
                            .await?;
                    }
                    ETCH_RATE => {
                        let sample = mode.recv::<MonitorRequest<ETCH_RATE>>().await?;
                        let args = InstrumentRequest::<ETCH_RATE>::try_from(sample.as_bytes())?;
                        meter.send::<MonitorRequest<ETCH_RATE>>(&args).await?;
                        let permit = meter.recv::<SensorPermit<ETCH_RATE>>().await?;
                        let result = effects.apply_pair(sample, permit).await?;
                        let bytes = encode(&result)?;
                        meter.send::<SampleResult>(&bytes.as_slice()).await?;
                        input
                            .send::<MonitorReading<ETCH_RATE>>(&bytes.as_slice())
                            .await?;
                        let reported = if result["success"] == true {
                            input.recv::<EtchRatePassed>().await?.to_vec()
                        } else {
                            input.recv::<EtchRateFailed>().await?.to_vec()
                        };
                        input
                            .send::<MonitorReading<ETCH_RATE>>(&reported.as_slice())
                            .await?;
                    }
                    ETCH_ENDPOINT => {
                        let sample = mode.recv::<MonitorRequest<ETCH_ENDPOINT>>().await?;
                        let args = InstrumentRequest::<ETCH_ENDPOINT>::try_from(sample.as_bytes())?;
                        meter.send::<MonitorRequest<ETCH_ENDPOINT>>(&args).await?;
                        let permit = meter.recv::<SensorPermit<ETCH_ENDPOINT>>().await?;
                        let result = effects.apply_pair(sample, permit).await?;
                        let bytes = encode(&result)?;
                        meter.send::<SampleResult>(&bytes.as_slice()).await?;
                        input
                            .send::<MonitorReading<ETCH_ENDPOINT>>(&bytes.as_slice())
                            .await?;
                        let reported = if result["success"] == true {
                            input.recv::<EndpointPassed>().await?.to_vec()
                        } else {
                            input.recv::<EndpointFailed>().await?.to_vec()
                        };
                        input
                            .send::<MonitorReading<ETCH_ENDPOINT>>(&reported.as_slice())
                            .await?;
                    }
                    CrossReference::LOGICAL_LABEL => {
                        let sample = mode.recv::<CrossReference>().await?;
                        let args = InstrumentRequest::<MONITOR>::try_from(sample.as_bytes())?;
                        meter.send::<CrossReference>(&args).await?;
                        let permit = meter.recv::<SensorPermit<MONITOR>>().await?;
                        let result = effects.apply_pair(sample, permit).await?;
                        let bytes = encode(&result)?;
                        meter.send::<SampleResult>(&bytes.as_slice()).await?;
                        input
                            .send::<CrossReferenceResult>(&bytes.as_slice())
                            .await?;
                        let confirmed = if result["success"] == true {
                            input.recv::<CrossReferencePassed>().await?.to_vec()
                        } else {
                            input.recv::<CrossReferenceFailed>().await?.to_vec()
                        };
                        input
                            .send::<CrossReferenceResult>(&confirmed.as_slice())
                            .await?;
                    }
                    label => {
                        return Err(ShellError::message(format!(
                            "unexpected monitoring request {label}"
                        )));
                    }
                }
            }
        };
        async {
            let mut electronics = rv.enter(session, &p_readout)?;
            let probe = rv.enter(session, &p_probe)?;
            electronics.send::<Loan<PROBE>>(probe).await?;
            loop {
                let request = electronics.offer().await?;
                let result = match request.label() {
                    CD => {
                        effects
                            .apply(request.recv::<Dimension<CD>>().await?)
                            .await?
                    }
                    THICKNESS => {
                        effects
                            .apply(request.recv::<Dimension<THICKNESS>>().await?)
                            .await?
                    }
                    PROFILE => {
                        effects
                            .apply(request.recv::<Dimension<PROFILE>>().await?)
                            .await?
                    }
                    ALIGNMENT_CHECK => {
                        effects
                            .apply(request.recv::<Dimension<ALIGNMENT_CHECK>>().await?)
                            .await?
                    }
                    THICKNESS_VERIFICATION => {
                        effects
                            .apply(request.recv::<Dimension<THICKNESS_VERIFICATION>>().await?)
                            .await?
                    }
                    Measure::LOGICAL_LABEL => {
                        effects.apply(request.recv::<Measure>().await?).await?
                    }
                    label => {
                        return Err(ShellError::message(format!(
                            "unexpected probe request {label}"
                        )));
                    }
                };
                electronics
                    .send::<SampleResult>(&encode(&result)?.as_slice())
                    .await?;
            }
        };
        let mut metrology = rv.enter(session, &metrology)?;
        let mut probe = metrology.recv::<Loan<PROBE>>().await?;
        let mut mounted = None;
        loop {
            let request = metrology.offer().await?;
            match request.label() {
                CD => {
                    let received = request.recv::<Dimension<CD>>().await?;
                    let sample = InstrumentRequest::<CD>::try_from(received.as_bytes())?;
                    probe.send::<Dimension<CD>>(&sample).await?;
                    let result = probe.recv::<SampleResult>().await?;
                    metrology.send::<Reading<CD>>(&result).await?;
                }
                THICKNESS => {
                    let received = request.recv::<Dimension<THICKNESS>>().await?;
                    let sample = InstrumentRequest::<THICKNESS>::try_from(received.as_bytes())?;
                    probe.send::<Dimension<THICKNESS>>(&sample).await?;
                    let result = probe.recv::<SampleResult>().await?;
                    metrology.send::<Reading<THICKNESS>>(&result).await?;
                }
                PROFILE => {
                    let received = request.recv::<Dimension<PROFILE>>().await?;
                    let sample = InstrumentRequest::<PROFILE>::try_from(received.as_bytes())?;
                    probe.send::<Dimension<PROFILE>>(&sample).await?;
                    let result = probe.recv::<SampleResult>().await?;
                    metrology.send::<Reading<PROFILE>>(&result).await?;
                }
                ALIGNMENT_CHECK => {
                    let received = request.recv::<Dimension<ALIGNMENT_CHECK>>().await?;
                    let sample =
                        InstrumentRequest::<ALIGNMENT_CHECK>::try_from(received.as_bytes())?;
                    probe.send::<Dimension<ALIGNMENT_CHECK>>(&sample).await?;
                    let result = probe.recv::<SampleResult>().await?;
                    metrology.send::<Reading<ALIGNMENT_CHECK>>(&result).await?;
                }
                THICKNESS_VERIFICATION => {
                    let received = request.recv::<Dimension<THICKNESS_VERIFICATION>>().await?;
                    let sample =
                        InstrumentRequest::<THICKNESS_VERIFICATION>::try_from(received.as_bytes())?;
                    probe
                        .send::<Dimension<THICKNESS_VERIFICATION>>(&sample)
                        .await?;
                    let result = probe.recv::<SampleResult>().await?;
                    metrology
                        .send::<Reading<THICKNESS_VERIFICATION>>(&result)
                        .await?;
                }
                Loan::<STAGE>::LOGICAL_LABEL => {
                    mounted = Some(request.recv::<Loan<STAGE>>().await?);
                }
                Measure::LOGICAL_LABEL => {
                    let received = request.recv::<Measure>().await?;
                    let stage = mounted
                        .as_mut()
                        .ok_or_else(|| ShellError::message("overlay has no mounted stage"))?;
                    let sample = InstrumentRequest::<METROLOGY>::try_from(received.as_bytes())?;
                    probe.send::<Measure>(&sample).await?;
                    let result = probe.recv::<SampleResult>().await?;
                    if decode(result)?["success"] == true {
                        stage
                            .send::<OverlayPassed>(&ReviewReply::from_bytes(result)?)
                            .await?;
                        metrology
                            .send::<Loan<STAGE>>(mounted.take().expect("borrowed mounted stage"))
                            .await?;
                    } else {
                        stage
                            .send::<OverlayFailed>(&ReviewReply::from_bytes(result)?)
                            .await?;
                    }
                }
                label => {
                    return Err(ShellError::message(format!(
                        "unexpected metrology request {label}"
                    )));
                }
            }
        }
    };
    let result = hibana_runtime::race(hibana_runtime::race(trial, instruments), io.run(bus)).await;
    bus.checkpoint(SESSION, tap.checkpoint());
    io.finish(bus, result, Some("reference_wafer_integration"))
        .await
}
