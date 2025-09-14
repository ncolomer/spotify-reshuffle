/// Utilities for Spotify track processing and validation
pub mod tracks {
    use std::collections::HashSet;

    /// Checks if the URI is a valid Spotify track URI
    pub fn is_valid_spotify_track_uri(uri: &str) -> bool {
        // Expected format: spotify:track:TRACK_ID
        let parts: Vec<&str> = uri.split(':').collect();
        parts.len() == 3 && parts[0] == "spotify" && parts[1] == "track" && !parts[2].trim().is_empty()
    }

    /// Validates and deduplicates a list of track URIs
    pub fn validate_and_deduplicate_tracks(tracks: Vec<String>) -> Vec<String> {
        // Step 1: Filter valid tracks
        let valid_tracks: Vec<String> = tracks
            .iter()
            .filter(|uri| is_valid_spotify_track_uri(uri))
            .cloned()
            .collect();

        // Step 2: Deduplicate
        valid_tracks.into_iter().collect::<HashSet<_>>().into_iter().collect()
    }

    /// Filters out invalid track URIs from a list
    pub fn filter_valid_track_uris(tracks: &[String]) -> Vec<String> {
        tracks
            .iter()
            .filter(|uri| is_valid_spotify_track_uri(uri))
            .cloned()
            .collect()
    }

