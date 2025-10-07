//! Enumeration types for the badge access simulator
//!
//! This module contains all enumeration types used throughout the simulation system,
//! including room types, security levels, activity types, event types, and output formats.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Types of rooms within buildings
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RoomType {
    /// Building entrance - required for all building access
    Lobby,
    /// Individual or shared workspace
    Workspace,
    /// Conference rooms
    MeetingRoom,
    /// Restroom facilities
    Bathroom,
    /// Dining areas
    Cafeteria,
    /// Break room kitchens
    Kitchen,
    /// High-security technical areas
    ServerRoom,
    /// High-level offices
    ExecutiveOffice,
    /// Storage areas
    Storage,
    /// Research/testing areas
    Laboratory,
}

impl fmt::Display for RoomType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RoomType::Lobby => write!(f, "Lobby"),
            RoomType::Workspace => write!(f, "Workspace"),
            RoomType::MeetingRoom => write!(f, "Meeting Room"),
            RoomType::Bathroom => write!(f, "Bathroom"),
            RoomType::Cafeteria => write!(f, "Cafeteria"),
            RoomType::Kitchen => write!(f, "Kitchen"),
            RoomType::ServerRoom => write!(f, "Server Room"),
            RoomType::ExecutiveOffice => write!(f, "Executive Office"),
            RoomType::Storage => write!(f, "Storage"),
            RoomType::Laboratory => write!(f, "Laboratory"),
        }
    }
}

impl FromStr for RoomType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "lobby" => Ok(RoomType::Lobby),
            "workspace" => Ok(RoomType::Workspace),
            "meeting room" | "meetingroom" => Ok(RoomType::MeetingRoom),
            "bathroom" => Ok(RoomType::Bathroom),
            "cafeteria" => Ok(RoomType::Cafeteria),
            "kitchen" => Ok(RoomType::Kitchen),
            "server room" | "serverroom" => Ok(RoomType::ServerRoom),
            "executive office" | "executiveoffice" => Ok(RoomType::ExecutiveOffice),
            "storage" => Ok(RoomType::Storage),
            "laboratory" | "lab" => Ok(RoomType::Laboratory),
            _ => Err(format!("Unknown room type: {}", s)),
        }
    }
}

/// Security levels for rooms and access control
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SecurityLevel {
    /// Public areas accessible to all users
    Public,
    /// Standard office areas with basic access control
    Standard,
    /// Restricted areas requiring specific permissions
    Restricted,
    /// High-security areas with strict access control
    HighSecurity,
    /// Maximum security areas with very limited access
    MaxSecurity,
}

impl fmt::Display for SecurityLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SecurityLevel::Public => write!(f, "Public"),
            SecurityLevel::Standard => write!(f, "Standard"),
            SecurityLevel::Restricted => write!(f, "Restricted"),
            SecurityLevel::HighSecurity => write!(f, "High Security"),
            SecurityLevel::MaxSecurity => write!(f, "Max Security"),
        }
    }
}

impl FromStr for SecurityLevel {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "public" => Ok(SecurityLevel::Public),
            "standard" => Ok(SecurityLevel::Standard),
            "restricted" => Ok(SecurityLevel::Restricted),
            "high security" | "highsecurity" | "high" => Ok(SecurityLevel::HighSecurity),
            "max security" | "maxsecurity" | "max" | "maximum" => Ok(SecurityLevel::MaxSecurity),
            _ => Err(format!("Unknown security level: {}", s)),
        }
    }
}

/// Types of activities users perform throughout the day
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ActivityType {
    /// Coming to work
    Arrival,
    /// Going to meeting room
    Meeting,
    /// Bathroom break
    Bathroom,
    /// Lunch break
    Lunch,
    /// Visiting colleague
    Collaboration,
    /// Unauthorized access attempt (curious users)
    UnauthorizedAccess,
    /// Night-shift patrol activity
    NightPatrol,
    /// Leaving work
    Departure,
}

impl fmt::Display for ActivityType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ActivityType::Arrival => write!(f, "Arrival"),
            ActivityType::Meeting => write!(f, "Meeting"),
            ActivityType::Bathroom => write!(f, "Bathroom"),
            ActivityType::Lunch => write!(f, "Lunch"),
            ActivityType::Collaboration => write!(f, "Collaboration"),
            ActivityType::UnauthorizedAccess => write!(f, "Unauthorized Access"),
            ActivityType::NightPatrol => write!(f, "Night Patrol"),
            ActivityType::Departure => write!(f, "Departure"),
        }
    }
}

