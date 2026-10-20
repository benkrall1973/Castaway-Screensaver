# Johnny Castaway original 16-bit program forensic record

**Status:** active private research record  
**Scope:** analyze legally obtained original program files for interoperability,
behavioral preservation, and regression testing. This record must not include
Sierra/Dynamix artwork, sound, or original executable payloads.

## Evidence set located

| File | Size | Role / current assessment |
|---|---:|---|
| `LOADER.EXE` | 30,223 bytes | Original 16-bit installation/launch component; pending NE-header and import analysis. |
| `SETUP.EXE` | 47,616 bytes | Original 16-bit setup component; pending analysis. |
| `SCRANTIC.SC$` | 158,536 bytes | Compressed/renamed installer payload for the original screensaver; must be expanded before code analysis. |
| `RESOURCE.00$` | 1,072,332 bytes | Compressed/renamed original SCI resource volume; content is not to be published. |
| `RESOURCE.001` | 35 bytes | Placeholder/small companion volume in this archived installer state. |
| `RESOURCE.MAP` | 1,453 bytes | SCI resource map; supports private indexing and correlation. |
| `INSTALL.EX$` | 102,948 bytes | Compressed installer executable payload. |
| `INSTALL.INS` | 3,006 bytes | Installer instructions/control data. |

## Primary installed Win16 screensaver

An installed `SCRANTIC.SCR` supplied from Windows 3.1 is now the primary
binary reference. It is retained with this **private** repository for personal
preservation and analysis. It is not included in the report, public project,
or any release package.

A personal evidence copy is retained at the private project root as
`SCRANTIC.SCR` and intentionally tracked only by this private repository. It
must never be copied into a public repository, release archive, or source
distribution.

| Attribute | Verified value |
|---|---|
| File size | 295,952 bytes |
| Timestamp carried by file | 1992-12-09 14:08:52 |
| SHA-256 | `7811B526B36BBA2F08326D7A680F2F063190B4278E5A36393FDE5A02080B0262` |
| Container | DOS `MZ` stub + Windows 3.x New Executable (`NE`) module |
| NE header offset | `0x250` |
| NE linker version | 5.10 |
| Segment count | 14 |
| Imported modules | `MMSYSTEM`, `GDI`, `KERNEL`, `USER` |
| Module name | `SCRNATIC` (spelling as embedded) |
| User-facing title | `SCRNSAVE :Screen Antics` |

### Confirmed exported entry points

The binary exports these Windows screensaver/configuration routines by name:

- `ScreenSaverProc`
- `ScreenSaverConfigureDialog`
- `PasswordDialog`
- `ChangePasswordDialog`
- `AdsControlDialog`

The export table yields the following exact NE ordinal/entry mapping:

| Ordinal | Export | NE code address |
|---:|---|---|
| 1 | `ScreenSaverProc` | segment 2, offset `0x0FD9` |
| 2 | `ScreenSaverConfigureDialog` | segment 2, offset `0x034F` |
| 3 | `PasswordDialog` | segment 5, offset `0x026E` |
| 4 | `ChangePasswordDialog` | segment 2, offset `0x0A2D` |
| 5 | `AdsControlDialog` | segment 5, offset `0x082A` |

### Initial control-flow findings

`ScreenSaverProc` is a conventional Win16 window procedure. Its entry performs
a message dispatch over at least these confirmed Windows messages:

| Message | Interpretation | Branch target |
|---:|---|---|
| `0x0001` | `WM_CREATE` | `0x1035` |
| `0x0002` | `WM_DESTROY` | `0x1239` |
| `0x0005` | `WM_SIZE` | `0x104E` |
| `0x000F` | `WM_PAINT` | `0x1081` |
| `0x0014` | `WM_ERASEBKGND` | `0x10F0` |
| `0x0112` | `WM_SYSCOMMAND` | `0x12A2` |
| `0x0113` | `WM_TIMER` | `0x1266` |
| `0x0400` | private application message | `0x1172` |

This already separates rendering (`WM_PAINT`) from timing (`WM_TIMER`) and
proves the scheduler is event-driven rather than a single blocking animation
loop. The next pass labels the timer IDs and the internal calls made from the
`WM_TIMER` branch.

### Timer/frame-pump model (now confirmed)

The `WM_TIMER` branch begins at segment 2:`0x1266`. Its behavior is not a
large `switch` on several independent Windows timer IDs. Instead it:

1. Rejects re-entry while a global busy flag (`DS:0x2D86`) is set.
2. Saves the current screensaver window handle.
3. Calls a mutable far callback held at `DS:0x2D91:0x2D93` when one is
   installed.
4. Performs follow-up message/paint work and returns through the default
   window procedure path.

In pseudocode:

```text
on_WM_TIMER(hwnd):
    if timer_dispatch_busy:
        return default_window_proc(...)
    timer_dispatch_busy = true
    saved_screensaver_window = hwnd
    if active_animation_tick != null:
        active_animation_tick()
    perform_pending_frame/message_work()
    timer_dispatch_busy = false
    return default_window_proc(...)
```

This is a major compatibility result. The original uses a **single guarded
frame pump plus a replaceable active animation callback**. The clock,
interstitial/special sequence, and ordinary island sequence should therefore
be modeled as state/callback handoffs, not as separate screensaver processes
or unrelated top-level timers. Exact callback writers and per-frame intervals
remain to be identified.

### Confirmed frame interval and canvas geometry

The `WM_CREATE` branch of `ScreenSaverProc` invokes the Win16 `SetTimer`
signature with:

```text
SetTimer(hwnd, 1, 0x0032, NULL)
```

That is one timer with ID **1** and an interval of **50 ms**—a nominal
**20 ticks per second**. `WM_DESTROY` correspondingly invokes `KillTimer` for
the same window/ID before running the active cleanup callback.

