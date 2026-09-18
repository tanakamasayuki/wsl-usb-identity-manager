# Identification policy

*[English](identification-policy.md) | [日本語](identification-policy.ja.md)*

Assumes: [research-findings.md](research-findings.md)

This document sets out how the application decides which device is which
physical unit. The requirements themselves are in
[requirements.md](requirements.md) §4.

---

## 1. The problem to solve

Established by measurement F3:

> From what Windows offers alone, several CH340s with no serial number cannot be
> told apart by anything beyond which port each one is in.

Identifying a unit therefore means probing it — asking the thing itself. But as
measurement F8 shows, probing has side effects.

- `esptool` and its kin drive DTR/RTS, so **the firmware restarts**
- Attaching to a WCH-Link **halts the target core**
- Serial output during a probe is lost

**"Probe in order to identify" and "do not probe" collide head-on.** How that
collision is handled is the centre of this application's design.

---

## 2. The policy

> **An identification is only ever what was asked and answered just now.
> The present is never inferred from a past answer.**

There are two moments when probing is safe.

1. **When the user asks for it** — an action taken knowing the side effects
2. **Just after a device is plugged in** — nothing has started using it yet, so a
   reset costs little

Outside those two, **nothing is probed at all.**

And for anything that has not been probed, **no identity is shown.** A past
answer is not reused to offer a "probably this one".

### 2.1 Why matching against a cache is not used

"Record the port and the VID/PID, then match against them next time and skip the
probe" does not work. It cannot detect this:

```text
recorded:  CH340-α (last in port1)   CH340-β (last in port2)
now:       one device in port1       one device in port2
actually:  α and β were swapped
```

