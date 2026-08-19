---
name: door-testing
description: Run cepheus-trader-door headlessly against a local server and scrape its screens, so a presentation change can be seen rather than described. Use when verifying door output, reproducing a rendering bug, or checking a screen at 40 columns.
---

# Cepheus Trader Door Testing

The door is an OpenDoors program, so it normally expects a BBS to launch it.
It can also be run directly and driven from a script, which is enough to look
at any screen at any supported width.

Everything below was run on Linux. Nothing here writes to a real installation:
the sandbox uses its own data directory, credential, and config.

## 1. Build

```console
cmake -S client -B build/client -G Ninja
cmake --build build/client
cargo build --release --manifest-path server/Cargo.toml
```

Build the server too, even when only the client changed. The wire protocol
version is checked at connect time, and an older server binary refuses a newer
client with `CT-RPC version N is no longer supported`.

**A clean local build does not mean CI is clean.** CI compiles at `-O3
-DNDEBUG` while this CMake configuration defaults to no optimisation, and
several GCC warnings are only emitted by the optimiser. Under `-Werror` those
fail the build on the runners while the local build and `ctest` stay green — a
one-element `std::vector<std::string_view>` grown by `push_back` did exactly
that, failing both MinGW jobs with `array subscript 1 is outside array bounds
of 'std::basic_string_view<char> [1]'`. Compile any translation unit you
changed with the CI flags before pushing:

```console
g++ -std=c++20 -O3 -DNDEBUG -Wall -Wextra -Wpedantic -Werror -DCT_PRODUCT_VERSION='"0.0.0"' -Iclient/include -Ibuild/client -isystem client/third_party/opendoors -c client/src/door_main.cpp -o /tmp/check.o
```

## 2. Bring up a sandbox universe

```console
./server/target/release/cepheus-trader-server --data /tmp/ct-sandbox &
echo "INITIALIZE FEDERATION" | ./build/client/cepheus-trader-admin --psk-file /tmp/ct-sandbox/admin.psk initialize-universe
./build/client/cepheus-trader-admin --psk-file /tmp/ct-sandbox/admin.psk add-bbs "Sandbox"
```

`add-bbs` prints the BBS id and PSK. `initialize-universe` reads its
confirmation phrase from stdin, so it can be piped.

Then create the door's own installation in a scratch directory. The utility
refuses to overwrite an existing one, so start from an empty directory:

```console
mkdir -p /tmp/ct-run && cd /tmp/ct-run
printf '1\n<PSK-FROM-ADD-BBS>\n' | cepheus-trader-sysop --server localhost --game-port 7323 --sysop-port 7325 init-credential
cepheus-trader-sysop --config cepheus-trader.conf set-config "Sandbox" "Test Polity" 3 3
```

That writes `cepheus-trader.conf`, `cepheus-trader.credential`, and
`cepheus-trader.identities`. Set `terminal-columns` and `terminal-rows` in the
conf to pin the geometry; `40` and `24` are the supported minimum and the width
worth testing, since 80 columns hides most layout faults.

## 3. Run the door

```console
cd /tmp/ct-run && /path/to/build/client/cepheus-trader-door -LOCAL -USERNAME "Test Captain" -G 1
```

`-LOCAL` is the way in. `-G 1` selects ANSI. The door reads
`cepheus-trader.conf` from the working directory, which is why the `cd`
matters.

Drop files and sockets are the deployed path, not the testing path:

- A `DOOR32.SYS` with communication type `0` starts, connects, and draws
  nothing to the terminal.
- `-SOCKET` cannot be given descriptor `0`. `ODInEx1.c` only takes the
  existing-handle path when the handle is non-zero, so a socket on stdin falls
  through to the serial path and exits with `Unable to access serial port`.
  Passing the socket on descriptor 3 works.
- If a socket is used, connect with `nc`, not `telnet`. The door-side socket is
  a raw stream; a Telnet client injects IAC negotiation into it. Synchronet
  handles Telnet itself and hands the door a raw passthrough socket.

## 4. Drive it

```console
tmux -u new-session -d -s door -x 40 -y 24 /tmp/ct-run/inner.sh
tmux send-keys -t door Enter
tmux capture-pane -p -t door
```

Put the invocation in a wrapper script that ends with a long `sleep`, so the
pane survives the door's exit and its last screen can still be read. Send one
key per call, capture after each, and decide the next key from what is on the
screen: the flow differs between a new captain and an existing one, so a
pre-written key sequence desynchronises.

`capture-pane -ep` keeps the SGR escapes, which is what to use when the
question is about colour roles rather than layout.

Reaching the help screens: `?` at any prompt that offers it opens context help
for that screen; from the topic prompt, `H` opens the browser, and `B` and `X`
switch the level for the rest of the visit. Expert bodies are short enough that
the pager does not pause on them, so a paged help screen is always a beginner
one.

Reaching the subsystem screen from a fresh captain: Enter through the splash,
register, accept the captain sheet, choose a starting offer, accept the ship
name and fit, accept the crew, confirm, then `N` for docked operations, `U` for
the command console, `S` for ship management, and `S` again.

## 5. Notes

- **Do not use `pkill -f cepheus`.** The pattern matches the shell running the
  command, which kills the session issuing it. Use `pgrep -f "[c]epheus…"` and
  kill by pid.
- The pager pauses whenever content exceeds the screen, so a long screen shows
  `(Enter/Sp) Continue` partway down. That is the presentation pager, not the
  screen's own paging.
- A screen can also be rendered without the server at all, by linking
  `libct-door-presentation.a` and calling the presentation API directly. That
  is the faster way to compare layouts, and the only way to render a case the
  catalog cannot produce; use the live door to confirm the production path.

## Keep this skill current

Edit this file whenever a session using it learns something it does not say.
Do this without being asked:

- A step that no longer works, or that names a flag, path, port, or file that
  has since changed. Correct it in place rather than working around it.
- A faster or more reliable route than the one described.
- A trap that cost time and is not listed under the notes. Record what the
  failure looked like, not only the fix, since the symptom is what a later
  reader will be searching for.

Record only what was actually run in that session. A procedure written from
expectation rather than execution is worse than an omission, because the next
reader has no way to tell the two apart. If a step could not be verified, say
so in the text.

Include the edit in whatever change the session is already making. When the
session commits nothing, say plainly that the skill was edited and left
uncommitted, so the correction is not lost.
