use serde_json::Value;
use std::fs::File;
use std::io::{BufReader, BufRead};
use serde::{Serialize, Deserialize};
use chrono::Utc;

#[derive(Serialize, Deserialize, Debug)]
pub struct UserAnomaly {
    pub user_id: String,
    pub anomaly_type: String, // "cloned_badge" or "curious_access"
    pub details: String,
    pub issue: String,
    pub severity: String, // "low", "medium", "high"
    pub timestamp: String,
}

pub fn detect_cloned_badges(path: &str) -> Vec<UserAnomaly> {
    let file = File::open(path).expect("Cannot open user profile JSON");
    let reader = BufReader::new(file);
    let mut anomalies = Vec::new();

    for line in reader.lines() {
        let line = line.expect("Failed to read line");
        let user: Value = serde_json::from_str(&line).expect("Cannot parse JSON line");
        let user_id = user["user_id"].as_str().unwrap_or("unknown").to_string();

        if user["has_cloned_badge"].as_bool().unwrap_or(false) {
            anomalies.push(UserAnomaly {
                user_id,
                anomaly_type: "cloned_badge".to_string(),
                details: "User has a cloned badge assigned.".to_string(),
                issue: "Multiple active badge IDs detected".to_string(),
                severity: "high".to_string(),
                timestamp: Utc::now().to_rfc3339(),
            });
        }
    }

    anomalies
}

pub fn detect_curious_users(path: &str) -> Vec<UserAnomaly> {
    let file = File::open(path).expect("Cannot open user profile JSON");
    let reader = BufReader::new(file);
    let mut anomalies = Vec::new();

    for line in reader.lines() {
        let line = line.expect("Failed to read line");
        let user: Value = serde_json::from_str(&line).expect("Cannot parse JSON line");
        let user_id = user["user_id"].as_str().unwrap_or("unknown").to_string();

        if user["is_curious"].as_bool().unwrap_or(false) {
            anomalies.push(UserAnomaly {
                user_id,
                anomaly_type: "curious_access".to_string(),
                details: "User accessed unauthorized rooms.".to_string(),
                issue: "Unauthorized access pattern detected".to_string(),
                severity: "high".to_string(),
                timestamp: Utc::now().to_rfc3339(),
            });
        }
    }

    anomalies
}

pub fn detect_night_shift_users(path: &str) -> Vec<UserAnomaly> {
    use chrono::NaiveTime;
    let file = File::open(path).expect("Cannot open user profile JSON");
    let reader = BufReader::new(file);
    let mut anomalies = Vec::new();

    for line in reader.lines() {
        let line = line.expect("Failed to read line");
        let user: Value = serde_json::from_str(&line).expect("Cannot parse JSON line");
        let user_id = user["user_id"].as_str().unwrap_or("unknown").to_string();

        // Simulated access time field check
        if let Some(access_time) = user["last_access_time"].as_str() {
            if let Ok(time) = NaiveTime::parse_from_str(access_time, "%H:%M:%S") {
                if time < NaiveTime::from_hms_opt(6, 0, 0).unwrap()
                    || time > NaiveTime::from_hms_opt(22, 0, 0).unwrap()
                {
                    anomalies.push(UserAnomaly {
                        user_id,
                        anomaly_type: "night_shift_access".to_string(),
                        details: format!("User accessed during night hours: {}", access_time),
                        issue: "Access outside normal business hours".to_string(),
                        severity: "medium".to_string(),
                        timestamp: Utc::now().to_rfc3339(),
                    });
                }
            }
        }
    }

    anomalies
}

pub fn generate_report(output_path: &str, anomalies: &Vec<UserAnomaly>) {
    use serde_json::json;

    let summary = json!({
        "total_anomalies": anomalies.len(),
        "cloned_badge": anomalies.iter().filter(|a| a.anomaly_type == "cloned_badge").count(),
        "curious_access": anomalies.iter().filter(|a| a.anomaly_type == "curious_access").count(),
        "night_shift_access": anomalies.iter().filter(|a| a.anomaly_type == "night_shift_access").count(),
    });

    let full_report = json!({
        "generated_at": chrono::Utc::now().to_rfc3339(),
        "anomalies": anomalies,
        "summary": summary
    });

    let file = File::create(output_path).expect("Cannot create report file");
    serde_json::to_writer_pretty(&file, &full_report).expect("Cannot write JSON report");

    println!(" Report generated: {}", output_path);
    println!(
        "Summary: {} total anomalies ({} cloned badges, {} curious, {} night shift)",
        summary["total_anomalies"],
        summary["cloned_badge"],
        summary["curious_access"],
        summary["night_shift_access"]
    );
}
pub fn simulate_s3_upload(file_path: &str, bucket_name: &str) {
    use chrono::Utc;
    use std::fs::{self, OpenOptions};
    use std::io::Write;

    println!(" Simulating upload of '{}' to AWS S3 bucket '{}'", file_path, bucket_name);

    // Read the JSON content
    let mut json_text = fs::read_to_string(file_path).expect("Failed to read JSON report");

    // Append summary metadata
    let summary = format!(
        r#",
{{
    "summary": {{
        "uploaded_to_s3": true,
        "s3_bucket": "{}",
        "timestamp": "{}"
    }}
}}"#,
        bucket_name,
        Utc::now().to_rfc3339()
    );

    if let Some(pos) = json_text.rfind(']') {
        json_text.insert_str(pos + 1, &summary);
    }

    // Write it back
    let mut file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(file_path)
        .expect("Failed to open file for appending");
    file.write_all(json_text.as_bytes())
        .expect("Failed to update JSON file");

    println!(" Simulated S3 upload complete and summary added!");
}