The set of ports now and the set of ports recorded are identical, so the match
succeeds for both and **both are shown under the other one's name.** This is not
an implementation flaw but the limit of having nothing to go on but port
position
([F3](research-findings.md#f3-a-device-with-no-serial-number-cannot-be-identified-from-windows-alone)).

A USB-serial adapter can be moved to a different board **without one byte
changing on the USB side.** "The board that was behind this port, or behind this
serial number, last time" does not imply the board behind it now.

Attaching a confidence level (`confirmed`, `probable`, …) to a "probably this
one" is not done either.

- A confidence level **moves the chance of being wrong onto the user**; it does
  not make the answer less wrong
- A name that is on screen is treated as that device, near enough always
- And while a name is on screen, there is no reason to press identify

**Showing the wrong name confidently is worse than showing nothing.** With
"not identified" on screen, one press settles it when it matters (identify-all,
R4.19).

What follows from this is that **nothing observed is saved as fact**
(requirements §7.1). Every confirmed value is observed again.

What is saved is what the user decided, and what the screen marks as a reminder
(R4.21). The judgement above — never show the wrong name confidently — is met by
putting the confidence into the display, not by withholding the value.

---

## 3. The three routes to an identity

### Route 1: the USB serial number identifies the transport (no probe)

When a device reports a serial number, the third element of its instance id is
an identifier for **that USB device itself**.

```text
USB\VID_1A86&PID_55D3\5B5F090816            CH343
USB\VID_303A&PID_1001\70:04:1D:DA:86:F0     ESP32-S3 native USB (the MAC is the serial)
USB\VID_2341&PID_0043\8573531333335160D1C2  Arduino Uno
```

On a native-USB part such as an ESP32-S3 or C3 the board is the USB device, so
**this route identifies it completely** — a probe is only needed for the older
ESP32s reached through a CH340 or similar.

A CH343's or a WCH-Link's serial number, on the other hand, belongs to **the
adapter or the probe itself**. It says nothing about the board behind it
(requirements §1.2).

→ **A serial number is shown as the transport's identifier and never treated as
the target's** (R4.1).

### Route 3: VID:PID and the serial number identify the target (no probe)

Route 1 stops at the transport because **there is no telling whose serial number
it is**. Once the VID:PID is known to be the board's own, that stops being true.

```text
USB\VID_2341&PID_0069\34B7DA65B1C8   Arduino UNO R4 Minima
USB\VID_1A86&PID_7523\(no serial)    a CH340 — a cable, with no telling what is on the end
```

The table it is decided against is taken from board-identify (R4.24). **Stock
bridge IDs are always refused** (R4.25): a CH340 or CP2102 pair names the cable,
and so does the serial number that comes with it.

This route sends the device nothing, so it is not subject to the triggers in §6.
It is evaluated for every device on every refresh.

### Route 2: a probe identifies the target (side effects)

The only way to learn what is behind an adapter or a probe. It runs only at the
moments in §6.

Where routes 2 and 3 could both answer, route 2 wins (R4.26): a value read from
the silicon outlives a reflash that rewrites the descriptors.

---

## 4. How long an identity lasts

An identity is held **only while its device stays plugged in** (R4.3).

| Event | The confirmed identity | The reminder | Why |
| --- | --- | --- | --- |
| Attach to / detach from WSL | **kept** | kept | what is on the end of the cable has not changed |
| The application restarts | gone | **kept** | a confirmed identity is not saved (R7.8) |
| **Physically unplugged** | **dropped** | kept | the one moment the far end can change |

Unplugging is the only moment at which the far end of the cable can be swapped
leaving no trace on the USB side. Carrying an identity across it **as a fact**
would turn it into **a guess wearing the clothes of a fact**.

The reminder (the last identification, R4.21) survives because it is not wearing
those clothes: it is greyed, dated, and shown apart from a confirmed value. It
has to survive a restart because an attached device cannot be probed at all (F4),
so without it a machine that starts with its boards already forwarded can answer
nothing (R4.23). It is never used to decide an automatic attach (R4.22).

The interface distinguishes identified from not identified (R4.4). "Not
identified" means "not asked", not "unknown device".

---

## 5. Probe precedence

With several families supported, more than one probe can apply to one device. A
WCH-Link has a serial port of its own, so to a generic serial probe it is
**indistinguishable from an adapter with an ESP32 behind it.**

Each probe therefore declares whether it can **recognise its device from the
VID/PID** (R4.17).

| Kind | Meaning | Example |
| --- | --- | --- |
| By identifier | The VID/PID says what the hardware is; decided without touching anything | CH32 family (WCH-Link) |
| Fallback | Only the shape matches; what is behind it is unknown until asked | ESP32 family (a COM port) |

If a probe that recognises by identifier claims the device, the fallback probes
are not run (R4.18). "Claims" includes claiming it and being unable to proceed —
an ARM-mode WCH-Link, say. **An ARM-mode WCH-Link is still a WCH-Link, and still
not something to send an ESP32 reset and sync sequence to.**

> **The fallback must not carry an exclusion list.**
> Expressed as "VID/PIDs the ESP32 probe skips", that list grows with every
> family added. Expressed as precedence, a new family only has to name its own
> VID/PID.

---

## 6. When a probe runs

### 6.1 On request

The **identify** action, in the device list and in the detail pane.

- Probes the moment it is pressed
- States the side effects first — that the board restarts, and so on. Skipping
  that warning is a setting
- **Identify-all** covers every connected device that has not been identified
  (R4.19)

Identify-all is how the user picks up in one action what automatic
identification leaves out: adapters that report a serial number, and WCH-Links.

### 6.2 Just after a device is plugged in

Probes automatically, but only within a short window after the arrival. On by
default.

**Why that moment**: a board just plugged in is usually not in use yet, so a
reset does little harm.

| Setting | Default | Meaning |
| --- | --- | --- |
| Automatic identification | on | the feature as a whole |
| What it covers | only devices with no serial number | a serial number is usually enough on its own (route 1) |
| Grace window | 10 seconds from the arrival | after that, nothing runs by itself |
| Excluded VID/PID | listed by the user, empty by default | naming the hardware that must not be disturbed |

**Why on by default**: since identities are not saved, making the user press a
button on every plug-in would not be usable. It happens automatically at the
moment the side effects cost least, and anything that must be left alone can be
named.

**Why an exclusion list and not an allow list**: a CH340's VID/PID says nothing
about whether a dev board or a router console is on the other end. An allow list
ends up naming every common bridge anyway — the same effect as an exclusion
list, reached the long way round.

#### Retrying inside the grace window

There is a gap between a device becoming visible to Windows and its COM port
being assigned. Deciding once, at the instant it arrives, loses that race more
often than not. **Inside the grace window the attempt is repeated until the
probe can run** (R4.20). What bounds the attempts is the window, not the first
try.

### 6.3 Where a probe must never run

- Periodic polling
- Refreshing the list
- A sweep of everything at startup
- Deciding an automatic attach

### 6.4 One at a time

**Two probes never run against the same device at once**, and identify-all works
through its queue one device at a time. Two probes fighting over one adapter
either both fail or cut each other off partway.

---

## 7. Worked scenarios

### Three CH340s, each wired to a different board

**On plugging them in**

1. All three appear as transports (`VID_1A86&PID_7523`, differing only by port)
2. None reports a serial number, so all three qualify for automatic
   identification
3. They are probed one after another, settling as `esp32-7cdfa1123456` and so on
4. The board column fills in. **The device name stays as Windows reports it**
   (R10.7)

**With automatic identification switched off**

1. All three read "not identified"
2. The identify action on the row, or identify-all, settles them

**After swapping the ports**

The identity was dropped the moment each was unplugged. Plugged back in, it is a
new unidentified device, and identifying it again gives the right answer.
**There is no swap to detect in the first place.**

### An ESP32-S3 (native USB)

It reports a serial number (its MAC) and the board is itself the USB device, so
identifying the transport identifies the target. **It is never probed.**

### A CH32V305 behind a WCH-Link

1. The WCH-Link reports a serial number, so it is outside automatic
   identification
2. The board column reads "not identified", with the CH32 probe offered
3. Identify halts the target core, reads the part UUID, and settles an
   identifier of the form `<chip type>-<part UUID>`
4. If the WCH-Link is in ARM mode, that is reported and the switch is left to
   the user. **The application does not change its mode** (R4.15)

---

## 8. How this maps to the requirements

This policy gives an implementable form to the following requirements in
[requirements.md](requirements.md).

| Requirement | Where it is handled here |
| --- | --- |
| R4.1 a serial number identifies the transport | §3, route 1 |
| R4.2 identities are not stored against a port or a serial number | §2.1 |
| R4.3 an identity lasts only while the device is connected | §4 |
| R4.4 identified is distinguished from not identified | §4 |
| R4.5 the two moments a probe is allowed | §2, §6.1, §6.2 |
| R4.6 where probing is forbidden | §6.3 |
| R4.8 automatic identification on arrival | §6.2 |
| R4.9 nothing is sent to an excluded VID/PID | §6.2 |
| R4.17 / R4.18 probe precedence | §5 |
| R4.19 identify-all | §6.1 |
| R4.20 retrying inside the grace window | §6.2 |
