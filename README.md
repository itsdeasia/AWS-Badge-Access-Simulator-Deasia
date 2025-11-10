<<<<<<< HEAD
# 🧠 Deasia Craig’s Enhanced AWS Badge Access Simulator

This repository contains my customized version of the **AWS Badge Access Simulator**, originally created by Amazon’s Security Engineering team.  
I expanded upon the project by completing both **security detection challenges** and implementing **additional anomaly detection logic** to strengthen event analysis.

---

## 💻 My Contributions
- **Completed both official AWS challenges:**
  - Detecting cloned badge activity (impossible traveler detection)
  - Identifying “curious users” attempting unauthorized access
- **Developed an extended `analysis.rs` module** to improve anomaly reporting and detection accuracy.
- Enhanced **`main.rs`** to include optimized logic for variance handling and security event analysis.
- Structured and cleaned the codebase with a new `.gitignore` for Rust environments.

---

## ⚙️ Technologies Used
- **Language:** Rust 🦀  
- **Frameworks/Tools:** Cargo, JSON, AWS Simulation Framework  
- **Focus Areas:** Security anomaly detection, event streaming, data analysis

---

## 📂 Repository Purpose
This simulation models real-world badge access events across multiple locations and facilities. It supports advanced testing for:
- Security analytics and behavior modeling  
- Intrusion detection systems  
- Machine learning feature engineering for user activity prediction

---

## 🧩 Collaborators & Acknowledgments
Created as part of the **AWS Security Career Pathway** activity.  
Originally authored by **Principal Security Engineer Karl Anderson** and adapted with custom logic by **Deasia Craig**.



# Amazon AWS Security Career Pathway Activity: Badge Access Simulator

This code is a Badge Access Simulator that generates days of simulated badge events to mimic real-world security badge access patterns. The system simulates thousands of users accessing facilities across multiple geographical locations with behavioral patterns, authorization violations, and security anomalies including impossible traveler scenarios.  This simulation is intended to be used to complete the AWS Security Career Pathway Activity below.

## Simulation Scenario
Users at this company are assigned badges which grant them access to certain rooms, building, and locations. Each location is approximately 4 hours from any other location. Each location has multiple buildings and each building has multiple rooms.  You are observing badge access events when each user attempts to access rooms within the building.  

# Career Activity
Create a repeatable method (program/code/process) to identify users that appear to have their badge cloned such that they appear in multiple geographic locations at the same time.  Use the --user-profiles-output option to get an answer key for each user to validate your results.

## Bonus challenge 1
Create a repeatable way to identify curious users who are trying to access rooms that they are not authorized to access

## Bonus challenge 2
Determine the type of each room based on user behavior, when they use that room, and how long they linger.


# Simulation Overview

The Badge Access Simulator is designed for security analysts and detection engineers who need realistic test data for security monitoring systems. It generates simulated daily batches of time-ordered badge events that can be processed as a batch or continuous stream.  The badge events have authentic patterns including:

- **Realistic Human Behavior**: Location affinity, daily schedules, meeting patterns
- **Authorization Violations**: Curious users attempting unauthorized access
- **Security Anomalies**: Impossible traveler scenarios from badge cloning
- **Overnight Personnel**: That patrol the building
- **Configurable Scale**: From hundreds to tens of thousands of users

## Quick Start

### Basic Usage

Run with default settings (10,000 users across 10 locations for 1 day):

```bash
#install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Build the project
cargo build --release

# Run directly
./target/release/amzn-career-pathway-activity-rust

```


### Custom Configuration

```bash
# Direct execution
./target/release/amzn-career-pathway-activity-rust \
  --user-count 5000 \
  --location-count 5 \
  --user-profile-output userprofile.json \
  --days 5
```

### Configuration File

Create a `config.json` file and use it:

```bash
# Direct execution
./target/release/amzn-career-pathway-activity-rust --config config.json

```

## Command Line Options

| Option | Description | Default |
|--------|-------------|---------|
| `--days <N>` | Number of work days to simulate | 1 |
| `--user-profiles-output <FILE>` | User Profile Answer Key | disabled |
| `--user-count <N>` | Number of users to simulate | 10000 |
| `--location-count <N>` | Number of geographical locations | 5 |
| `--curious-percentage <PCT>` | Probability of curious users | 0.05 |
| `--cloned-badge-percentage <PCT>` | Probability with cloned badges | 0.001 |
| `--config <FILE>` | Load configuration from JSON file | - |
| `--dry-run` | Validate configuration without running | false |
| `--print-config` | Print default configuration as JSON | false |
| `--verbose` | Enable verbose logging | false |
| `--debug` | Enable debug logging | false |

## Configuration

### Configuration File Format