The original paint path explicitly sets and uses a **640 × 480** logical
rendering surface (`0x280 × 0x1E0`) and computes half-differences between the
client size and that surface when painting. This confirms a 4:3 native canvas
and centered presentation logic. Stretching the artwork to arbitrary window
dimensions is not original behavior.

`ScreenSaverConfigureDialog` is a normal dialog procedure responding to
`WM_INITDIALOG` (`0x0110`), `WM_COMMAND` (`0x0111`), and `WM_DESTROY`
(`0x002B`). `AdsControlDialog` currently resolves to a no-op exported stub in
this build; that is a binary fact, not a conclusion that the `.ADS` scheduler
is unused.

### Confirmed original configuration surface

The executable itself contains the following `SCRANTIC.INI` keys:

`SourceDir`, `Background`, `Clouds`, `Waves`, `Sounds`, `Password`,
`NumDays`, `Introduction`, `CurrentYear`, `CurrentMonth`, `CurrentDay`, and
`StartTime`.

It also names original supporting assets directly: `JOHNCAST.PAL`, `INTRO.SCR`,
`OCEAN00.SCR`, `OCEAN01.SCR`, `OCEAN02.SCR`, `NIGHT.SCR`, `HOLIDAY.BMP`,
`BACKGRND.BMP`, `MRAFT.BMP`, `JOHNWALK.BMP`, `RESOURCE.MAP`, and the ten
`.ADS` scene directors. The embedded `WAVESFX1` through `WAVESFX25` strings
confirm a dedicated original wave-effect sound set.

### Verified installer transformation

`INSTALL.INS` explicitly directs the installer to perform these transformations:

| Installer input | Installed output |
|---|---|
| `SCRANTIC.SC$` | `SCRANTIC.SCR` |
| `RESOURCE.00$` | `RESOURCE.001` |
| `RESOURCE.MAP` | `RESOURCE.MAP` |

Therefore `SCRANTIC.SCR`, not `LOADER.EXE` or `SETUP.EXE`, is the target for
the actual Win16 screensaver disassembly. `LOADER.EXE` and `SETUP.EXE` have
been confirmed as DOS `MZ` executables; they are installer/launcher support
components rather than the screensaver's application code.

Further classification refined that result: `SETUP.EXE` is an `MZ` wrapper
containing a Win16 `NE` header at file offset `0x400` (NE version `5.1`). Its
embedded imports include `KERNEL` and `USER`, and its strings identify it as
an InstallShield launcher using PKWARE's 1990–91 compression library. It is
the most promising private reference for identifying the `SC$` expansion
method. `LOADER.EXE` is a separate DOS-side loader and contains no Win16 `NE`
header at its DOS-header offset.

### Preservation hashes

| File | SHA-256 |
|---|---|
| `LOADER.EXE` | `5AD7DBA88653F4F9ED1EA4A401AF959B53AE7F82D1D6C35EEF7CD33007B4B0ED` |
| `SETUP.EXE` | `7FCCED8C961F3C137CD33AD2479BC0AC265DDE5169EB539FE988EDE7075D94AD` |
| `SCRANTIC.SC$` | `BB4CB355A8C0E3BCCCB032038817AAAA4F397E662B103136949748565432FA34` |
| `RESOURCE.00$` | `108346238741F2436BBE81EE5028786BC6FF73E151A2322331A55CBC8B95446C` |

The `$` extensions are installer-era compressed/renamed forms. They are not
evidence that the source is missing; they are evidence that the original
installer must be reconstructed or safely expanded before its executable can
be disassembled.

### Provenance caution

The archived package contains a third-party release note and altered installer
metadata. It is useful evidence for file layout, but it is not a clean,
authoritative primary source. Findings must be cross-checked against a
lawfully obtained installed `SCRANTIC.SCR` and resource folder before they are
treated as definitive compatibility requirements.

## Resource archive facts already recovered

Private resource indexing found **179 entries**: 1 palette, 116 bitmap resources,
10 screen resources, 41 animation/director (`TTM`) resources, and 10 scene
director (`ADS`) resources. The resource names and embedded labels provide a
strong behavioral map even before CPU-level analysis.

Examples confirmed from embedded labels:

- `THEEND.TTM`: the end sequence, plane drop, dance, credits, and timer.
- `MEANWHIL.TTM`: watch/clock interstitial behavior.
- `SUZYCITY.TTM`: Suzy city scene and thought/island timer.
- `GJVIS5.TTM`: plane scene, including the “jumping Johnny” and plane walk
  routines.
- `MJFIRE.TTM`, `MJFISH.TTM`, `MJSAND.TTM`, and `MJRAFT.TTM`: major island
  activity sequences and their named timer transitions.

These labels are original embedded program-data identifiers. They are useful
for a compatibility specification, but they do **not** recover original C/C++
or SCI source code.

## Research plan

1. Preserve hashes, sizes, headers, and provenance for every original file.
2. Safely expand the `$` installer payloads into a private working copy.
3. Identify executable formats (DOS MZ, Win16 NE, or installer container),
   segments, imported APIs, resources, and strings.
4. Disassemble the screensaver program and label entry points, window/saver
   message handling, clock/special-scene scheduling, random selection, and
   timing code.
5. Correlate calls/resource IDs with the already indexed `ADS`/`TTM` scene
   graph.
6. Produce readable pseudocode and an evidence-backed behavioral specification.
7. Keep any extracted proprietary binaries and art outside the public GitHub
   repository; publish only independently written documentation and our code.

## Boundaries

Machine code can be disassembled substantially, but the original symbol names,
comments, build settings, and exact high-level source structure cannot be
recovered perfectly. Findings will be classified as either **confirmed from
binary/data** or **inferred from behavior**.
