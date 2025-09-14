# 🎵 Spotify Reshuffle

A powerful command-line tool that combines and shuffles tracks from your Spotify playlists and liked songs into a new playlist.

[![CI](https://github.com/ncolomer/spotify-reshuffle/actions/workflows/ci.yml/badge.svg)](https://github.com/ncolomer/spotify-reshuffle/actions/workflows/ci.yml)
[![Release](https://github.com/ncolomer/spotify-reshuffle/actions/workflows/release.yml/badge.svg)](https://github.com/ncolomer/spotify-reshuffle/actions/workflows/release.yml)

## ✨ Features

- 🎵 **Combine multiple playlists** into one shuffled playlist
- ❤️ **Include your Liked Songs** in the mix
- 🧹 **Automatic deduplication** removes duplicate tracks
- 🎲 **True random shuffling** using cryptographically secure randomization
- ⚠️ **Smart filtering** removes invalid, local, and unavailable tracks
- 📝 **Flexible playlist management** - creates new or updates existing playlists
- 🔧 **Configurable cache** for authentication tokens
- 🚀 **Fast and memory-efficient** built with Rust

## 📦 Installation

### Option 1: Download Pre-built Binaries (Recommended)

Download the latest release for your platform:

- **Linux (x86_64)**: [`spotify-reshuffle-linux-amd64`](https://github.com/ncolomer/spotify-reshuffle/releases/latest)
- **Linux (ARM64/Raspberry Pi)**: [`spotify-reshuffle-linux-arm64`](https://github.com/ncolomer/spotify-reshuffle/releases/latest)

```bash
# Download and extract (Linux x86_64 example)
curl -L -o spotify-reshuffle.tar.gz https://github.com/ncolomer/spotify-reshuffle/releases/latest/download/spotify-reshuffle-linux-amd64.tar.gz
tar -xzf spotify-reshuffle.tar.gz
chmod +x spotify-reshuffle

# Move to PATH (optional)
sudo mv spotify-reshuffle /usr/local/bin/
```

### Option 2: Install with Cargo

```bash
cargo install --git https://github.com/ncolomer/spotify-reshuffle
```

### Option 3: Build from Source

```bash
git clone https://github.com/ncolomer/spotify-reshuffle
cd spotify-reshuffle
cargo build --release
```

## 🚀 Quick Start

### 1. Create a Spotify App

1. Go to [Spotify Developer Dashboard](https://developer.spotify.com/dashboard)
2. Click **"Create an app"**
3. Fill in the details and create
4. Add `http://localhost:8888/callback` as a **Redirect URI**
5. Save your **Client ID** and **Client Secret**

### 2. Set Environment Variables

```bash
export RSPOTIFY_CLIENT_ID="your_client_id_here"
export RSPOTIFY_CLIENT_SECRET="your_client_secret_here"
```

💡 **Tip**: Add these to your `~/.bashrc` or `~/.zshrc` for permanent setup.

### 3. Run the Tool

```bash
# Basic usage - combine playlists into a new playlist
spotify-reshuffle --target-playlist-name "My Shuffled Mix" --source-playlists "playlist_id_1,playlist_id_2"

# Include your liked songs too
spotify-reshuffle --target-playlist-name "Ultimate Mix" --source-playlists "playlist_id_1" --include-liked

# Use a custom cache location
spotify-reshuffle --target-playlist-name "My Mix" --include-liked --cache-path "~/.spotify-cache.json"
```

## 📋 Usage Examples

### Find Playlist IDs

You can find playlist IDs in the Spotify URL:
```
https://open.spotify.com/playlist/37i9dQZF1DXcBWIGoYBM5M
                                 ↑ This is the playlist ID
```

### Basic Examples

```bash
# Get help
spotify-reshuffle --help

# Combine two playlists
spotify-reshuffle \
  --target-playlist-name "My Party Mix" \
  --source-playlists "37i9dQZF1DXcBWIGoYBM5M,1G4dQaJc8VhG4D5aYi7iWv"

# Include your liked songs only
spotify-reshuffle \
  --target-playlist-name "My Liked Shuffled" \
  --include-liked

# Combine everything - playlists + liked songs
spotify-reshuffle \
  --target-playlist-name "Ultimate Mix" \
  --source-playlists "37i9dQZF1DXcBWIGoYBM5M,1G4dQaJc8VhG4D5aYi7iWv" \
  --include-liked
```

### Advanced Examples

```bash
# Use custom cache location
spotify-reshuffle \
  --target-playlist-name "My Mix" \
  --include-liked \
  --cache-path "/tmp/spotify-tokens.json"

# Update an existing playlist (will clear and re-populate)
spotify-reshuffle \
  --target-playlist-name "Weekly Mix" \
  --source-playlists "37i9dQZF1DXcBWIGoYBM5M"
```

### Sample Output

```
🎲 Starting Spotify Reshuffle...
📂 Retrieving tracks from 2 playlists...
   Processing playlist 1: 'Discover Weekly'
   Processing playlist 2: 'Release Radar'
❤️ Retrieving Liked Songs...
🎵 Total tracks retrieved: 1,247
🧹 After deduplication: 891 unique tracks
📝 Found existing playlist: 'My Ultimate Mix'
🧹 Clearing existing tracks...
   Clearing batch 1: 100 tracks
🎲 Tracks shuffled: 891 tracks ready
⬆️ Adding tracks to playlist...
   Adding batch 1: 100 tracks
   Adding batch 2: 100 tracks
   Adding batch 3: 100 tracks
   ...
✅ Playlist updated successfully: https://open.spotify.com/playlist/xyz
🎉 891 tracks added!
```

## ⚙️ Configuration

### Command Line Options

```
spotify-reshuffle [OPTIONS] --target-playlist-name <TARGET_PLAYLIST_NAME>

Options:
  -s, --source-playlists <SOURCE_PLAYLISTS>
          Comma-separated playlist IDs to use as sources
  
  -t, --target-playlist-name <TARGET_PLAYLIST_NAME>
          Name of the target playlist to create/update
  
      --include-liked
          Include liked songs in the shuffle
  
      --cache-path <CACHE_PATH>
          Path to the cache file for storing authentication tokens
  
  -h, --help
          Print help
  
  -V, --version
          Print version
```

### Environment Variables

| Variable | Description | Required |
|----------|-------------|----------|
| `RSPOTIFY_CLIENT_ID` | Your Spotify App Client ID | ✅ Yes |
| `RSPOTIFY_CLIENT_SECRET` | Your Spotify App Client Secret | ✅ Yes |

## 🔧 How It Works

1. **🔐 Authentication**: Initiates Spotify OAuth flow (opens browser)
2. **📥 Collection**: Retrieves tracks from specified playlists and/or liked songs
3. **✨ Validation**: Filters out invalid, local, or unavailable tracks
4. **🧹 Deduplication**: Removes duplicate tracks across all sources
5. **🎲 Shuffling**: Randomly shuffles the final track list
6. **📝 Playlist**: Creates new playlist or clears existing one
7. **⬆️ Upload**: Adds all tracks in batches of 100 (Spotify API limit)

## 🛠️ Development

### Prerequisites

- Rust 1.70+ 
- Git

### Building

```bash
git clone https://github.com/ncolomer/spotify-reshuffle
cd spotify-reshuffle
cargo build --release
```

### Running Tests

```bash
cargo test
```

### Cross-Compilation

```bash
# For Raspberry Pi (ARM64)
cross build --release --target aarch64-unknown-linux-gnu

# For x86_64 Linux
cargo build --release --target x86_64-unknown-linux-gnu
```

## 🚨 Troubleshooting

### Environment Variables Not Set
```
Error: Environment variable not found: RSPOTIFY_CLIENT_ID
```
**Solution**: Make sure you've set both `RSPOTIFY_CLIENT_ID` and `RSPOTIFY_CLIENT_SECRET`.

### Authentication Failed
```
Error: OAuth error: invalid_client
```
**Solutions**:
1. Double-check your Client ID and Secret
2. Verify redirect URI is exactly: `http://localhost:8888/callback`
3. Ensure your Spotify app settings match

### Playlist Not Found
```
Error: Playlist not found or access denied
```
**Solutions**:
1. Verify the playlist ID is correct
2. Make sure the playlist is public or owned by you
3. Check that you have the right permissions

### Network/API Issues
```
Error: Request failed with status 429
```
**Solution**: Spotify API rate limiting. Wait a few minutes and try again.

### Permission Denied
```
Error: Permission denied while creating cache file
```
**Solution**: Use `--cache-path` to specify a writable location:
```bash
spotify-reshuffle --cache-path "~/spotify-cache.json" [other options]
```

## 🤝 Contributing

Contributions are welcome! Please:

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests if applicable
5. Submit a pull request

## 📄 License

MIT License - see [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- [rSpotify](https://github.com/ramsayleung/rspotify) - Excellent Spotify Web API client
- [Tokio](https://tokio.rs/) - Async runtime for Rust
- [Clap](https://docs.rs/clap/) - Command-line argument parser