    /// Deduplicates a list of track URIs while preserving order
    pub fn deduplicate_tracks(tracks: Vec<String>) -> Vec<String> {
        let mut seen = HashSet::new();
        tracks.into_iter().filter(|track| seen.insert(track.clone())).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::tracks::*;
    use std::collections::HashSet;

    #[test]
    fn test_is_valid_spotify_track_uri_valid_cases() {
        // Valid Spotify track URIs
        let valid_uris = vec![
            "spotify:track:4iV5W9uYEdYUVa79Axb7Rh",
            "spotify:track:1234567890abcdefghijAB",
            "spotify:track:7ouMYWpwJ422jRcDASZB7P",
            "spotify:track:a1b2c3d4e5f6g7h8i9j0k1",
        ];

        for uri in valid_uris {
            assert!(
                is_valid_spotify_track_uri(uri),
                "Expected '{}' to be a valid Spotify track URI",
                uri
            );
        }
    }

    #[test]
    fn test_is_valid_spotify_track_uri_invalid_cases() {
        // Invalid Spotify track URIs
        let invalid_uris = vec![
            "",                                           // Empty string
            "spotify:track:",                             // Missing track ID
            "spotify:track",                              // Missing colon and track ID
            "invalid:track:4iV5W9uYEdYUVa79Axb7Rh",       // Wrong service
            "spotify:album:4iV5W9uYEdYUVa79Axb7Rh",       // Wrong type
            "spotify:artist:4iV5W9uYEdYUVa79Axb7Rh",      // Wrong type
            "track:4iV5W9uYEdYUVa79Axb7Rh",               // Missing spotify prefix
            "spotify:track",                              // Incomplete
            "spotify:track:4iV5W9uYEdYUVa79Axb7Rh:extra", // Extra parts
            "random string",                              // Not a URI at all
            "http://spotify.com/track/123",               // HTTP URL instead of URI
            "spotify::track::123",                        // Double colons
        ];

        for uri in invalid_uris {
            assert!(
                !is_valid_spotify_track_uri(uri),
                "Expected '{}' to be an invalid Spotify track URI",
                uri
            );
        }
    }

    #[test]
    fn test_is_valid_spotify_track_uri_edge_cases() {
        // Edge cases
        let edge_cases = vec![
            ("spotify:track: ", false),   // Space in track ID
            ("spotify:track:\n", false),  // Newline in track ID
            ("spotify:track:\t", false),  // Tab in track ID
            ("SPOTIFY:TRACK:123", false), // Uppercase (should be lowercase)
            ("spotify:Track:123", false), // Mixed case
            ("spotify:track:123", true),  // Short but valid track ID
        ];

        for (uri, expected) in edge_cases {
            assert_eq!(
                is_valid_spotify_track_uri(uri),
                expected,
                "Expected '{}' to be {} valid Spotify track URI",
                uri,
                if expected { "" } else { "in" }
            );
        }
    }

    #[test]
    fn test_validate_and_deduplicate_tracks() {
        // Test the combined validation and deduplication process
        let input_tracks = vec![
            "spotify:track:valid1".to_string(),
            "invalid:track:123".to_string(), // Invalid - wrong prefix
            "spotify:track:valid2".to_string(),
            "spotify:track:valid1".to_string(), // Duplicate
            "".to_string(),                     // Invalid - empty
            "spotify:track:valid3".to_string(),
            "spotify:album:123".to_string(),    // Invalid - wrong type
            "spotify:track:valid2".to_string(), // Another duplicate
        ];

        let result = validate_and_deduplicate_tracks(input_tracks);

        // Should have 3 unique valid tracks
        assert_eq!(result.len(), 3);

        // All remaining tracks should be valid
        for track in &result {
            assert!(is_valid_spotify_track_uri(track));
        }

        // Convert to set for easy comparison
        let result_set: HashSet<String> = result.into_iter().collect();
        let expected_set: HashSet<String> = vec![
            "spotify:track:valid1".to_string(),
            "spotify:track:valid2".to_string(),
            "spotify:track:valid3".to_string(),
        ]
        .into_iter()
        .collect();

        assert_eq!(result_set, expected_set);
    }

    #[test]
    fn test_filter_valid_track_uris() {
        let mixed_tracks = vec![
            "spotify:track:valid1".to_string(),
            "invalid:track:123".to_string(),
            "spotify:track:valid2".to_string(),
            "".to_string(),
            "spotify:album:123".to_string(),
        ];

        let valid_tracks = filter_valid_track_uris(&mixed_tracks);

        assert_eq!(valid_tracks.len(), 2);
        assert_eq!(valid_tracks[0], "spotify:track:valid1");
        assert_eq!(valid_tracks[1], "spotify:track:valid2");
    }

    #[test]
    fn test_deduplicate_tracks() {
        let tracks_with_duplicates = vec![
            "spotify:track:1".to_string(),
            "spotify:track:2".to_string(),
            "spotify:track:1".to_string(), // Duplicate
            "spotify:track:3".to_string(),
            "spotify:track:2".to_string(), // Another duplicate
            "spotify:track:4".to_string(),
        ];

        let unique_tracks = deduplicate_tracks(tracks_with_duplicates);

        assert_eq!(unique_tracks.len(), 4);
        // Should preserve order of first occurrence
        assert_eq!(unique_tracks[0], "spotify:track:1");
        assert_eq!(unique_tracks[1], "spotify:track:2");
        assert_eq!(unique_tracks[2], "spotify:track:3");
        assert_eq!(unique_tracks[3], "spotify:track:4");
    }

    #[test]
    fn test_empty_track_list_handling() {
        // Test behavior with empty track lists
        let empty_tracks: Vec<String> = vec![];

        let result = validate_and_deduplicate_tracks(empty_tracks);
        assert!(result.is_empty());

        let valid_filtered = filter_valid_track_uris(&[]);
        assert!(valid_filtered.is_empty());

        let deduplicated = deduplicate_tracks(vec![]);
        assert!(deduplicated.is_empty());
    }

    #[test]
    fn test_all_invalid_tracks() {
        // Test behavior when all tracks are invalid
        let all_invalid = vec![
            "invalid:track:123".to_string(),
            "".to_string(),
            "not-a-uri".to_string(),
            "spotify:album:123".to_string(),
        ];

        let result = validate_and_deduplicate_tracks(all_invalid.clone());
        assert!(result.is_empty());

        let valid_tracks = filter_valid_track_uris(&all_invalid);
        assert!(valid_tracks.is_empty());
    }

    #[test]
    fn test_uri_component_parsing() {
        // Test the internal logic of URI parsing more thoroughly
        let test_cases = vec![
            ("spotify:track:123", (3, "spotify", "track", "123")),
            ("spotify:album:abc", (3, "spotify", "album", "abc")),
            ("a:b:c", (3, "a", "b", "c")),
            ("single", (1, "single", "", "")),
            ("", (1, "", "", "")),
        ];

        for (uri, (expected_parts, part0, part1, part2)) in test_cases {
            let parts: Vec<&str> = uri.split(':').collect();
            assert_eq!(parts.len(), expected_parts);
            if expected_parts >= 1 {
                assert_eq!(parts[0], part0);
            }
            if expected_parts >= 2 {
                assert_eq!(parts[1], part1);
            }
            if expected_parts >= 3 {
                assert_eq!(parts[2], part2);
            }
        }
    }
}