impl FromStr for ActivityType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "arrival" => Ok(ActivityType::Arrival),
            "meeting" => Ok(ActivityType::Meeting),
            "bathroom" => Ok(ActivityType::Bathroom),
            "lunch" => Ok(ActivityType::Lunch),
            "collaboration" => Ok(ActivityType::Collaboration),
            "unauthorized access" | "unauthorizedaccess" | "unauthorized" => {
                Ok(ActivityType::UnauthorizedAccess)
            }
            "night patrol" | "nightpatrol" | "patrol" => Ok(ActivityType::NightPatrol),
            "departure" => Ok(ActivityType::Departure),
            _ => Err(format!("Unknown activity type: {}", s)),
        }
    }
}

/// Types of access events that can occur
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EventType {
    /// Successful access granted
    Success,
    /// Access denied due to insufficient permissions
    Failure,
    /// Access attempt with invalid badge
    InvalidBadge,
    /// Access attempt outside of allowed hours
    OutsideHours,
    /// Suspicious access pattern detected
    Suspicious,
}

/// Types of failures that can occur during event generation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FailureType {
    /// No failure - normal event generation
    None,
    /// Curious user unauthorized access attempt
    CuriousAccess,
    /// Impossible traveler scenario
    ImpossibleTraveler,
    /// Badge reader technical failure
    BadgeReaderFailure,
}

/// Reasons for access event failures
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FailureReason {
    /// Access denied due to insufficient permissions
    Unauthorized,
    /// Curious user attempting unauthorized access
    CuriousUser,
    /// Impossible traveler scenario detected
    ImpossibleTraveler,
    /// Access attempt outside allowed hours
    OutsideHours,
    /// Badge reader technical malfunction
    BadgeReaderError,
    /// General system failure
    SystemFailure,
}

impl fmt::Display for EventType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EventType::Success => write!(f, "Success"),
            EventType::Failure => write!(f, "Failure"),
            EventType::InvalidBadge => write!(f, "Invalid Badge"),
            EventType::OutsideHours => write!(f, "Outside Hours"),
            EventType::Suspicious => write!(f, "Suspicious"),
        }
    }
}

impl FromStr for EventType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "success" => Ok(EventType::Success),
            "failure" => Ok(EventType::Failure),
            "invalid badge" | "invalidbadge" => Ok(EventType::InvalidBadge),
            "outside hours" | "outsidehours" => Ok(EventType::OutsideHours),
            "suspicious" => Ok(EventType::Suspicious),
            _ => Err(format!("Unknown event type: {}", s)),
        }
    }
}

impl fmt::Display for FailureType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FailureType::None => write!(f, "None"),
            FailureType::CuriousAccess => write!(f, "Curious Access"),
            FailureType::ImpossibleTraveler => write!(f, "Impossible Traveler"),
            FailureType::BadgeReaderFailure => write!(f, "Badge Reader Failure"),
        }
    }
}

impl FromStr for FailureType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "none" => Ok(FailureType::None),
            "curious access" | "curiousaccess" | "curious" => Ok(FailureType::CuriousAccess),
            "impossible traveler" | "impossibletraveler" => Ok(FailureType::ImpossibleTraveler),
            "badge reader failure" | "badgereaderfailure" | "badge reader" => Ok(FailureType::BadgeReaderFailure),
            _ => Err(format!("Unknown failure type: {}", s)),
        }
    }
}

impl fmt::Display for FailureReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FailureReason::Unauthorized => write!(f, "Unauthorized"),
            FailureReason::CuriousUser => write!(f, "Curious User"),
            FailureReason::ImpossibleTraveler => write!(f, "Impossible Traveler"),
            FailureReason::OutsideHours => write!(f, "Outside Hours"),
            FailureReason::BadgeReaderError => write!(f, "Badge Reader Error"),
            FailureReason::SystemFailure => write!(f, "System Failure"),
        }
    }
}