```json
{
  "user_count": 10000,
  "location_count": 10,
  "min_buildings_per_location": 4,
  "max_buildings_per_location": 6,
  "min_rooms_per_building": 10,
  "max_rooms_per_building": 50,
  "curious_user_percentage": 0.03,
  "cloned_badge_percentage": 0.005,
  "primary_building_affinity": 0.85,
  "same_location_travel": 0.10,
  "different_location_travel": 0.05,
  "time_acceleration_factor": 288.0,
  "output_format": "json",
  "streaming": true,
  "seed": null
}
```

### Key Configuration Parameters

#### Scale Parameters
- **user_count**: Total number of users (default: 10,000)
- **location_count**: Number of geographical locations (default: 5)
- **buildings_per_location**: Range of buildings per location (default: 4-6)
- **rooms_per_building**: Range of rooms per building (default: 10-50)

#### Behavior Parameters
- **curious_user_percentage**: Users who attempt unauthorized access (default: 5%)
- **cloned_badge_percentage**: Users with cloned badges for impossible traveler scenarios (default: 0.1%)
- **primary_building_affinity**: Time spent in primary building (default: 85%)
- **same_location_travel**: Time spent in other buildings at same location (default: 10%)
- **different_location_travel**: Time spent at different locations (default: 5%)

## Output Format

### JSON Format (Default)

```json
{
  "timestamp": "2025-08-27T00:01:26.663Z",
  "user_id": "USER_90c7b5d0-bf0b-4cb2-b109-269855688aa2",
  "room_id": "ROOM_dd52709c-ed8c-4568-a4ed-6fcf77875bfb",
  "building_id": "BLD_f40723f3-e72a-48c7-94d8-34fc4719b9b1",
  "location_id": "LOC_217b2d12-6e53-4b27-84a2-f34db4fc3219",
  "success": true
}
```

## Use Cases

### Security System Testing

Test intrusion detection systems with realistic access patterns:

```bash
# Generate high-volume event stream for load testing
./target/release/amzn-career-pathway-activity-rust \
  --user-count 50000 \
  --days 30 > security_events.jsonl

```


## Architecture

The simulator consists of several key components:

- **Configuration System**: Handles all configurable parameters with smart defaults
- **Facility Generator**: Creates realistic building and room layouts
- **User Generator**: Generates users with appropriate permissions
- **Behavior Engine**: Implements realistic human behavior patterns
- **Time Manager**: Handles time sorting and realistic temporal patterns
- **Event Generator**: Creates badge access events from user activities

## Event Types

### Success Events
- Authorized access to permitted rooms
- Normal daily activities (arrival, meetings, departure)
- Legitimate travel between authorized locations

### Failure Events
- Unauthorized access attempts by curious users
- Access to rooms without proper permissions
- Attempts to access high-security areas

### Suspicious Events
- Impossible traveler scenarios (simultaneous access from distant locations)
- Rapid sequential access attempts
- Access patterns outside normal business hours

## Behavioral Patterns

### Daily Schedules
- Arrival times (8-10 AM)
- Meeting patterns (1-4 meetings per day)
- Bathroom breaks (2-3 per day)
- Lunch breaks (11:30 AM - 1:30 PM)
- Departure times (4-7 PM)

### Location Affinity
- 85% of time in primary building
- 10% of time in other buildings at same location
- 5% of time at different geographical locations

### Access Flows
- Building lobby access required before room access
- Sequential access through security checkpoints
- Realistic travel times between locations

## Troubleshooting

### Common Issues

**High Memory Usage**
- Reduce `user_count` or `location_count`
- Use streaming mode instead of batch processing

**Configuration Errors**
- Use `--dry-run` to validate configuration
- Check `--print-config` for default values

### Logging

Enable detailed logging for troubleshooting:

```bash
# Verbose logging
./target/release/amzn-career-pathway-activity-rust --verbose

# Debug logging
./target/release/amzn-career-pathway-activity-rust --debug

```

## Development

### Building

```bash
# Release build (optimized)
cargo build --release

# Debug build (faster compilation)
cargo build
```

### Testing

```bash
cargo test
```

### Examples

See the `examples/` directory for usage examples:
- `facility_generation_demo.rs`: Facility generation examples
- `user_profile_demo.rs`: User behavior examples
- `impossible_traveler_demo.rs`: Anomaly detection examples
- `streaming_demo.rs`: Event streaming examples
=======
# AWS-Badge-Access-Simulator-Deasia
My customized version of the AWS Badge Access Simulator — includes both security challenges and anomaly detection improvements. Created by Principal Security Engineer Karl Anderson
>>>>>>> 168368f842ef20e7c6c241652ede563258784180
