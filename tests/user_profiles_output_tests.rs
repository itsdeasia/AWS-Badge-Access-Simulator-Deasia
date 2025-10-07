use amzn_career_pathway_activity_rust::user::{User, UserProfile};
use amzn_career_pathway_activity_rust::permissions::PermissionSet;
use amzn_career_pathway_activity_rust::types::{BuildingId, LocationId, RoomId, SimulationConfig};
use std::fs::File;
use std::io::{BufWriter, Write};
use tempfile::NamedTempFile;

/// Generate user profiles output file in JSONL format
/// 
/// This creates the "answer key" file containing ground truth information
/// about each user's permissions, behavior, and characteristics.
fn generate_user_profiles_output(
    config: &SimulationConfig,
    users: &[User],
    output_path: &str,
) -> Result<(), String> {
    // Create output file
    let file = File::create(output_path)
        .map_err(|e| format!("Failed to create user profiles output file '{}': {}", output_path, e))?;
    let mut writer = BufWriter::new(file);

    // Generate user profile for each user and write as JSONL
    for user in users {
        let user_profile = UserProfile::from_user(user, config);
        
        // Serialize to JSON
        let json_line = serde_json::to_string(&user_profile)
            .map_err(|e| format!("Failed to serialize user profile for user {}: {}", user.id, e))?;
        
        // Write JSON line
        writeln!(writer, "{}", json_line)
            .map_err(|e| format!("Failed to write user profile line: {}", e))?;
    }

    // Ensure all data is written
    writer.flush()
        .map_err(|e| format!("Failed to flush user profiles output: {}", e))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_generate_user_profiles_output() {
        // Create test data
        let config = SimulationConfig::default();
        let location_id = LocationId::new();
        let building_id = BuildingId::new();
        let room_id = RoomId::new();
        let permissions = PermissionSet::new();

        let user = User::new(location_id, building_id, room_id, permissions);
        let users = vec![user];

        // Create temporary output file
        let temp_file = NamedTempFile::new().unwrap();
        let output_path = temp_file.path().to_str().unwrap();

        // Test the function
        let result = generate_user_profiles_output(&config, &users, output_path);
        assert!(result.is_ok());

        // Verify file was created and contains data
        let content = fs::read_to_string(output_path).unwrap();
        assert!(!content.is_empty());
        assert!(content.contains("user_id"));
        assert!(content.contains("primary_location"));
        assert!(content.contains("authorized_rooms"));

        // Verify it's valid JSONL (each line should be valid JSON)
        for line in content.lines() {
            if !line.trim().is_empty() {
                let _: serde_json::Value = serde_json::from_str(line).unwrap();
            }
        }
    }

    #[test]
    fn test_user_profile_serialization() {
        // Create test data
        let config = SimulationConfig::default();
        let location_id = LocationId::new();
        let building_id = BuildingId::new();
        let room_id = RoomId::new();
        let permissions = PermissionSet::new();

        let user = User::new(location_id, building_id, room_id, permissions);
        let user_profile = UserProfile::from_user(&user, &config);

        // Test serialization
        let json = serde_json::to_string(&user_profile).unwrap();
        assert!(!json.is_empty());

        // Test deserialization
        let _deserialized: UserProfile = serde_json::from_str(&json).unwrap();
    }

    #[test]
    fn test_multiple_users_output() {
        // Create test data with multiple users
        let config = SimulationConfig::default();
        let mut users = Vec::new();

        for _ in 0..5 {
            let location_id = LocationId::new();
            let building_id = BuildingId::new();
            let room_id = RoomId::new();
            let permissions = PermissionSet::new();
            let user = User::new(location_id, building_id, room_id, permissions);
            users.push(user);
        }

        // Create temporary output file
        let temp_file = NamedTempFile::new().unwrap();
        let output_path = temp_file.path().to_str().unwrap();

        // Test the function
        let result = generate_user_profiles_output(&config, &users, output_path);
        assert!(result.is_ok());

        // Verify file was created and contains data for all users
        let content = fs::read_to_string(output_path).unwrap();
        let lines: Vec<&str> = content.lines().filter(|line| !line.trim().is_empty()).collect();
        assert_eq!(lines.len(), 5);

        // Verify each line is valid JSON
        for line in lines {
            let profile: UserProfile = serde_json::from_str(line).unwrap();
            // Verify basic structure
            assert!(!profile.user_id.to_string().is_empty());
            assert!(!profile.primary_location.to_string().is_empty());
        }
    }
}
