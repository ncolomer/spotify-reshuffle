# Spotify Reshuffle (Rust)

A Rust application that reshuffles your Spotify playlists and liked songs into a new shuffled playlist.

## Features

- 🎵 Combine tracks from multiple playlists
- ❤️ Include your Liked Songs
- 🧹 Automatic deduplication
- 🎲 Random shuffling
- ⚠️ Filters out invalid/local tracks
- 📝 Creates a new playlist with all shuffled tracks

## Prerequisites

- Rust 1.70+ installed
- Spotify Developer Account
- Spotify App credentials (Client ID and Client Secret)

## Setup

1. **Clone/Download** this repository

2. **Create a Spotify App**:
   - Go to [Spotify Developer Dashboard](https://developer.spotify.com/dashboard)
   - Create a new app
   - Add `http://127.0.0.1:8888/callback` as a redirect URI
   - Note your Client ID and Client Secret

3. **Set Environment Variables**:
   ```bash
   export SPOTIPY_CLIENT_ID="your_client_id_here"
   export SPOTIPY_CLIENT_SECRET="your_client_secret_here"
   ```

4. **Configure Playlists** (optional):
   - Edit `src/main.rs` and update the `SOURCE_PLAYLISTS` constant with your playlist IDs
   - You can find playlist IDs in the Spotify URL: `https://open.spotify.com/playlist/PLAYLIST_ID`

## Usage

### Build and Run

```bash
# Build the project
cargo build --release

# Run the application
cargo run --release
```

### What It Does

1. **Authentication**: Opens a browser for Spotify OAuth
2. **Track Collection**: Retrieves tracks from configured playlists and liked songs
3. **Validation**: Filters out local tracks (those starting with `spotify:local:`)
4. **Deduplication**: Removes duplicate tracks
5. **Shuffling**: Randomly shuffles all tracks
6. **Playlist Creation**: Creates a new playlist called "🎲 Reshuffled Playlist"
7. **Track Addition**: Adds all tracks in batches of 100

### Sample Output

```
📂 Retrieving tracks from 3 playlists...
⚠️  Invalid URI ignored: spotify:local:Artist::Local+Track:300
❤️  Retrieving Liked Songs...
🎵 Total tracks retrieved: 1311
🧹 After deduplication: 1054 unique tracks
🎲 Tracks shuffled: 1054 tracks ready
📝 Playlist created: '🎲 Reshuffled Playlist'
⬆️  Adding tracks to playlist...
   Adding batch 1: 100 tracks
   Adding batch 2: 100 tracks
   ...
✅ Playlist created successfully: https://open.spotify.com/playlist/...
🎉 1054 tracks added!
```

## Configuration

You can modify these constants in `src/main.rs`:

```rust
// Playlist IDs to include (without spotify:playlist: prefix)
const SOURCE_PLAYLISTS: &[&str] = &[
    "3FRqF28glFVef3PqU4Fhoi",
    "7HPMJjw9ncQGgRUqt48pOb", 
    "4Qj9TfkDUlanrZNK6JmaJV",
];

// Whether to include Liked Songs
const INCLUDE_LIKED: bool = true;

// Name of the created playlist
const PLAYLIST_NAME: &str = "🎲 Reshuffled Playlist";
```

## Dependencies

- [rspotify](https://github.com/ramsayleung/rspotify) - Spotify Web API SDK for Rust
- [tokio](https://tokio.rs/) - Async runtime
- [rand](https://docs.rs/rand/) - Random number generation for shuffling
- [anyhow](https://docs.rs/anyhow/) - Error handling
- [log](https://docs.rs/log/) - Logging

## Differences from Python Version

- **Better Error Handling**: Uses Rust's `Result` type for robust error handling
- **Memory Efficiency**: More efficient memory usage with Rust's ownership system
- **Type Safety**: Compile-time guarantees about data types and memory safety
- **Performance**: Generally faster execution due to Rust's zero-cost abstractions
- **Async/Await**: Fully async implementation for better I/O performance

## Troubleshooting

### Environment Variables Not Set
```
Error: SPOTIPY_CLIENT_ID environment variable not set
```
Make sure you've exported both `SPOTIPY_CLIENT_ID` and `SPOTIPY_CLIENT_SECRET`.

### Authentication Issues
If you have issues with authentication, try:
1. Clearing any cached tokens (delete `.cache` files)
2. Making sure the redirect URI matches exactly: `http://127.0.0.1:8888/callback`
3. Checking that your Spotify app has the correct redirect URI configured

### Build Issues
If you encounter build issues:
```bash
# Clean build artifacts and rebuild
cargo clean
cargo build --release
```

## License

MIT License

