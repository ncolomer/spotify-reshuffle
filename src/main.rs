use anyhow::Result;
use clap::{error::ErrorKind, CommandFactory, Parser};
use futures_util::stream::TryStreamExt;
use log::{info, warn};
use rand::seq::SliceRandom;
use rspotify::{
    model::{Country, FullPlaylist, Market, PlayableId, PlayableItem, PlaylistId, SearchResult, SearchType, TrackId},
    prelude::*,
    scopes, AuthCodeSpotify, Config, Credentials, OAuth,
};
use spotify_reshuffle::tracks::{filter_valid_track_uris, is_valid_spotify_track_uri};
use std::collections::HashSet;
use std::path::PathBuf;

/// Spotify Reshuffle CLI tool
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Comma-separated playlist IDs to use as sources
    #[arg(short, long, value_delimiter = ',', default_values = &[] as &[&str])]
    source_playlists: Vec<String>,

    /// Name of the target playlist to create/update
    #[arg(short, long)]
    target_playlist_name: String,

    /// Include liked songs in the shuffle
    #[arg(long)]
    include_liked: bool,

    /// Path to the cache file for storing authentication tokens
    #[arg(long, help = "Path to the cache file for storing authentication tokens")]
    cache_path: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Validate that at least one source is provided
    if args.source_playlists.is_empty() && !args.include_liked {
        Args::command()
            .error(
                ErrorKind::MissingRequiredArgument,
                "You must provide at least one --source-playlists, or use --include-liked",
            )
            .exit();
    }

    // Validate the target playlist is non-empty
    if args.target_playlist_name.trim().is_empty() {
        Args::command()
            .error(ErrorKind::InvalidValue, "Playlist name cannot be empty")
            .exit();
    }

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
    let spotify = init_spotify_client(args.cache_path.as_deref()).await?;

    // Run the reshuffle process
    reshuffle_and_create_playlist(&spotify, &args).await?;

    Ok(())
}

/// Initialize the Spotify client with OAuth authentication
async fn init_spotify_client(cache_path: Option<&str>) -> Result<AuthCodeSpotify> {
    let creds = Credentials::from_env().unwrap();
    let oauth = OAuth {
        scopes: scopes!("user-library-read", "playlist-modify-private"),
        redirect_uri: "http://localhost:8888/callback".to_owned(),
        ..Default::default()
    };

    let config = match cache_path {
        Some(path) => Config {
            token_cached: true,
            cache_path: PathBuf::from(path),
            ..Default::default()
        },
        None => Config {
            token_cached: true,
            ..Default::default()
        },
    };

    let spotify = AuthCodeSpotify::with_config(creds, oauth, config);
    let url = spotify.get_authorize_url(true)?;
    spotify.prompt_for_token(&url).await?;

    Ok(spotify)
}

/// Find an existing playlist by search API or create a new one
async fn find_or_create_playlist(spotify: &AuthCodeSpotify, playlist_name: &str) -> Result<FullPlaylist> {
    // Use Search API to find playlist by name
    let search_result = spotify
        .search(
            playlist_name,
            SearchType::Playlist,
            None,     // market
            None,     // include_external
            Some(50), // limit
            Some(0),  // offset
        )
        .await?;

    if let SearchResult::Playlists(playlists_page) = search_result {
        for playlist in playlists_page.items {
            if playlist.name == playlist_name {
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
            playlist_name,
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
    for (batch_num, batch) in track_ids.chunks(REMOVE_BATCH_SIZE).enumerate() {
        info!("   Clearing batch {}: {} tracks", batch_num + 1, batch.len());

        spotify
            .playlist_remove_all_occurrences_of_items(playlist_id.clone(), batch.iter().cloned(), None)
            .await?;
    }

    Ok(())
}

/// Retrieves all tracks from the provided playlists
async fn get_tracks_from_playlists(spotify: &AuthCodeSpotify, playlist_ids: &[&str]) -> Result<Vec<String>> {
    let mut tracks = Vec::new();
    let mut invalid_count = 0;

    for (playlist_num, &playlist_id) in playlist_ids.iter().enumerate() {
        let playlist_id = PlaylistId::from_id(playlist_id)?;

        // Get playlist info for logging
        let playlist_info = spotify.playlist(playlist_id.clone(), None, None).await?;
        info!("   Processing playlist {}: '{}'", playlist_num + 1, playlist_info.name);

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
async fn reshuffle_and_create_playlist(spotify: &AuthCodeSpotify, args: &Args) -> Result<()> {
    let mut all_tracks = Vec::new();

    // Regular playlists
    if !args.source_playlists.is_empty() {
        info!("📂 Retrieving tracks from {} playlists...", args.source_playlists.len());
        let source_playlist_refs: Vec<&str> = args.source_playlists.iter().map(|s| s.as_str()).collect();
        let playlist_tracks = get_tracks_from_playlists(spotify, &source_playlist_refs).await?;
        all_tracks.extend(playlist_tracks);
    }

    // Liked Songs
    if args.include_liked {
        info!("❤️ Retrieving Liked Songs...");
        let liked_tracks = get_liked_tracks(spotify).await?;
        all_tracks.extend(liked_tracks);
    }

    let total_tracks = all_tracks.len();
    info!("🎵 Total tracks retrieved: {}", total_tracks);

    // 🔄 Deduplication
    let unique_tracks: Vec<String> = all_tracks.into_iter().collect::<HashSet<_>>().into_iter().collect();
    let after_dedup = unique_tracks.len();
    info!("🧹 After deduplication: {} unique tracks", after_dedup);

    // Final validation using library function
    let valid_tracks = filter_valid_track_uris(&unique_tracks);
    let after_validation = valid_tracks.len();

    if after_validation != after_dedup {
        let removed = after_dedup - after_validation;
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
    let playlist = find_or_create_playlist(spotify, &args.target_playlist_name).await?;

    // Adding in batches of 100
    info!("⬆️ Adding tracks to playlist...");
    const BATCH_SIZE: usize = 100;

    for (batch_num, batch) in tracks_to_add.chunks(BATCH_SIZE).enumerate() {
        info!("   Adding batch {}: {} tracks", batch_num + 1, batch.len());

        let track_ids: Result<Vec<TrackId>, _> = batch.iter().map(|uri| TrackId::from_uri(uri)).collect();

        let track_ids = track_ids?;
        let playable_ids: Vec<PlayableId> = track_ids.into_iter().map(PlayableId::Track).collect();

        spotify
            .playlist_add_items(playlist.id.clone(), playable_ids, None)
            .await?;
    }

    info!(
        "✅ Playlist updated successfully: {}",
        playlist.external_urls.get("spotify").unwrap_or(&"N/A".to_string())
    );
    info!("🎉 {} tracks added!", tracks_to_add.len());

    Ok(())
}
