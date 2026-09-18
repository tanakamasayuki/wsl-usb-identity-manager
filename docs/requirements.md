# Requirements: WSL USB Identity Manager

*[English](requirements.md) | [日本語](requirements.ja.md)*

This document defines the requirements for the application. The measurements
behind them are in [research-findings.md](research-findings.md); the design of
the identification method is in
[identification-policy.md](identification-policy.md).

---

## 1. Purpose

For work where USB devices are forwarded to WSL with `usbipd` on Windows,
**make it certain which devices are plugged in and which physical device each
one is.**

This is not a GUI wrapper over `usbipd`. What it is for is **keeping track of
which device is which, over time.**

### 1.1 The problems

None of the following holds with the tool in use today (wsl-usb-gui).

1. **Identical devices with no serial number cannot be told apart.**
   With three CH340s plugged in, there is no way to say which board is which.
2. **The bus id cannot be trusted.**
   A bus id is `Hub_#NNNN` plus `Port_#MMMM`, and `Hub_#NNNN` is a dynamic
   number Windows assigns in the order it enumerates hubs
   ([F1](research-findings.md#f1-a-usbipd-bus-id-is-a-dynamic-windows-hub-number-not-a-physical-position)).
   It changes with the order hubs are plugged in and across a restart, so
   **the same bus id comes to mean a different device.**
3. **COM numbers and `/dev/ttyUSB*` move too**, for the same reason.
4. **A debug probe (a WCH-LinkE, say) can identify itself and still say nothing
   about the board behind it.**

### 1.2 What is identified

A USB-serial adapter or a debug probe **can be identified itself, and still
leaves what is connected to it unknown.**

- A CH340's, CP210x's or FTDI's serial number belongs to **the adapter**, not to
  the board behind it. Move the adapter to another board and the number is
  unchanged.
- A WCH-LinkE's serial number belongs to **the probe**, not to the board on its
  debug pins. Swap the board and the number is unchanged.
- On a native-USB part such as an ESP32-S3 or C3 the board is itself the USB
  device, so the problem does not arise.

When someone says "that device", they usually mean **the board, not the adapter
or the probe in front of it.**

> **The purpose of this application is to identify the board behind the adapter
> or the probe.**

#### The families it can identify

The board behind the adapter can be identified for two families.

| Family | What is identified |
| --- | --- |
| **ESP32** | the individual board by its eFuse MAC, and the chip type |
| **CH32 RISC-V** (behind a WCH-Link / WCH-LinkE) | the individual board by its part UUID, and the chip type |

**R1.1**: For anything else, identification stops at the adapter or the probe. In
that case the interface must show that what is identified reaches only the
transport layer.

**R1.2**: Keep the identification of each family in a replaceable unit, so
families can be added later.

### 1.3 The idea at the centre

**Keep "where it is plugged in" apart from "which physical device it is".**

| Class | Content | How it is treated |
| --- | --- | --- |
| Runtime connection | bus id / COM number / `/dev/ttyUSB*` / attach state | displayed only; never used as an identifier |
| USB identity | VID/PID / USB serial number / port path | read from Windows every time |
| Application identity | the board's own id, from a probe | held while running, dropped on disconnect (§4.2) |
| User metadata | the user's own names, notes and settings | saved (§7.1) |

**The only things saved are the ones the user decided.** Everything observed is
observed again. §4.1 and §7.1 give the reason.

---

## 2. Division of responsibility

### 2.1 This application, on the Windows side

- Enumerating the USB devices Windows knows about
- Reading their identifying information
- Asking a device what it is (probing)
- Reading and driving `usbipd` state (bind / unbind / attach / detach)
- Managing which WSL distribution to attach to
- Saving the user's own information (names, notes, attach settings)
- Watching connections and disconnections, and updating the display
- Showing the Windows-side state and the WSL attach state together
- Showing the result of each operation, and its errors

### 2.2 The WSL / Linux side, which is not this application's business

- Managing device nodes such as `/dev/ttyUSB*` and `/dev/ttyACM*`
- Handling device events with udev
- Device node permissions and Linux group membership
- Producing stable device names, and creating or removing symlinks
- Anything on the Linux side that follows an attach or a detach

The existing arrangement, where
[board-identify](https://github.com/tanakamasayuki/board-identify) manages the
symlinks inside WSL, stays as it is. **This application creates and changes no
symlinks.** It reads and displays them when it can, and nothing more.

### 2.3 Out of scope

The application does not deal with any of the following.

- Writing or managing udev rules
- Creating or removing symlinks inside WSL
- Linux device permissions and group management
- Starting applications inside WSL
- Managing Linux drivers for USB devices
- Writing firmware
- Managing development environments such as the ESP32 toolchain

---

## 3. The state model

### 3.1 Three layers of identity

As §1.2 sets out, there are up to three "things" on the end of one cable. They
are treated with the same three-layer model board-identify uses.

| Layer | Content | Example | How it is identified |
| --- | --- | --- | --- |
| **Port** | a transient position | bus id / COM8 / `/dev/ttyUSB2` | LocationPaths (never persisted) |
| **Transport** | a USB adapter, bridge or probe | CH340, CH343, WCH-LinkE | the USB serial number (which may not exist) |
| **Target** | the microcontroller board behind it | ESP32-S3, CH32X035 | **a probe** (§4.6) |

- On a native-USB board (an ESP32-S3, say) the transport and the target are the
  same thing.
- With nothing connected to a bare USB-serial adapter, there is no target.
- **One physical connection can carry a transport identifier and a target
  identifier at the same time** (R4.11).

**R3.1**: In the list and in the detail pane, it must be possible to tell whether
an identifier on screen belongs to the transport or to the target.

§3.2 to §3.5 divide the state of a device into four groups.

### 3.2 USB identity

What comes from the physical USB device and can be read from Windows.

| Item | Source | Notes |
| --- | --- | --- |
| VID / PID | instance id | |
| USB serial number | third element of the instance id | **some devices have none** (CH340 and others) |
| Manufacturer | `DEVPKEY_Device_Manufacturer` | |
| Product name | `DEVPKEY_Device_BusReportedDeviceDesc` | the USB iProduct string |
| Windows friendly name | `DEVPKEY_Device_FriendlyName` | |
| Device instance id | `DEVPKEY_Device_InstanceId` | **the join key with usbipd** |
| Container id | `DEVPKEY_Device_ContainerId` | UUIDv5 and stable with a serial number, UUIDv1 without |
| Bound driver | `DEVPKEY_Device_Service` | shown, to say which driver is attached |

### 3.3 Runtime connection

The connection as it is now. **Never used as a lasting identifier.**

| Item | Source |
| --- | --- |
| Physical port path | `DEVPKEY_Device_LocationPaths` |
| Bus id | `BusId` from `usbipd state` (`null` when not connected) |
| Present / missing | the enumeration |
| Shared / not shared | `PersistedGuid` from `usbipd state` |
| Attached / detached | `StubInstanceId` / `ClientIPAddress` from `usbipd state` |
| The attached client | `ClientIPAddress` from `usbipd state` (the WSL-side IP; usbipd does not report the distribution name) |
| COM port | registry `Device Parameters\PortName` (**for display only**) |

### 3.4 Application identity

What the equipment behind the USB device reports. Obtained by probing.

```text
Device ID          e.g. an ESP32's eFuse MAC, a CH32's part UUID
Device Type        e.g. esp32-s3, ch32x035c8t6
Hardware Revision
Firmware Version
Role
```

### 3.5 User metadata

What the user maintains themselves. It survives changes in whatever Windows or
USB reports.

```text
Alias (display name)   required
Memo                   required
Tags
Purpose / location / asset number
Default WSL distribution
```

Automatic attach is not an attribute of a device but **a list of rules** (§9).
One rule can match several devices, so holding it per device would scatter the
same setting across them.

---

## 4. Identifying a device

See [identification-policy.md](identification-policy.md) for the design. This
section states the requirements only.

### 4.1 The two routes

| # | Route | What it covers | Probe |
| --- | --- | --- | --- |
| 1 | the USB serial number identifies the transport | devices that report one | **not needed** |
| 2 | a probe identifies the target | devices whose board can be asked | needed |

Route 1 reaches **the transport and no further**. The serial number an adapter
or a probe reports for itself does not name the board behind it (§1.2).

**R4.1**: A device that reports a serial number must have it shown as the
transport's identifier. It must not be treated as the target's.

**R4.2**: An identity must not be stored against a port position or a USB serial
number and used, next time, to infer the target from that association.

> **Why inferring is forbidden**
>
> A USB-serial adapter can be moved to a different board without one byte
> changing on the USB side. The same goes for a port position: plug a different
> board into the same socket and the value is the same. So "the board that was
> behind this port, or behind this serial number, last time" does not give the
> board now. **Infer it and the application shows a wrong name confidently.**
> That is worse than showing nothing.

### 4.2 How long an identity lasts

**R4.3**: A target identity is held only while its device is connected. It must
be dropped the moment the device is unplugged.

Unplugging is the one moment at which the far end of the cable can change.
Attaching to and detaching from WSL is not a physical disconnection, so it does
not drop the identity.

**R4.4**: It must be possible to tell an unidentified device from an identified
one in the interface. A guessed identity must never be shown in the same form as
one that was asked for and answered.

**R4.21**: The last identification that succeeded is shown for reference, as the
last one rather than the current one. It must be visually distinct from a
confirmed identity and must not be treated as one (R4.4). The date it was read
is shown with it.

> Dropping an identity on disconnect (R4.3) is because what is on the end of the
> cable can have changed. That is "may have changed", not "has changed". Erasing
> what was known up to a moment ago leaves a device that was merely replugged
> looking exactly like one that has never been identified at all; keeping it as
> a reference answers both questions.
>
> The date is shown because the value outlives a restart (R4.23). "What it was"
> is only something to go on next to "when".

**R4.23**: The last identification is saved, and shown for reference after a
restart.

> A device attached to WSL cannot be probed at all (F4). Without saving it, a
> machine that starts with its boards already forwarded can never say which is
> which — the answer would be "detach them and identify again", which is not
> what this application is for.

**R4.22**: The last identification must not be used for:

- deciding an automatic attach (R4.6, §9)
- restoring a confirmed identity — what is read back is always a reference

> It is material for the person reading the list, not for the machine deciding
> what to hand over. The instance id of a device with no serial number derives
> from the port, so plugging a different board into the same socket leaves the
> previous board's identification sitting there. Somewhere a person can see it
> and say "that is not it" is a good place for it. A rule is not.
### 4.3 When a probe may run

A probe has side effects. `esptool` and its kin restart the board, and attaching
to a WCH-Link halts the target core.

**R4.5**: A probe may run at these two moments and no others.

1. **When the user asks for it**
2. **Inside the grace window after an arrival** (§4.4; can be switched off)

**R4.6**: A probe must not run:

- on a periodic poll
- when the list is refreshed
- in a sweep of everything at startup
- to decide an automatic attach

**R4.7**: State the side effects — that the target restarts, and so on — before
running a probe.

### 4.4 Identifying on arrival

**R4.8**: Provide a feature that identifies a device automatically just after it
is plugged in.

| Setting | Default |
| --- | --- |
| Automatic identification | on |
| What it covers | only devices with no serial number |
| Grace window | 10 seconds from the arrival event |
| Excluded VID/PID | listed by the user, empty by default |

Devices that report a serial number are left out because most of them are
native-USB boards, where the transport and the target are the same thing and
nothing needs to be asked.

> **There is a hole in that rule.** The board behind an *adapter* that reports a
> serial number (a CH343, say) or behind a **WCH-Link** still cannot be known
> without asking. Those fall outside automatic identification and stay
> unidentified until someone identifies them by hand. This follows from putting
> first the narrowing of what gets restarted or halted automatically, and is
> recorded as a choice rather than an oversight. Identify-all (R4.19) is how the
> hole is filled.

**R4.9**: Nothing may be sent automatically to a device whose VID/PID is
excluded.

> **Why an exclusion list and not an allow list**
>
> "List the VID/PIDs to cover" does not work. A CH340's VID/PID says nothing
> about whether a dev board or a router console is on the other end, so an allow
> list ends up naming every common bridge anyway — the same effect as an
> exclusion list, reached the long way round. Naming the hardware that must not
> be disturbed fits what actually happens.

**R4.19**: Provide an action that **identifies every connected device that has
not been identified**, so the user can pick up in one action what automatic
identification left out.

**R4.20**: Inside the grace window, keep retrying until the probe can run.

> There is a gap between a device becoming visible to Windows and its COM port
> being assigned. Deciding once, at the instant it arrives, loses that race more
> often than not. What bounds the attempts is the window, not the first try.

### 4.5 The identifier format

An identifier is **internal to this application**; compatibility with other tools
is not required. The format follows the rule board-identify has proven.

```text
<variant>-<unique-id>

esp32-s3-7cdfa1123456
ch32x035c8t6-1ff9abcd880ebc48
wch-link-fc928f068181
```

Rules: lowercase ASCII / `-` as the separator / punctuation stripped from the
unique id / a unique id shorter than 6 characters is not used / never contains
`/` or NUL.

**R4.10**: Generating an identifier must be deterministic. The same input — chip
type and unique id — always gives the same string.

**The rule must not change once it has been released.**

> Identifiers are shown on screen, and people use them in notes and to line up
> against other tools. Change the rule and the same piece of hardware turns up
> under a different name. Precisely because the application saves none of them,
> **there is no way to correct the strings that have already left it.**

**R4.11**: One device can hold several identifiers at once. For a debug probe,
both the probe itself and the board behind it are held.

### 4.6 What is probed, and how

Identifying the transport — the adapter or the probe — needs only the USB
descriptors. **Identifying the target, the board behind it, needs a probe.**

#### Families whose target can be identified

| Family | Method | Target ID obtained | Prerequisite |
| --- | --- | --- | --- |
| **ESP32** (behind a CH340, CP210x, …) | ROM bootloader over the serial line, reading the eFuse MAC | MAC address + chip type | the COM port can be opened |
| **CH32 RISC-V** (behind a WCH-Link) | the WCH-Link vendor protocol | part UUID + chip signature | the probe is in RISC-V mode |

For CH32 the whole sequence is these calls into `ch32rv-wchlink`.

```rust
WchLink::open(&UsbDeviceInfo)   // interface 0 in RISC-V mode (1a86:8010)
  .probe_info()      -> ProbeInfo        // about the probe itself
  .set_speed_default(Speed)              // optional; see R4.16
  .attach_chip()     -> AttachInfo       // family + chip_id (top 4 bits = silicon revision)
  .chip_info()       -> ChipInfoStatus   // flash_kb, uuid: [u8; 8], protection_raw
  .detach_chip()
```

- **The target's unique identifier** = `ChipInfo.uuid`, an 8-byte factory UUID
- **The target's chip type** = the family and chip id from `AttachInfo`, resolved
  to an SKU through `ch32rv-target`'s `Db`
- `attach_chip()` halts the target core, which is what makes this a probe with
  side effects (R4.5)

**R4.15**: A WCH-Link in ARM mode cannot be opened on the RISC-V VID/PID. Detect
the mode and say that it has to be switched. **The application must not switch
it.**

**R4.16**: A failed `set_speed` must not be treated as a failed identification.

> The debug link speed is a performance hint and is not needed to identify
> anything. The original WCH-Link (CH549 based, measured on firmware 2.6) does
> not implement the command and answers `82 81 01 ff` whether it is sent before
> or after attach. Requiring it would fail an identification that would otherwise
> have worked.

**R4.12**: Connecting to a WCH-Link must not require WinUSB to be assigned.

> WCH's own driver (`WCHLink_A64`) on interface 0 is the normal state of a
> WCH-Link. When nusb cannot claim the interface, `ch32rv-usb` falls back on
> Windows to the CH375-style `DeviceIoControl` path, so **it probes on the stock
> driver** (measured). Asking for a driver swap puts the user through Zadig or
> similar and costs them the WCH-Link's serial port. **The application must not
> swap drivers.**

#### Where no probe is needed

| Device | Why |
| --- | --- |
| ESP32-S3 / C3 and similar (native USB) | the transport and the target are the same, and the MAC is exposed as the USB serial number |
| the USB-serial adapter itself | the descriptors identify the transport |

#### Families that are not supported

Anything else — Arduino, RP2040, STM32, an unknown board behind a generic CH340 —
**is identified down to the transport only.**

**R4.13**: Do not attempt to identify the target of an unsupported family. Show
that it is unsupported, and let the user fill the gap with an alias.

That said, nothing in the descriptors says what is behind a generic USB-serial
adapter, so "unsupported" can only be decided in advance **when the VID/PID
identifies the family**. Otherwise it emerges from trying and getting no
answer (§4.7).

### 4.7 Probe precedence

**R4.17**: Each probe declares whether it can **recognise its device from the
VID/PID**.

| Kind | Meaning | Example |
| --- | --- | --- |
| By identifier | the VID/PID says what the hardware is; decided without touching anything | CH32 family (WCH-Link) |
| Fallback | only the shape matches; what is behind it is unknown until asked | ESP32 family (a COM port) |

**R4.18**: When a probe that recognises by identifier claims a device, the
fallback probes must not run.

"Claims" includes claiming it and being unable to proceed — an ARM-mode
WCH-Link, say. An ARM-mode WCH-Link is still a WCH-Link, and still not something
to send an ESP32 reset and sync sequence to.

> **The fallback must not carry an exclusion list.**
> A WCH-Link has a serial port, so a generic serial probe cannot tell it from an
> adapter. Expressed as "VID/PIDs the ESP32 probe skips", that list grows with
> every family added. Expressed as precedence, a new family only has to name its
> own VID/PID.

**R4.14**: Structure it so that adding a family means adding identification and
nothing else — no change to the families already there.

---

## 5. Working with usbipd

### 5.1 The join key

**R5.1**: Match devices to usbipd on the `InstanceId` that `usbipd state`
returns. A bus id must not be used for matching.

**R5.2**: Read usbipd state only from `usbipd state` (JSON). The text output of
`usbipd list` must not be parsed.

**R5.3**: Track an attached device by its `StubInstanceId`
(`Vid_80EE&Pid_CAFE\<the original third element>`).

### 5.2 Operations

These must be available from the interface.

| Operation | Command | Rights |
| --- | --- | --- |
| Bind (share) | `usbipd bind --busid <BUSID>` | **administrator** |
| Unbind | `usbipd unbind --busid <BUSID>` | **administrator** |
| Unbind (not connected) | `usbipd unbind --guid <PersistedGuid>` | **administrator** |
| Attach | `usbipd attach --busid <BUSID> --wsl <DISTRO>` | ordinary user |
| Detach | `usbipd detach --busid <BUSID>` | ordinary user |
| Refresh state | `usbipd state` | ordinary user |

**R5.4**: Use the bus id read from `usbipd state` immediately before the command
is issued. A recorded bus id must never be reused.

**R5.5**: The user must be able to choose which device an operation acts on with
its identifying information in front of them.

**R5.11**: Do not offer an operation that needs a bus id for a device that is not
connected.

A bind record outlives its device, so **the only operation that means anything
for a device that is not connected is unbind.** There is no bus id then, so the
device is named with `--guid`, which usbipd provides for exactly this.

> This does not contradict R7.1, which forbids persisting `PersistedGuid` as an
> identifier. The value is read from `usbipd state` immediately before the
> command and passed straight to it — the same treatment as R5.4.

### 5.3 Rights

**R5.6**: The application itself starts unelevated. Only when an operation needs
administrator rights is `usbipd.exe` run elevated, through
`ShellExecuteExW(verb="runas")`, and its exit code read to decide the result.

**R5.10**: Before putting a value on an elevated command line, check that it has
the expected shape (a bus id is `<digits>-<digits>`). Do not run with a value
that fails the check.

> The command line of an elevated process cannot take its arguments as an array
> the way `CreateProcess` can. Since it has to be built as a string, what can end
> up in it is closed off by the caller.

**R5.7**: Provide a path through setting up `usbipd policy add` (AutoBind), which
brings elevation in everyday use down to almost never.

### 5.4 Warnings that depend on the environment

**R5.8**: Detect the warnings usbipd prints — incompatible USB filter drivers and
so on — and tell the user when `bind --force` will be needed.

**R5.9**: `bind --force` takes the device away from Windows and so cannot coexist
with probing. State that constraint in the interface.

---

## 6. Working with WSL

**R6.1**: Enumerate the available WSL distributions and let one be chosen as the
attach target.

**R6.2**: Let a default distribution be set per device.

**R6.3**: Let the WSL-side state be **read**. Read only; nothing is changed.

Examples of what is read:

```text
the matching Linux device node   /dev/ttyUSB1
existing symlinks                /dev/board-identify/by-id/co2-sensor
vhci ports against bus ids       usbip port
```

`usbipd-win` ships the Linux binary `usbip`
(`C:\Program Files\usbipd-win\WSL\usbip`). Running it through `wsl.exe` gives
the state of the vhci ports.

**R6.4**: Nothing read from the WSL side may be used as lasting identifying
information on the Windows side. It is for display only.

---

## 7. The data model and what is saved

### 7.1 What is saved

**Only what the user decided, and what is labelled as a reminder.** What the
application observed — the enumeration, the identifications, the usbipd state —
must not be saved as fact. It can be observed again at the next start, and saving
it turns "this is how it was last time" into "this is how it is".

The last identification (R4.21) is saved with that path closed off: what is read
back is always presented as a reference, and never reaches the automatic-attach
decision (R4.22).

```text
Settings (application-wide)
  auto_identify            identify a device right after it arrives (§4.4)
  auto_exclude             VID:PIDs never identified automatically
  confirm_before_identify  state the side effects before probing (§4.3)
  start_with_windows       start when the user signs in to Windows
  auto_attach              automatic attach on or off (§9)
  auto_attach_rules        the automatic attach rules (kind + value; R9.1)
  told_about_tray          whether closing to the tray has been explained once (R10.15)

User metadata (per target; §3.5)
  key                      the key from R7.7
  alias / memo / tags
  default_wsl_distribution

Last seen (per device; reference only; R4.21 / R4.23)
  instance_id              the device instance id it was seen on
  name                     the name Windows had while it could see it (R10.23)
  identity                 the last identification
  identified_at            when that identification was read
```

**R7.7**: The key for user metadata is the USB serial number when there is one,
and the port path (`DEVPKEY_Device_LocationPaths`, F2) when there is not.

Metadata keyed on a port path is **a note about whatever is plugged into that
port**, not about a particular unit. Swap what is plugged in and the note goes
with the port. The interface shows that difference (R4.4).

### 7.2 What must never be saved

**R7.1**: None of the following may be persisted as an identifier.

- The bus id
- The COM number (keeping it as a displayed value is fine)
- Linux device nodes such as `/dev/ttyUSB*`
- usbipd's `PersistedGuid` — an identifier internal to usbipd, which on a device
  with no serial number ends up tied to the port

**R7.8**: The application identity obtained by a probe (§3.4) must not be saved
**as fact**. A confirmed identity is held only while the application runs, and
dropped on disconnect (R4.2 / R4.3). What may be saved is the last identification
as a reference (R4.21 / R4.23), and what is read back must never be promoted to a
confirmed identity (R4.22).

**R7.9**: What is saved for reference has a limit on how many devices it covers;
past that, the least recently seen goes first. The user must be able to clear all
of it.

> Boards travel between ports faster than they multiply. Without a limit, every
> socket a device was ever plugged into earns a permanent entry. Clearing is
> offered because this is the one part of the file that can be wrong **without
> anything having gone wrong**: move a board to another port and its old entry
> stays behind.

### 7.3 Where it is saved

**R7.2**: Beside the executable when a `portable.txt` sits in the same directory;
under `%APPDATA%` otherwise.

### 7.4 Format and schema version

**R7.3**: The format is human-readable text (JSON), so that settings can be
carried around in a portable install and looked at when something goes wrong.

**R7.4**: The file carries a schema version at the top.

```json
{ "schema_version": 1, "settings": { ... } }
```

**R7.5**: Check the schema version when reading.

- A known older version → migrate it, keeping the pre-migration file aside
- **An unknown newer version → do not read it, and do not write over it.**
  Settings written by a newer version must not be destroyed by an older one

**R7.6**: Save in a way that an interrupted write cannot destroy the file: write
a temporary file and replace the target with it.

---

## 8. Watching for changes

**R8.1**: Detect and display these changes.

- A USB device connected to or disconnected from Windows
- A change in `usbipd` bind state
- An attach to or detach from WSL

**R8.2**: Call `CfgMgr32` / `SetupAPI` natively for enumeration and property
reads. The PowerShell PnP cmdlets must not be used — measured at more than 700
times slower
([F7](research-findings.md#f7-reaching-windows-pnp-through-powershell-is-not-usable)).

**R8.3**: Use `CM_Register_Notification` or `WM_DEVICECHANGE` for hot-plug
notification. Periodic polling must not be the primary means of detection.

**R8.4**: Decide "has arrived" from whether the Windows device node **can be
reached**. It must not be decided from the presence of `BusId` in
`usbipd state`.

> usbipd keeps reporting a bus id throughout an attach (measured). Taken as the
> test for being connected, it does not change across an attach or a detach, and
> it calls the period when Windows cannot touch the device "connected".

---

## 9. Automatic attach

Hands a device matching a rule to WSL without the user doing anything.

### 9.1 The kinds of rule

**R9.1**: An automatic attach rule names a device in one of four ways.

| Kind | What it names | Stability |
| --- | --- | --- |
| **Board ID** (the identifier) | what the board itself answered (§4.5) | follows the board. **Known only once identified** |
| **USB serial number** | that USB device itself | stable, but does not name the board behind an adapter (§1.2) |
| **VID/PID** | every device of that kind | stable. "every CH340" |
| **Bus id** | whatever is at that bus id | **not stable** (F1) |

**R9.2**: Show which layer (§3.1) each kind names.

**Matching on the device name (description) is not offered.**

> The device name is a string from Windows or the USB descriptors, and **every
> device of the same model carries the same one.** That makes it a coarser rule
> than VID/PID, and on top of that it can change — with a driver swap, or with a
> COM number mixed into it. With VID/PID available to choose, matching on the
> name buys nothing.

> A bus id is renumbered by plugging in a hub (F1). It is offered as a rule
> anyway because "send whatever I plug into this socket to WSL" is genuinely
> useful. **It is offered, and what it names is stated plainly.**

**R9.3**: When several rules match, report the match under **the most specific
kind** — the order of the table above, top first.

### 9.2 When it runs

**R9.4**: A probe must never run to decide an automatic attach (R4.6).

Every value a rule is matched against comes from the enumeration that has already
happened. A board-ID rule therefore **matches only a device that has been
identified**; until then it waits, unmatched.

**R9.5**: An automatic attach may run only when all of the following hold.

- Automatic attach is on
- A rule matches
- **`usbipd` is already sharing the device (bound)**
- Windows can see it (R8.4)

> Sharing needs administrator rights (§5.3). Going off to share a device that has
> never been shared would put a UAC dialog in front of a user who did nothing.
> **An automatic action must not ask for elevation.**

**R9.6**: A device queued for identification is not attached until that has run.

> Attaching takes the device out of Windows' reach, and with it the ability to
> probe. Attach first and the answer a board-ID rule is waiting for can never
> arrive.

**R9.7**: Automatic attach is attempted once per device per arrival. After a
failure it is not retried until the device is unplugged and plugged back in.

> Retrying every two seconds means a device that keeps failing keeps producing
> errors. And an automatic attach reappearing straight after a manual detach
> would mean **the application undoing what the user just did.** Attaching and
> detaching do not clear this record; only a physical disconnection does.

**R9.8**: Automatic attaches run one device at a time.

### 9.3 Operating it

**R9.9**: Automatic attach must be switchable on and off **in one action from the
main window**.

> "I want this on the Windows side right now" happens in the middle of working.
> It should not mean opening a settings dialog.

**R9.10**: Provide a screen that lists the rules and allows adding and removing
them, showing how many devices each rule matches at the moment.

**R9.11**: With a device selected, it must be possible to **choose one** of the
values that device offers (whichever of R9.1's four kinds it actually has) as a
rule. The choice is exclusive, and "none" must be available.

**R9.12**: When the choice removes a rule that also matched other devices, say
so.

> A VID/PID rule is not that one device's alone. An action on one device's pane
> changing how others behave must not happen silently.

**R9.13**: It must be possible to tell from the list which devices a rule
matches. Matches are shown while automatic attach is off as well, **displayed so
that it is clear nothing will happen.**

> Rules are usually built before the switch is turned on. Without seeing what
> they currently hit, building them is guesswork.

---

## 10. The user interface

### 10.1 The main window (the device list)

The USB devices Windows enumerates and the devices `usbipd` has a record of go in
**one list**, with filters by state along the top.

> Split into a table per state, a row that is attached jumps between tables and
> disappears from under the user as they watch. In one list the row stays put and
> only its state changes.

| Column | Content |
| --- | --- |
| State | shared / attached (§5.2) |
| Connection | whether Windows can reach it; the bus id and the port |
| Device | the name Windows and USB report. **Never rewritten by identification** |
| VID:PID | with the vendor name beside it (§10.5) |
| USB serial number | the USB serial number, or a note that there is none (transport layer, §3.1) |
| Board | the identification, or not identified (target layer; §4.2, R4.4) |
| Auto | whether an automatic attach rule matches (R9.13) |

**R10.12**: A column heading says **what is in the cell**. The names of the layers
in §3.1 — port, transport, target — are not used as headings.

> "Transport" is the name of a layer in the design, not a word the user brings
> with them. What is in the cell is a USB serial number, and saying so needs no
> explanation. The distinction between layers is carried by the tooltips and the
> detail pane, not by the heading.
>
> "Number" is part of it because nearly everything this application deals with
> sits behind a USB-serial adapter, where **"serial" on its own reads as the
> serial line.**

**R10.1**: Devices Windows cannot see — attached, or only a `usbipd` record —
appear in the list too, distinguishable from connected ones.

**R10.2**: For an unidentified device of a family that can be identified, put the
way to identify it in the row.

**R10.7**: An identification must not rewrite the device name. The name Windows
and USB report stays as it is, and the identification goes in its own column.

> With the name rewritten, there is no telling which Windows device the row is,
> and it can no longer be lined up against `usbipd` or USBTreeView.

**R10.23**: Where Windows can no longer describe a device, the name it last
reported is shown for reference. It must be visually distinct from a confirmed
value, and shown beside the current name rather than in place of it (R10.7).

> A device attached to WSL is re-enumerated as the VBoxUSB stub, so the name
> Windows had for it is gone (F4); a bind with `--force` does the same. What
> stands in is the description `usbipd` cached, and if that cache was taken
> after the driver was swapped it is a generic name too. A row that stops saying
> which unit it is the moment it is shared is the state this application exists
> to prevent.

### 10.2 The detail pane

The four groups of §3.2 to §3.5 are shown apart from one another.

```text
USB identity         VID / PID / serial / manufacturer / product /
                     instance id / location path / driver service
Application identity Device ID / device type / HW revision / FW version / role
User metadata        Alias / memo / tags / default WSL
Runtime connection   bus id / COM / shared / attached / the attached client
WSL state            device node / symlinks (read only)
```

The operations for the selected device sit alongside them.

- The `usbipd` operations (§5.2) and identification (§4.3)
- **Choosing the automatic attach rule** (R9.11)

**R10.22**: The version of the build must be visible in the interface, in the
same place as the path to the log.

> A report needs the version and the log together. There is no reason to make
> someone look in two places for them. The value comes from the single version
> in `Cargo.toml` (R12.4).

**R10.13**: The values in the detail pane must be selectable and copyable.

> What is shown here exists to be pasted into a `usbipd` command or an issue.
> Readable but not extractable means copying it out by hand.

### 10.3 The settings screen

- How many devices are remembered, and clearing them (R7.9)
- Automatic identification on arrival and its exclusion list (§4.4)
- Whether the side effects are confirmed before identifying (§4.3)
- Starting when the user signs in to Windows
- The default WSL distribution
- The display language (§10.4)

Automatic attach, on/off and its rules, **does not go here**. The switch is on the
main window (R9.9) and the rules have their own screen (R9.10).

**R10.8**: When settings cannot be saved — the file was refused under R7.5, for
instance — say so on the settings screen. A setting that silently does not
persist does more harm than one that says it will not.

### 10.4 Languages

**R10.3**: Read the OS display language at startup and use it automatically when
it is one of the supported languages.

**R10.4**: Let the display language be chosen explicitly on the settings screen.
The choices are "follow the system" (the default) and each supported language.

**R10.5**: English is the reference text for interface strings. Translations live
in a file per language, and an untranslated item falls back to English.

**R10.6**: The supported languages are English and Japanese. Adding a language
must mean adding a translation file and nothing else.

**R10.9**: Messages returned by the identification code carry a translation key.
Translating by matching on the English text is forbidden: it comes apart the
moment the wording is edited.

### 10.5 Vendor names

VID and PID as numbers do not say who made something, so the vendor and product
names from `usb.ids` — the
[USB ID Repository](http://www.linux-usb.org/usb-ids.html) — are shown beside
them.

**R10.10**: `usb.ids` must not be shipped with this application. The copy in the
`usbipd-win` install directory, beside `usbipd.exe`, is what is read.

> Shipping it is redistribution, and brings its licence terms into scope. This
> application already depends on `usbipd-win`, and `usbipd-win` puts the same
> file there for the same purpose. Reading it is enough.

**R10.11**: Keep working when `usb.ids` cannot be found. The vendor column is
simply empty; it is not an error.

> Vendors that are not listed are common enough, and a user gains nothing from
> telling "not listed" apart from "could not be read".

### 10.6 Staying resident, and the window

Automatic attach (§9) and identification on arrival (§4.4) **only work while
the process is running.** Exiting with the window would leave the rules that
were set up doing nothing.

**R10.14**: Closing the window does not quit; the application stays in the
notification area. Quitting is done from the tray icon's menu.

**R10.15**: The first time the window is closed, say once that this is **not
quitting**, and how to quit. Do not say it again.

> Without it, an application the user thinks they closed goes on attaching
> devices automatically and restarting boards through identification on arrival.
> Saying it every time would be in the way, so it is said once and recorded
> (`told_about_tray`).

**R10.16**: The tray menu holds the following.

| Item | Content |
| --- | --- |
| Status | how many are connected / shared / in WSL (not clickable) |
| Auto-attach | switches it on and off (one of the routes of R9.9) |
| Identify all | identifies every connected device that has not been (R4.19); not selectable when there are none |
| Open | shows the window; a left click on the icon does the same |
| Settings | shows the window with the settings panel open |
| Quit | exits the application |

**R10.17**: The tray labels come from the same translations as the rest of the
interface. The backend holds no catalogue of its own (R10.5).

**R10.20**: Identify-all started from the tray follows the same confirmation
setting as elsewhere (R4.7). When a confirmation is due, the window is shown
first and then the question asked.

> The confirmation dialog lives in the window. Asked while it is hidden, the
> work stops in front of a dialog nobody can see.

**R10.21**: While it runs, the tray's status line shows the progress.

> Since identify-all can be started with the window closed, there would
> otherwise be a stretch of boards restarting one after another with nothing on
> screen to say so.

**R10.18**: Started through the autostart entry (§7.1), the application goes to
the notification area **without opening a window**.

> Signing in is not a request to be shown a window. For someone who keeps it
> resident for the sake of automatic attach, a window at every sign-in is only
> in the way.

**R10.19**: Launching the application again while it is resident **shows the
hidden window and brings it to the front** (R13.10).

---

## 11. The technical stack

| Layer | Choice |
| --- | --- |
| Language | Rust |
| GUI | Tauri 2 (WebView2) |
| Device enumeration and properties | the `windows` crate → `CfgMgr32` / `SetupAPI` |
| Raw USB access | `nusb` (through `ch32rv-usb`; not a direct dependency) |
| Serial | `serialport` |
| WCH-Link probe | the [ch32rv](https://github.com/ch32-riscv-ug/ch32rv) crates (`ch32rv-wchlink` / `ch32rv-usb` / `ch32rv-target`) |
| ESP32 probe | the `espflash` crate |

> **Why Tauri (WebView2) for the GUI**
>
> - A list, a detail pane and a settings screen are least work in HTML/CSS.
> - Japanese is typed into the alias and memo fields (§3.5), so the IME has to
>   work reliably. WebView2 is the OS text input mechanism itself.
> - Displaying Japanese needs no bundled font; the OS font is used. A native Rust
>   GUI cannot subset a font when arbitrary kanji can be typed, so it would need
>   a CJK font (5–8 MB) shipped with it.
>
> The cost is the dependency on WebView2, which as §11.2 sets out can be detected
> and is rarely missing.

### 11.1 Which layer does what

**R11.1**: `CfgMgr32` is responsible for enumeration and for reading identifying
information. Enumeration through `nusb` is an internal matter for a probe and
must not be a source for the list.

> On Windows, `nusb` cannot read a serial number when the whole device is bound
> to a specific driver, which is the case for a CH340 (`CH341SER_A64`).
> `CfgMgr32` returns the instance id, LocationPaths, ContainerId and service, so
> it carries more besides (F3).

### 11.2 WebView2

**R11.2**: Check for the WebView2 runtime at startup. When it is not installed,
point at where to download it and exit.

Either of these existing with a `pv` other than `0.0.0.0` means it is installed.

```text
HKLM\SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}\pv   per-machine
HKCU\Software\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}\pv                per-user
```

- Windows 11 ships it with the OS.
- For Windows 10 Microsoft states it is present on the large majority of
  machines. The exceptions are Windows Server, LTSC, and machines cut off from
  Windows Update.
- It is installed with the Evergreen Bootstrapper (about 2 MB,
  `MicrosoftEdgeWebview2Setup.exe /silent /install`). The fixed version (over
  250 MB) is not used.

The installer resolves this at install time through NSIS's `webviewInstallMode`
(`downloadBootstrapper`), so the check is mainly for the portable build.

### 11.3 Depending on ch32rv

**R11.3**: Depend on the **crates.io releases** of the ch32rv crates. No git
references and no vendoring.

```text
ch32rv-wchlink   the WCH-Link protocol
ch32rv-usb       finding and opening a probe (an nusb wrapper, with the CH375 fallback)
ch32rv-target    the chip id → SKU database
```

`serialport` is pinned to the version ch32rv's workspace fixes. It is shared with
espflash, so a split version breaks both.

#### Depending on a 0.x crate

ch32rv is before 1.0 and **its API is expected to change.** Under Cargo's semver
`0.7` → `0.8` is a breaking change.

**R11.5**: Pin to the minor version (`"0.7"` = `>=0.7.0, <0.8.0`). Moving to the
next minor version is done deliberately, never automatically.

**R11.6**: Do not take a ch32rv minor bump **until identification has been
re-checked on real hardware.** CI has neither a WCH-Link nor a target board, so
it cannot verify this.

**R11.7**: USB enumeration through `ch32rv-usb` is used **only to find and open a
WCH-Link probe.** The source for this application's device list is `CfgMgr32`
(R11.1). Where the two enumerations are matched up, the instance id is the join
key.

### 11.4 Rust discipline

**R11.4**: The same discipline as ch32rv.

```toml
edition      = "2024"
rust-version = "1.88"

[lints.rust]
unsafe_code = "forbid"

[lints.clippy]
unwrap_used   = "deny"
expect_used   = "deny"
todo          = "deny"
unimplemented = "deny"
panic         = "deny"
```

`unsafe_code = "forbid"` can collide with calling the Windows API directly. Where
it does, **the crate doing the FFI is separated out and only that crate sets
`unsafe_code = "allow"`.** No unsafe reaches the application layer or the
identification layer.

---

## 12. Distribution and releases

### 12.1 What is published

Two files per release.

```text
wsl-usb-identity-manager_<version>_x64_setup.exe     NSIS / per-user / no administrator rights
wsl-usb-identity-manager_<version>_x64_portable.zip  portable (ships portable.txt)
```

**R12.1**: No MSI.

> With per-machine (MSI) and per-user (NSIS) side by side, the same application
> ends up installed in two places and there are two update paths.

**R12.2**: No self-update mechanism. Updating is `winget upgrade`, or replacing
the ZIP.

> Self-update alongside WinGet puts winget's record out of step with what is
> actually installed. Tauri's updater also requires signing, and losing the key
> means no further updates can be published.

**R12.3**: Register only the NSIS installer with WinGet
(`InstallerType: nullsoft`). The portable ZIP is distributed from the GitHub
release directly.

> WinGet fills in the silent-install switches automatically for `nullsoft`, and
> the community repository requires silent installation. A `zip` can be
> registered too, but needs `NestedInstallerType`, and puts a GUI application
> behind WinGet's Links shim.

**R12.9**: The portable ZIP is made from the built executable and its resources.

> Tauri does not officially support a portable target. This ZIP is for machines
> that already have WebView2; unlike the installer it cannot embed a bootstrapper
> (§11.2).

### 12.2 Versioning

**R12.4**: The `version` in `Cargo.toml` is the single source of the version.
`tauri.conf.json` carries no `version` field — Tauri falls back to `Cargo.toml`
when it is absent.

### 12.3 The release procedure

**R12.5**: A release is started by hand from the GitHub Actions page, through
`workflow_dispatch`. Pushing a tag does not release.

The bump level (`major` / `minor` / `patch`) is chosen when it is started. The
procedure is in [release.md](release.md).

**R12.6**: Changes are written into `## Unreleased` in `CHANGELOG.md` as they are
made. The release process inserts the version number and copies that section into
the release notes.

**R12.7**: Set `concurrency` on the workflow, so two runs cannot collide over the
version.

**R12.8**: The bump commit has to be pushed, so the default branch's protection
settings and the workflow's permissions have to agree.

---

## 13. Diagnostics and error handling

### 13.1 Checking preconditions

**R13.1**: Check the following at startup and, where one is not met, **say what
is missing and what to do about it.** Never fail silently.

| Check | When it is not met |
| --- | --- |
| The WebView2 runtime | point at the download and exit (R11.2) |
| `usbipd.exe` exists | explain how to install it. The device list still works; usbipd operations are disabled |
| The `usbipd` version | warn below 5.x, saying the `state` JSON may be shaped differently |
| The `usbipd` service is running | explain how to start it |
| WSL and its distributions | disable attaching only; everything else still works |

**R13.2**: Even when a precondition is not met, **whatever does work must stay
usable.** The only thing that stops startup outright is a missing WebView2.

### 13.2 Knowing when an asynchronous operation finished

For `usbipd attach` and its like, the command exiting and the state actually
changing are not the same moment.

**R13.3**: Decide whether an operation succeeded not from its exit code alone but
**by reading `usbipd state` again.**

**R13.4**: Put a timeout on waiting for the state to change. On a timeout, say
"the operation was issued but its result could not be confirmed". Do not assert
either success or failure.

**R13.11**: Read the state again and update the display when an operation fails
too.

> A failure does not mean nothing happened; it can have got part of the way.
> Leaving the display as it was puts **the state from before the operation next
> to the error**, and the user reads that as the state after it.

### 13.3 Exclusion

**R13.5**: Two probes must never run against the same device at once. Use a lock
per device (`ch32rv-usb` has an equivalent mechanism).

**R13.6**: An attach or bind against a device being probed is either refused or
made to wait until the probe finishes. A `bind --force` during a probe cuts the
probe off partway.

**R13.10**: A second copy of this application must not run. When one is already
running, bring its window to the front and exit.

> With two running, both drive `usbipd` and both act on the automatic attach
> rules. Two attaches collide over one device and one of them fails, with the
> user having done nothing. The scope is the sign-in session: the settings file,
> the `Run` entry and the window all belong to one user, so another user signing
> in may run their own copy.

### 13.4 The log

This drives hardware, and **reproducing a problem needs the log from where it
happened.**

**R13.7**: Record the following to a file.

- The application's name and version, at startup
- Connections and disconnections (instance id, location path, time)
- Probes and their results (the device, how long it took, success or failure)
- The `usbipd` commands run, their exit codes, and what changed in `state`
- The results of the precondition checks

**R13.8**: The log must be reachable from the interface. Rotate it, so it cannot
grow without limit.

**R13.9**: The log must not contain anything that identifies a person. Aliases
and memos the user typed are not written to it.

---

## 14. Documentation rules

**R14.1**: Every document under `docs/`, and the README, exists as a pair: the
English `X.md` and the Japanese `X.ja.md`, linked to each other at the top.

**R14.2**: The two say the same thing. Changing one without the other is an
unfinished change. Section numbers and requirement numbers are the same in both,
so a cross-reference resolves in either.

**R14.3**: Comments and identifiers in the source are English.

**R14.4**: `CHANGELOG.md` is a single file, with each entry written as an `(EN)`
and `(JA)` pair.

**R14.5**: What goes under `docs/` is **what was decided** and the **facts**
behind it. How it was considered, what was rejected, and how things used to be do
not.

> With the path mixed in, a reader has to work out which part is the current
> specification before anything else. The reasoning answers "why is it like
> this", so it stays; it does not answer "how did you arrive at it". The history
> of changes is in Git and in `CHANGELOG.md`.

---

## 15. The feature set

What the application provides, and the section that defines each.

| Feature | Defined in |
| --- | --- |
| The USB device list (connected and attached alike) | §10.1 |
| Showing VID / PID / USB serial / the Windows name / bus id / port path | §3.2, §10.2 |
| Distinguishing the USB serial number from the board (transport / target) | §3.1 (R3.1), §10.1 (R10.12) |
| Showing `usbipd` state (shared / attached / the client) | §5.2 |
| Bind / unbind / attach / detach | §5.2, §5.3 |
| Choosing the WSL distribution to attach to | §6 |
| Identifying a target on request | §4.3, §4.6 |
| Identifying on arrival | §4.4 |
| Distinguishing identified from not identified | §4.2 (R4.4) |
| The user's own names and notes | §3.5, §7.1 |
| Automatic attach (by board ID / USB serial number / VID:PID / bus id) | §9 |
| Refusing to run a second copy | §13.3 (R13.10) |
| Staying in the notification area (closing does not quit) | §10.6 |
| Keeping the list in step with connections and disconnections | §8 |
| Detecting the display language, and choosing it by hand | §10.4 |
| Starting when the user signs in to Windows | §7.1 |
| Identify-all | §4.4 (R4.19) |
| The diagnostic log | §13.4 |

Target families that are not supported (Arduino, RP2040, STM32 and so on),
showing the symlinks inside WSL (§6), and user-defined query protocols are all
designed for as **additions** to the structure that is already there.

**R15.1**: Keep the identification routes (§4.1) and the probe implementations
separable, so that adding a family means adding a probe and nothing else (R4.14).

---

## 16. How it is expected to be used

```text
a USB device is plugged into Windows
        ↓
the application detects it (§8)
        ↓
the USB identity is read (CfgMgr32)
        ↓
it appears in the list as a transport (VID/PID, serial number, port, Windows name)
        ↓
  ┌─ a family whose target can be identified ──→ shown as "not identified"
  │                                              the user presses identify → a probe
  │                                              (or automatic identification runs on arrival)
  │                                                   ↓
  │                                              the target is shown (held until disconnect, §4.2)
  │
  └─ anything else ───────────────────────────→ handled as a transport only
                                                 filled in with an alias or memo (§3.5)
        ↓
the usbipd state is read (usbipd state)
        ↓
attached to WSL by the user, or by an automatic attach
        ↓
WSL sees the USB device
        ↓
what already exists inside WSL runs (udev / board-identify)
        ↓
the symlinks are created
        ↓
the application reads and shows the WSL-side state where it is useful
```

---

## 17. References

- [research-findings.md](research-findings.md) — the measured facts
- [identification-policy.md](identification-policy.md) — the design of the identification method
- [release.md](release.md) — the release procedure
- [board-identify](https://github.com/tanakamasayuki/board-identify) — the identification tool on the WSL side
- [ch32rv](https://github.com/ch32-riscv-ug/ch32rv) — the WCH-Link probe implementation
- [usbipd-win](https://github.com/dorssel/usbipd-win)
