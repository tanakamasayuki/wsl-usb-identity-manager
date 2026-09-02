# WSL USB Identity Manager

*[English](README.md) | [日本語](README.ja.md)*

Track which physical device is which when forwarding USB devices to WSL with
[usbipd-win](https://github.com/dorssel/usbipd-win).

> **Status: specification phase.** Nothing is implemented yet.
> See [docs/requirements.ja.md](docs/requirements.ja.md) for the requirements
> (design documents are written in Japanese).

## The problem

Forwarding USB devices to WSL works, until you have more than one of the same adapter.

- **A CH340 reports no serial number.** With three of them plugged in, nothing in the
  USB descriptors tells them apart. Windows can only say which port each one sits in.
- **The bus ID is not a stable name.** `usbipd`'s bus ID is `Hub_#NNNN` plus
  `Port_#MMMM`, and the hub number is assigned in the order Windows happens to
  enumerate hubs. Plug a dock in later and every device behind it gets a different
  bus ID — the same bus ID now points at a different device.
- **COM numbers and `/dev/ttyUSB*` move too**, for the same reason.
- **An adapter or a probe names itself, not what is behind it.** A CH340's serial
  number — when it has one — belongs to the adapter, not to the board it is wired to.
  A WCH-LinkE's serial number belongs to the probe; swap the board on its debug pins
  and that serial does not change.

So "attach the CH340 to Ubuntu" is not a question this tooling can answer today.

## What this tool identifies

When you say "that device", you usually mean the board — not the adapter in front of
it. There are up to three separate things on one cable:

| | Example | How it is identified |
| --- | --- | --- |
| **Port** | bus ID, `COM8`, `/dev/ttyUSB2` | physical topology path — never persisted |
| **Transport** | CH340, CH343, WCH-LinkE | USB descriptors, when a serial number exists |
| **Target** | ESP32-S3, CH32X035 | by asking the board itself |

**Identifying the target — the board behind the adapter or probe — is the point of
this tool.** For a board with native USB the transport and the target are the same
thing, and its serial number is already enough. For everything else, the board has to
be asked.

### Currently supported targets

| Family | Identified by |
| --- | --- |
| **ESP32** (behind a CH340, CP210x, …) | eFuse MAC and chip type |
| **CH32 RISC-V** (behind a WCH-Link / WCH-LinkE) | part UUID and chip signature |

Anything else — Arduino, RP2040, STM32, or a bare adapter with an unknown board on it
— is identified down to the transport only, and says so rather than pretending
otherwise. Adding a family is meant to be additive.

## The approach

Separate **where a device is plugged in** from **what device it is**.

| | Examples | Persisted |
| --- | --- | --- |
| Runtime connection | bus ID, COM number, `/dev/ttyUSB0`, attach state | never |
| Identity | USB serial, board ID read from the target, your own label | yes |

Anything that cannot be pinned down from USB descriptors is identified by **asking the
target board itself** — the same conclusion
[board-identify](https://github.com/tanakamasayuki/board-identify) reached on the Linux
side, and this tool uses the same identifier format so both agree on names.

Asking the board disturbs it: reading an ESP32's eFuse MAC restarts the firmware, and
attaching to a WCH-Link halts the target core. So probing is not something this tool
does on a timer. It probes when you press the button, or — if you turn the option on —
in a short window right after a device is plugged in, when nothing is using it yet.
Everything else is answered from cache, and every device carries a visible confidence
level so a guess is never shown as a fact.

## Scope

This tool owns the Windows side: enumeration, identification, `usbipd` state and
operations, the WSL distribution to attach to, and your own labels and notes.

It does **not** manage anything inside WSL — no udev rules, no device node permissions,
no symlinks. Those stay with the existing Linux-side tooling. When the information is
available, it is displayed, never modified.

## Requirements

- Windows 10 or 11 (x64)
- [usbipd-win](https://github.com/dorssel/usbipd-win) 5.x
- WSL 2
- WebView2 Runtime — included in Windows 11, and already present on the vast majority
  of Windows 10 machines

## Installation

Planned once the first release is cut:

```console
winget install <package-id>
```

A portable ZIP will also be published on the releases page for people who would rather
not install anything.

## Documentation

Design documents are in Japanese.

- [Requirements](docs/requirements.ja.md)
- [Research findings](docs/research-findings.ja.md) — measurements this design is based on
- [Identification policy](docs/identification-policy.ja.md)
- [Platform evaluation](docs/platform-evaluation.ja.md)

## Related projects

- [board-identify](https://github.com/tanakamasayuki/board-identify) — the Linux-side
  counterpart, publishing stable symlinks inside WSL
- [ch32rv](https://github.com/ch32-riscv-ug/ch32rv) — WCH-Link probe implementation
- [usbipd-win](https://github.com/dorssel/usbipd-win) — the USB/IP host this tool drives

## License

MIT. See [LICENSE](LICENSE).
