use std::fmt;

/// Validation failures produced while constructing identity-domain values.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IdentityModelError {
    EmptyField(&'static str),
    EmptyCollection(&'static str),
    DuplicateCollectionValue(&'static str),
    InvalidValidityPeriod,
    InconsistentMembership(&'static str),
    InconsistentSponsorship(&'static str),
    InconsistentAccessDecision(&'static str),
}

impl fmt::Display for IdentityModelError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyField(field) => write!(formatter, "{field} must not be empty"),
            Self::EmptyCollection(field) => {
                write!(formatter, "{field} must contain at least one value")
            }
            Self::DuplicateCollectionValue(field) => {
                write!(formatter, "{field} must not contain duplicate values")
            }
            Self::InvalidValidityPeriod => {
                formatter.write_str("validity end must be later than validity start")
            }
            Self::InconsistentMembership(reason) => {
                write!(formatter, "inconsistent organization membership: {reason}")
            }
            Self::InconsistentSponsorship(reason) => {
                write!(formatter, "inconsistent sponsorship: {reason}")
            }
            Self::InconsistentAccessDecision(reason) => {
                write!(formatter, "inconsistent access decision: {reason}")
            }
        }
    }
}

impl std::error::Error for IdentityModelError {}
