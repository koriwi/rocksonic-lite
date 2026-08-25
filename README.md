# rocksonic-lite

`rocksonic-lite` is a small Rust CLI that mirrors Subsonic playlists and albums to a local music library. It can:

- download original audio or request MP3 transcoding;
- store tracks as `<artist>/<album>/<track> <title>.<extension>`;
- download baseline JPEG cover art;
- remove embedded MP3 artwork;
- create M3U files for playlists; and
- use multiple download threads.

> [!WARNING]
> The tool treats its output directory as a mirror. It deletes files and directories that the current configuration does not select.

## Requirements

- A current stable Rust toolchain
- A Subsonic-compatible server with API 1.16.1 support
- An HTTPS server URL to protect the password in transit

## Build

```sh
git clone https://github.com/koriwi/rocksonic-lite.git
cd rocksonic-lite
cargo build --release
```

The binary is at `target/release/rocksonic-lite`.

## Configure

Create `music.yaml`:

```yaml
server_url: https://music.example.com
user: alice
password: change-me
cover_size: 300
create_playlist: true
threads: 4
sync:
  - playlist.2f34a8c1
  - album.97bc21de
```

| Key | Meaning | Default |
| --- | --- | --- |
| `server_url` | Server base URL, without `/rest` | Required |
| `user` | Subsonic user name | Required |
| `password` | Subsonic password | Required |
| `mp3` | Optional MP3 bitrate in kbit/s | Original format |
| `cover_size` | Cover width in pixels | `300` |
| `create_playlist` | Create an M3U file for each selected playlist | `true` |
| `threads` | Number of download threads | `4` |
| `sync` | Entries in `playlist.<id>` or `album.<id>` form | Required |

The tool creates M3U files only for `playlist.<id>` entries. It does not create them for `album.<id>` entries.

Protect the configuration file. It contains the password as plain text.

## Use

Run through Cargo:

```sh
cargo run --release -- --config ./music.yaml
```

Or run the binary:

```sh
./target/release/rocksonic-lite --config ./music.yaml
```

The tool downloads original audio when you omit `mp3` or set it to `null`. To request MP3 transcoding at 256 kbit/s, set:

```yaml
mp3: 256
```

The configuration name sets the library path. For example, `./music.yaml` creates `./music/`. The tool writes playlist M3U files to the current directory.

## Disclaimer

I wrote all code without vibe coding. For now, I used vibe coding only for this README.
