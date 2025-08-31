use anyhow::Result;
use futures_util::stream::TryStreamExt;
use log::{info, warn};
use rand::seq::SliceRandom;
use rspotify::{
    model::{Country, Market, PlayableId, PlayableItem, PlaylistId, TrackId, FullPlaylist, SearchType, SearchResult},
    prelude::*,
    scopes, AuthCodeSpotify, Config, Credentials, OAuth,
};
use std::collections::HashSet;

// ⚙️ Configuration
const SOURCE_PLAYLISTS: &[&str] = &[
    "3FRqF28glFVef3PqU4Fhoi",
    "7HPMJjw9ncQGgRUqt48pOb", 
    "4Qj9TfkDUlanrZNK6JmaJV",
];

const INCLUDE_LIKED: bool = true; // Enable/disable inclusion of Liked Songs
const PLAYLIST_NAME: &str = "🎲 Reshuffled Playlist 2"; // Name for the new shuffled playlist

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logger with custom format (no timestamp/prefix) and levels
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .filter_module("rspotify", log::LevelFilter::Warn)
        .format_timestamp(None)
        .format_level(false)
        .format_target(false)
        .init();
    
    info!("🎲 Starting Spotify Reshuffle...");
    
    // Initialize Spotify client
    let spotify = init_spotify_client().await?;
    
    // Run the reshuffle process
    reshuffle_and_create_playlist(&spotify).await?;
    
    Ok(())
}

/// Initialize the Spotify client with OAuth authentication
async fn init_spotify_client() -> Result<AuthCodeSpotify> {
    let creds = Credentials::from_env().unwrap();
    let oauth = OAuth {
        scopes: scopes!(
            "user-library-read",
            "playlist-modify-private"
        ),
        redirect_uri: "http://localhost:8888/callback".to_owned(),
        ..Default::default()
    };
    let config = Config {
        token_cached: true,
        ..Default::default()
    };
    let spotify = AuthCodeSpotify::with_config(creds, oauth, config);
    let url = spotify.get_authorize_url(true)?;
    spotify.prompt_for_token(&url).await?;

    Ok(spotify)
}

/// Checks if the URI is a valid Spotify track URI
fn is_valid_spotify_track_uri(uri: &str) -> bool {
    // Expected format: spotify:track:TRACK_ID
    let parts: Vec<&str> = uri.split(':').collect();
    parts.len() == 3 && parts[0] == "spotify" && parts[1] == "track" && !parts[2].is_empty()
}

/// Find an existing playlist by search API or create a new one
async fn find_or_create_playlist(spotify: &AuthCodeSpotify) -> Result<FullPlaylist> {
    // Use Search API to find playlist by name
    let search_result = spotify
        .search(
            PLAYLIST_NAME,
            SearchType::Playlist,
            None, // market
            None, // include_external
            Some(50), // limit
            Some(0), // offset
        )
        .await?;
    
    if let SearchResult::Playlists(playlists_page) = search_result {
        for playlist in playlists_page.items {
            if playlist.name == PLAYLIST_NAME {
                // Get current user to check ownership
                let current_user = spotify.current_user().await?;
                if playlist.owner.id == current_user.id {
                    // Get the full playlist details
                    let full_playlist = spotify.playlist(playlist.id.clone(), None, None).await?;
                    info!("📝 Found existing playlist: '{}'", full_playlist.name);
                    info!("🧹 Clearing existing tracks...");
                    clear_playlist(spotify, &full_playlist.id).await?;
                    return Ok(full_playlist);
                }
            }
        }
    }
    
    // Create new playlist
    let user = spotify.current_user().await?;
    let new_playlist = spotify
        .user_playlist_create(
            user.id,
            PLAYLIST_NAME,
            Some(false), // private
            None,        // collaborative
            Some("Automatically generated shuffled playlist"),
        )
        .await?;
    
    info!("📝 Created new playlist: '{}'", new_playlist.name);
    
    Ok(new_playlist)
}

/// Clear all tracks from a playlist
async fn clear_playlist(spotify: &AuthCodeSpotify, playlist_id: &PlaylistId<'_>) -> Result<()> {
    // Get all track IDs in the playlist to remove them
    let items: Vec<_> = spotify
        .playlist_items(playlist_id.clone(), None, None)
        .try_collect()
        .await?;
    
    if items.is_empty() {
        return Ok(());
    }
    
    // Collect all track IDs
    let mut track_ids = Vec::new();
    for item in items {
        if let Some(PlayableItem::Track(track)) = item.track {
            if let Some(id) = track.id {
                track_ids.push(PlayableId::Track(id));
            }
        }
    }
    
    if track_ids.is_empty() {
        return Ok(());
    }
    
    // Remove tracks in batches (Spotify API has limits)
    const REMOVE_BATCH_SIZE: usize = 100;
    for batch in track_ids.chunks(REMOVE_BATCH_SIZE) {
        spotify.playlist_remove_all_occurrences_of_items(
            playlist_id.clone(),
            batch.iter().cloned(),
            None
        ).await?;
    }
    
    Ok(())
}

