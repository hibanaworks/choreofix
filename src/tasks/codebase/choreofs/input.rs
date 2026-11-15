//! Original benchmark objects, independent of execution progress.
pub struct Assignment {
    pub agent: &'static str,
    pub feature: &'static str,
    pub modules: [&'static str; 2],
}

pub const EASY_INPUT: [Assignment; 3] = [
    Assignment {
        agent: "DEVELOPER_A",
        feature: "user_authentication",
        modules: ["AUTH_MODULE", "DATABASE_MODULE"],
    },
    Assignment {
        agent: "DEVELOPER_B",
        feature: "rest_api_endpoints",
        modules: ["API_MODULE", "DATABASE_MODULE"],
    },
    Assignment {
        agent: "DEVELOPER_C",
        feature: "api_auth_middleware",
        modules: ["API_MODULE", "AUTH_MODULE"],
    },
];
pub const MEDIUM_INPUT: [Assignment; 5] = [
    Assignment {
        agent: "DEVELOPER_A",
        feature: "user_authentication",
        modules: ["AUTH_MODULE", "DATABASE_MODULE"],
    },
    Assignment {
        agent: "DEVELOPER_B",
        feature: "rest_api_endpoints",
        modules: ["API_MODULE", "DATABASE_MODULE"],
    },
    Assignment {
        agent: "DEVELOPER_C",
        feature: "payment_system",
        modules: ["API_MODULE", "PAYMENT_MODULE"],
    },
    Assignment {
        agent: "DEVELOPER_D",
        feature: "notification_service",
        modules: ["NOTIFY_MODULE", "PAYMENT_MODULE"],
    },
    Assignment {
        agent: "DEVELOPER_E",
        feature: "admin_dashboard",
        modules: ["AUTH_MODULE", "NOTIFY_MODULE"],
    },
];
pub const HARD_INPUT: [Assignment; 7] = [
    Assignment {
        agent: "DEVELOPER_A",
        feature: "user_authentication",
        modules: ["AUTH_MODULE", "DATABASE_MODULE"],
    },
    Assignment {
        agent: "DEVELOPER_B",
        feature: "rest_api_endpoints",
        modules: ["API_MODULE", "DATABASE_MODULE"],
    },
    Assignment {
        agent: "DEVELOPER_C",
        feature: "payment_system",
        modules: ["API_MODULE", "PAYMENT_MODULE"],
    },
    Assignment {
        agent: "DEVELOPER_D",
        feature: "notification_service",
        modules: ["NOTIFY_MODULE", "PAYMENT_MODULE"],
    },
    Assignment {
        agent: "DEVELOPER_E",
        feature: "data_analytics",
        modules: ["ANALYTICS_MODULE", "NOTIFY_MODULE"],
    },
    Assignment {
        agent: "DEVELOPER_F",
        feature: "search_engine",
        modules: ["ANALYTICS_MODULE", "SEARCH_MODULE"],
    },
    Assignment {
        agent: "DEVELOPER_G",
        feature: "admin_panel",
        modules: ["AUTH_MODULE", "SEARCH_MODULE"],
    },
];

pub const fn assignment(role: u8) -> &'static Assignment {
    match role {
        0..=2 => &EASY_INPUT[role as usize],
        3..=7 => &MEDIUM_INPUT[(role - 3) as usize],
        8..=14 => &HARD_INPUT[(role - 8) as usize],
        _ => panic!("unknown codebase participant"),
    }
}

/// The latest unmodified test response is the resolver's input data.
#[derive(Default)]
pub(crate) struct TestResult(std::cell::Cell<Option<serde_json::Value>>);

impl TestResult {
    pub(crate) fn record(&self, result: serde_json::Value) {
        self.0.replace(Some(result));
    }
    pub(crate) fn decide(
        &self,
    ) -> Result<hibana::runtime::resolver::DecisionArm, hibana::runtime::resolver::ResolverError>
    {
        use hibana::runtime::resolver::{DecisionArm, ResolverError};
        let result = self.0.take();
        let decision = match result.as_ref().and_then(|value| value["success"].as_bool()) {
            Some(true) => Ok(DecisionArm::Left),
            Some(false) => Ok(DecisionArm::Right),
            None => Err(ResolverError::reject()),
        };
        self.0.set(result);
        decision
    }
}