impl FromStr for FailureReason {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "unauthorized" => Ok(FailureReason::Unauthorized),
            "curious user" | "curioususer" | "curious" => Ok(FailureReason::CuriousUser),
            "impossible traveler" | "impossibletraveler" => Ok(FailureReason::ImpossibleTraveler),
            "outside hours" | "outsidehours" => Ok(FailureReason::OutsideHours),
            "badge reader error" | "badgereadererror" | "badge reader" => Ok(FailureReason::BadgeReaderError),
            "system failure" | "systemfailure" | "system" => Ok(FailureReason::SystemFailure),
            _ => Err(format!("Unknown failure reason: {}", s)),
        }
    }
}

/// Output format options for the simulation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum OutputFormat {
    /// JSON format for structured data
    Json,
    /// CSV format for tabular data
    Csv,
}

impl fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OutputFormat::Json => write!(f, "JSON"),
            OutputFormat::Csv => write!(f, "CSV"),
        }
    }
}

impl FromStr for OutputFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "json" => Ok(OutputFormat::Json),
            "csv" => Ok(OutputFormat::Csv),
            _ => Err(format!("Unknown output format: {}", s)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_room_type_display() {
        assert_eq!(format!("{}", RoomType::Lobby), "Lobby");
        assert_eq!(format!("{}", RoomType::MeetingRoom), "Meeting Room");
        assert_eq!(format!("{}", RoomType::ServerRoom), "Server Room");
        assert_eq!(format!("{}", RoomType::ExecutiveOffice), "Executive Office");
    }

    #[test]
    fn test_room_type_from_str() {
        assert_eq!("lobby".parse::<RoomType>().unwrap(), RoomType::Lobby);
        assert_eq!("meeting room".parse::<RoomType>().unwrap(), RoomType::MeetingRoom);
        assert_eq!("meetingroom".parse::<RoomType>().unwrap(), RoomType::MeetingRoom);
        assert_eq!("server room".parse::<RoomType>().unwrap(), RoomType::ServerRoom);
        assert_eq!("serverroom".parse::<RoomType>().unwrap(), RoomType::ServerRoom);
        assert_eq!("laboratory".parse::<RoomType>().unwrap(), RoomType::Laboratory);
        assert_eq!("lab".parse::<RoomType>().unwrap(), RoomType::Laboratory);

        // Test error case
        assert!("invalid".parse::<RoomType>().is_err());
    }

    #[test]
    fn test_security_level_display() {
        assert_eq!(format!("{}", SecurityLevel::Public), "Public");
        assert_eq!(format!("{}", SecurityLevel::HighSecurity), "High Security");
        assert_eq!(format!("{}", SecurityLevel::MaxSecurity), "Max Security");
    }

    #[test]
    fn test_security_level_from_str() {
        assert_eq!("public".parse::<SecurityLevel>().unwrap(), SecurityLevel::Public);
        assert_eq!("high security".parse::<SecurityLevel>().unwrap(), SecurityLevel::HighSecurity);
        assert_eq!("highsecurity".parse::<SecurityLevel>().unwrap(), SecurityLevel::HighSecurity);
        assert_eq!("high".parse::<SecurityLevel>().unwrap(), SecurityLevel::HighSecurity);
        assert_eq!("max security".parse::<SecurityLevel>().unwrap(), SecurityLevel::MaxSecurity);
        assert_eq!("maxsecurity".parse::<SecurityLevel>().unwrap(), SecurityLevel::MaxSecurity);
        assert_eq!("max".parse::<SecurityLevel>().unwrap(), SecurityLevel::MaxSecurity);
        assert_eq!("maximum".parse::<SecurityLevel>().unwrap(), SecurityLevel::MaxSecurity);

        // Test error case
        assert!("invalid".parse::<SecurityLevel>().is_err());
    }

    #[test]
    fn test_activity_type_display() {
        assert_eq!(format!("{}", ActivityType::Arrival), "Arrival");
        assert_eq!(format!("{}", ActivityType::UnauthorizedAccess), "Unauthorized Access");
        assert_eq!(format!("{}", ActivityType::Collaboration), "Collaboration");
        assert_eq!(format!("{}", ActivityType::NightPatrol), "Night Patrol");
    }

    #[test]
    fn test_activity_type_from_str() {
        assert_eq!("arrival".parse::<ActivityType>().unwrap(), ActivityType::Arrival);
        assert_eq!(
            "unauthorized access".parse::<ActivityType>().unwrap(),
            ActivityType::UnauthorizedAccess
        );
        assert_eq!(
            "unauthorizedaccess".parse::<ActivityType>().unwrap(),
            ActivityType::UnauthorizedAccess
        );
        assert_eq!(
            "unauthorized".parse::<ActivityType>().unwrap(),
            ActivityType::UnauthorizedAccess
        );
        assert_eq!("night patrol".parse::<ActivityType>().unwrap(), ActivityType::NightPatrol);
        assert_eq!("nightpatrol".parse::<ActivityType>().unwrap(), ActivityType::NightPatrol);
        assert_eq!("patrol".parse::<ActivityType>().unwrap(), ActivityType::NightPatrol);

        // Test error case
        assert!("invalid".parse::<ActivityType>().is_err());
    }

    #[test]
    fn test_event_type_display() {
        assert_eq!(format!("{}", EventType::Success), "Success");
        assert_eq!(format!("{}", EventType::InvalidBadge), "Invalid Badge");
        assert_eq!(format!("{}", EventType::OutsideHours), "Outside Hours");
    }

    #[test]
    fn test_event_type_from_str() {
        assert_eq!("success".parse::<EventType>().unwrap(), EventType::Success);
        assert_eq!("invalid badge".parse::<EventType>().unwrap(), EventType::InvalidBadge);
        assert_eq!("invalidbadge".parse::<EventType>().unwrap(), EventType::InvalidBadge);
        assert_eq!("outside hours".parse::<EventType>().unwrap(), EventType::OutsideHours);
        assert_eq!("outsidehours".parse::<EventType>().unwrap(), EventType::OutsideHours);

        // Test error case
        assert!("unknown".parse::<EventType>().is_err());
    }

    #[test]
    fn test_output_format_display() {
        assert_eq!(format!("{}", OutputFormat::Json), "JSON");
        assert_eq!(format!("{}", OutputFormat::Csv), "CSV");
    }

    #[test]
    fn test_output_format_from_str() {
        assert_eq!("json".parse::<OutputFormat>().unwrap(), OutputFormat::Json);
        assert_eq!("csv".parse::<OutputFormat>().unwrap(), OutputFormat::Csv);

        // Test error case
        assert!("invalid".parse::<OutputFormat>().is_err());
    }

    #[test]
    fn test_enum_serialization() {
        // Test that enums can be serialized and deserialized
        let room_type = RoomType::ServerRoom;
        let json = serde_json::to_string(&room_type).unwrap();
        let deserialized: RoomType = serde_json::from_str(&json).unwrap();
        assert_eq!(room_type, deserialized);

        let security_level = SecurityLevel::HighSecurity;
        let json = serde_json::to_string(&security_level).unwrap();
        let deserialized: SecurityLevel = serde_json::from_str(&json).unwrap();
        assert_eq!(security_level, deserialized);

        let activity_type = ActivityType::UnauthorizedAccess;
        let json = serde_json::to_string(&activity_type).unwrap();
        let deserialized: ActivityType = serde_json::from_str(&json).unwrap();
        assert_eq!(activity_type, deserialized);

        let night_patrol_type = ActivityType::NightPatrol;
        let json = serde_json::to_string(&night_patrol_type).unwrap();
        let deserialized: ActivityType = serde_json::from_str(&json).unwrap();
        assert_eq!(night_patrol_type, deserialized);

        let event_type = EventType::InvalidBadge;
        let json = serde_json::to_string(&event_type).unwrap();
        let deserialized: EventType = serde_json::from_str(&json).unwrap();
        assert_eq!(event_type, deserialized);

        let output_format = OutputFormat::Json;
        let json = serde_json::to_string(&output_format).unwrap();
        let deserialized: OutputFormat = serde_json::from_str(&json).unwrap();
        assert_eq!(output_format, deserialized);
    }

    #[test]
    fn test_enum_hash_and_equality() {
        use std::collections::HashSet;

        let mut room_types = HashSet::new();
        room_types.insert(RoomType::Lobby);
        room_types.insert(RoomType::ServerRoom);
        room_types.insert(RoomType::Lobby); // Duplicate

        assert_eq!(room_types.len(), 2);
        assert!(room_types.contains(&RoomType::Lobby));
        assert!(room_types.contains(&RoomType::ServerRoom));
        assert!(!room_types.contains(&RoomType::Kitchen));
    }
}