/// Retrieves all tracks from the provided playlists
async fn get_tracks_from_playlists(
    spotify: &AuthCodeSpotify,
    playlist_ids: &[&str],
) -> Result<Vec<String>> {
    let mut tracks = Vec::new();
    let mut invalid_count = 0;

    for &playlist_id in playlist_ids {
        let playlist_id = PlaylistId::from_id(playlist_id)?;

        // Collect all items from the stream
        let items: Vec<_> = spotify
            .playlist_items(playlist_id, None, Some(Market::Country(Country::UnitedStates)))
            .try_collect()
            .await?;

        for item in items {
            if let Some(PlayableItem::Track(track)) = item.track {
                if let Some(track_id) = track.id {
                    let uri = track_id.uri();
                    if is_valid_spotify_track_uri(&uri) {
                        tracks.push(uri);
                    } else {
                        invalid_count += 1;
                        warn!("⚠️  Invalid URI ignored: {uri}");
                    }
                }
            }
        }
    }

    if invalid_count > 0 {
        warn!("⚠️ {invalid_count} invalid tracks ignored from playlists");
    }

    Ok(tracks)
}

/// Retrieves all tracks from 'Liked Songs'
async fn get_liked_tracks(spotify: &AuthCodeSpotify) -> Result<Vec<String>> {
    let mut tracks = Vec::new();
    let mut invalid_count = 0;

    // Collect all items from the stream
    let items: Vec<_> = spotify
        .current_user_saved_tracks(Some(Market::Country(Country::UnitedStates)))
        .try_collect()
        .await?;

    for item in items {
        if let Some(track_id) = item.track.id {
            let uri = track_id.uri();
            if is_valid_spotify_track_uri(&uri) {
                tracks.push(uri);
            } else {
                invalid_count += 1;
                warn!("⚠️ Invalid URI ignored (Liked Songs): {uri}");
            }
        }
    }

    if invalid_count > 0 {
        warn!("⚠️ {invalid_count} invalid tracks ignored from Liked Songs");
    }

    Ok(tracks)
}

/// Merges, deduplicates, shuffles and creates a new playlist
async fn reshuffle_and_create_playlist(spotify: &AuthCodeSpotify) -> Result<()> {
    let mut all_tracks = Vec::new();

    // Regular playlists
    if !SOURCE_PLAYLISTS.is_empty() {
        info!("📂 Retrieving tracks from {} playlists...", SOURCE_PLAYLISTS.len());
        let playlist_tracks = get_tracks_from_playlists(spotify, SOURCE_PLAYLISTS).await?;
        all_tracks.extend(playlist_tracks);
    }

    // Liked Songs
    if INCLUDE_LIKED {
        info!("❤️ Retrieving Liked Songs...");
        let liked_tracks = get_liked_tracks(spotify).await?;
        all_tracks.extend(liked_tracks);
    }

    info!("🎵 Total tracks retrieved: {}", all_tracks.len());

    // 🔄 Deduplication
    let unique_tracks: Vec<String> = all_tracks.into_iter().collect::<HashSet<_>>().into_iter().collect();
    info!("🧹 After deduplication: {} unique tracks", unique_tracks.len());

    // Final validation - filter invalid URIs that might have passed through
    let valid_tracks: Vec<String> = unique_tracks
        .iter()
        .filter(|uri| is_valid_spotify_track_uri(uri))
        .cloned()
        .collect();
        
    if valid_tracks.len() != unique_tracks.len() {
        let removed = unique_tracks.len() - valid_tracks.len();
        warn!("⚠️ {removed} invalid URIs removed during final validation");
    }

    if valid_tracks.is_empty() {
        warn!("❌ No valid tracks found!");
        return Ok(());
    }

    // 🎲 Shuffle
    let mut tracks_to_add = valid_tracks;
    tracks_to_add.shuffle(&mut rand::rng());
    info!("🎲 Tracks shuffled: {} tracks ready", tracks_to_add.len());

    // Find or create reshuffle playlist
    let playlist = find_or_create_playlist(spotify).await?;

    // Adding in batches of 100
    info!("⬆️ Adding tracks to playlist...");
    const BATCH_SIZE: usize = 100;
    
    for (batch_num, batch) in tracks_to_add.chunks(BATCH_SIZE).enumerate() {
        info!("   Adding batch {}: {} tracks", batch_num + 1, batch.len());
        
        let track_ids: Result<Vec<TrackId>, _> = batch
            .iter()
            .map(|uri| TrackId::from_uri(uri))
            .collect();
        
        let track_ids = track_ids?;
        let playable_ids: Vec<PlayableId> = track_ids
            .into_iter()
            .map(PlayableId::Track)
            .collect();
            
        spotify
            .playlist_add_items(playlist.id.clone(), playable_ids, None)
            .await?;
    }

    info!("✅ Playlist updated successfully: {}", playlist.external_urls.get("spotify").unwrap_or(&"N/A".to_string()));
    info!("🎉 {} tracks added!", tracks_to_add.len());

    Ok(())
}
