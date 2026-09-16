# Measured facts

*[English](research-findings.md) | [日本語](research-findings.ja.md)*

Measured on: Windows 11 Pro 26200.9168 / usbipd-win 5.3.0 / WSL 2.7.12.0 (Ubuntu-24.04)

This document records the facts the specification rests on, with the values they
were measured from. The specification itself is in
[requirements.md](requirements.md).

---

## F1. A usbipd bus id is a dynamic Windows hub number, not a physical position

### Measured

`usbipd list` on one machine, before and after a dock was plugged in.

Without the dock:

```text
2-5    30c9:0050  Integrated Camera ...
2-7    27c6:6594  Goodix MOC Fingerprint
2-10   8087:0033  Intel(R) Wireless Bluetooth(R)
```

With the dock, in the same session:

```text
2-5,  2-7,  2-10   (as above, plus)
13-1   1d5c:7102  Generic Billboard Device
14-4   045e:0039  Microsoft USB IntelliMouse Optical
15-2   05e3:0749  USB Mass Storage Device
15-4   0b95:1790  ASIX USB to Gigabit Ethernet
16-1   1a86:7523  USB-SERIAL CH340 (COM10)
16-2   1a86:7523  USB-SERIAL CH340 (COM11)
16-3   1a86:55d3  USB-Enhanced-SERIAL CH343 (COM7)
16-4   1a86:7523  USB-SERIAL CH340 (COM3)
```

Lined up against `DEVPKEY_Device_LocationInfo` for the same devices, the bus ids
match exactly.

| Device | LocationInfo | usbipd bus id |
| --- | --- | --- |
| Bluetooth | `Port_#0010.Hub_#0002` | `2-10` |
| Goodix | `Port_#0007.Hub_#0002` | `2-7` |
| Mouse | `Port_#0004.Hub_#0014` | `14-4` |
| Card reader | `Port_#0002.Hub_#0015` | `15-2` |
| Billboard | `Port_#0001.Hub_#0013` | `13-1` |
| CH340 #1 | `Port_#0001.Hub_#0016` | `16-1` |

### Conclusion

**A bus id is `Hub_#NNNN`, a hyphen, and `Port_#MMMM`.** `Hub_#NNNN` is a
running number Windows assigns in the order it enumerates hubs; it is not
physical topology. It changes with the order hubs are plugged in, with timing,
and across a restart.

→ **A bus id is a temporary handle, valid at the instant an operation is issued
and no longer. It must never be persisted.**

The bus id is not itself a lie. The problem is that **the same bus id comes to
mean a different device over time**, so issuing an operation against a recorded
one takes hold of something else.

---

## F2. The stable identifier of a physical port is DEVPKEY_Device_LocationPaths

### Measured

```text
CH340 #1  PCIROOT(0)#PCI(1400)#USBROOT(0)#USB(1)#USB(3)#USB(1)#USB(1)
CH340 #2  PCIROOT(0)#PCI(1400)#USBROOT(0)#USB(1)#USB(3)#USB(1)#USB(2)
CH343     PCIROOT(0)#PCI(1400)#USBROOT(0)#USB(1)#USB(3)#USB(1)#USB(3)
CH340 #3  PCIROOT(0)#PCI(1400)#USBROOT(0)#USB(1)#USB(3)#USB(1)#USB(4)
```

A chain from the PCI root through the host controller to the hub port, with no
dynamic number such as `Hub_#NNNN` anywhere in it.

### Conclusion

Stable for as long as the physical wiring is. **This is what identifies a port.**
It identifies a *port*, though, not a *device* — see F3.

---

## F3. A device with no serial number cannot be identified from Windows alone

### Measured: device instance ids

```text
USB\VID_1A86&PID_7523\8&7CC2A31&0&1        CH340 (no serial number)
USB\VID_1A86&PID_7523\8&7CC2A31&0&2        CH340 (no serial number)
USB\VID_1A86&PID_7523\8&7CC2A31&0&4        CH340 (no serial number)
USB\VID_1A86&PID_55D3\5B5F090816           CH343 (has one)
USB\VID_303A&PID_1001\70:04:1D:DA:86:F0    ESP32-S3 native USB (the MAC is the serial)
USB\VID_2341&PID_0043\8573531333335160D1C2 Arduino Uno
```

The third element of an instance id is **the serial number when there is one,
and otherwise `<hash from the parent hub>&0&<port number>`.** The latter
identifies a port.

### Measured: ContainerId

```text
CH340 #1 (no serial)   6aa62d84-4e6a-11f1-b04a-c0a5e85bd89d   UUID v1 (time based)
CH340 #2 (no serial)   6aa62d6a-4e6a-11f1-b04a-c0a5e85bd89d   UUID v1
CH340 #3 (no serial)   fb0cb475-9ab0-11f1-b07d-c0a5e85bd89d   UUID v1
CH343    (has one)     24309343-175d-5c84-a14b-bb9cc186c126   UUID v5 (name based)
Card rdr (has one)     1b62dd7d-6c9a-57f3-a424-5979ac2abcaa   UUID v5
AX88179  (has one)     e17ce95f-2c07-5b26-ae60-762714168e78   UUID v5
```

**The version field splits cleanly along that line.**

- With a serial number → **UUID v5**, a deterministic hash of VID/PID/serial. The
  same value follows the device to another port.
- Without one → **UUID v1**, generated on the spot at first connection and stored
  against the instance id, which is to say against the port.

### Conclusion

`ContainerId` looks like a per-device identifier, but **on a device with no
serial number it degenerates into a port identifier.** On a device that has one
it is an excellent key for tying the interfaces of a composite device together.

> **From what Windows offers alone, several CH340s with no serial number can
> never be told apart by anything beyond which port each one is in.**
> This is not a question of implementation skill: nothing in the USB descriptors
> distinguishes them.

→ Identifying a unit means **asking the thing on the other end — probing it.**
There is no other way. This is the conclusion board-identify reached on the
Linux side.

---

## F4. The join key with usbipd is InstanceId, not the bus id

What `usbipd state` returns:

```json
{
  "BusId": "16-1",
  "ClientIPAddress": null,
  "Description": "USB-SERIAL CH340 (COM10)",
  "InstanceId": "USB\\VID_1A86&PID_7523\\8&7CC2A31&0&1",
  "IsForced": false,
  "PersistedGuid": "05d75b37-ae96-4b2a-9d0d-5cded25f52eb",
  "StubInstanceId": null
}
```

- `InstanceId` matches the Windows PnP device instance id exactly.
  → **The only correct key for joining usbipd to a device model of our own.**
- `BusId` is `null` while the device is not connected, and meaningful only while
  it is.
- `PersistedGuid` is issued by usbipd at bind and kept while the device is away.
  It is an identifier internal to usbipd, and on a device with no serial number
  it ends up tied to the port.

### Tracking across an attach

The registry under `HKLM\SYSTEM\CurrentControlSet\Enum\USB` held:

```text
Vid_80EE&Pid_CAFE\58FA041040      VirtualBox USB Driver (VBoxUSB)
Vid_80EE&Pid_CAFE\5B5F090816      VirtualBox USB Driver (VBoxUSB)
Vid_80EE&Pid_CAFE\8&7cc2a31&0&1   VirtualBox USB Driver (VBoxUSB)
Vid_80EE&Pid_CAFE\8&7cc2a31&0&2   VirtualBox USB Driver (VBoxUSB)
```

An attached device is re-enumerated on Windows as `VID_80EE&PID_CAFE`, the
VBoxUSB stub, but **the third element of the instance id — the serial number or
the port path — carries over unchanged.** `usbipd state` returns it as
`StubInstanceId`.

→ Identity can still be tracked during an attach. But **Windows cannot reach the
device itself**, so it cannot be probed.

---

## F5. usbipd 5.3's own automation cannot tell units apart

```text
usbipd bind    --busid <BUSID> | --hardware-id <VID:PID>  [--force]
usbipd attach  --busid <BUSID> | --hardware-id <VID:PID>  --wsl [DISTRO]
               [--auto-attach] [--unplugged] [--host-ip <IP>]
usbipd policy  add | list | remove     AutoBind rules, by bus id or by VID:PID
usbipd state                           JSON; the only machine-readable interface
```

- What can be named is **a bus id or a VID:PID**, and nothing else.
- With three CH340s plugged in, `--hardware-id 1a86:7523` matches all three.
- `--auto-attach` attaches in connection order, so which `/dev/ttyUSB*` a board
  lands on is not fixed.

→ **usbipd on its own cannot solve this problem. That is what this application
is for.**

### Permissions

- `bind` / `unbind` / `policy`: administrator rights required
- `attach` / `detach` / `state` / `list`: an ordinary user can run them
- The `usbipd` service starts automatically and stays resident

### Warnings specific to an environment

```text
usbipd: warning: Unknown USB filter 'CsDeviceControl' may be incompatible ...
usbipd: warning: USB filter 'USBPcap' is known to be incompatible ...
                 'bind --force' will be required.
```

USBPcap is installed on this machine, so `bind --force` is needed in some cases
here. `--force` takes the device away from Windows, which **cannot coexist with
probing** (F6). The interface has to distinguish the two explicitly.

### Binaries that ship with it

```text
C:\Program Files\usbipd-win\usbipd.exe
C:\Program Files\usbipd-win\Usbipd.PowerShell.dll
C:\Program Files\usbipd-win\WSL\usbip              Linux ELF
C:\Program Files\usbipd-win\WSL\usbip-auto-attach  Linux ELF
C:\Program Files\usbipd-win\Drivers\VBoxUSB.sys
```

`WSL\usbip` is a Linux binary. Running `usbip port` through `wsl.exe` gives the
mapping between the vhci ports inside WSL and the bus ids on the Windows side.

---

## F6. A WCH-Link can be opened on its stock driver

### Measured: which driver is bound

```text
USB\VID_1A86&PID_8010\<serial>            usbccgp       composite parent
USB\VID_1A86&PID_8010&MI_00\...           WCHLink_A64   vendor interface (WCH's own)
USB\VID_1A86&PID_8010&MI_01\...           usbser        CDC (the COM port)
```

Every WCH-Link ever plugged into this machine has **`WCHLink_A64` on interface
0** — WCH's own driver. Only a unit put through Zadig / libwdi reads `WinUSB`.

```text
oem79.inf   wchlinkwdm.inf              wch.cn    WCH's own WDM driver
oem24.inf   wch-link_(interface_0).inf  libwdi    WinUSB, assigned by Zadig
```

### Measured: it probes on the stock driver

`ch32rv-usb` **falls back to the CH375-style `DeviceIoControl` path when nusb
cannot claim the interface.** Against a WCH-Link left on its stock driver,
`ch32rv probe list` returns the probe information (`WCH-Link(CH549)`, firmware
2.6).

### Conclusion

- **WinUSB is not a prerequisite.** A probe works on the stock driver.
- So the application never tells anyone to "assign WinUSB with Zadig". Asking for
  a driver swap costs the user an extra procedure and loses them the WCH-Link's
  COM port.
- `DEVPKEY_Device_Service` is **information to display**, not a test of whether a
  probe can run.
- A bound WinUSB opens without administrator rights too. WinUSB is a user-mode
  API, and libusb, nusb and pyusb all go through it.
- The CDC interface (interface 1) appears as an ordinary COM port.

---

## F7. Reaching Windows PnP through PowerShell is not usable

### Measured (same machine, 20 USB devices)

| Method | Time |
| --- | --- |
| PowerShell `Get-PnpDevice` + `Get-PnpDeviceProperty` | **timed out past 120 seconds** |
| `CfgMgr32` called directly through ctypes (`CM_Get_Device_ID_ListW` + `CM_Get_DevNode_PropertyW`) | **170 ms** |

More than 700 times. A resident application that re-enumerates on every hot-plug
cannot use PowerShell.

→ **Requirement**: enumeration and property reads call `CfgMgr32` / `SetupAPI`
natively. Starting a process is acceptable for `usbipd.exe` alone, because it
happens rarely.

Also noted: reading `HKLM\SYSTEM\CurrentControlSet\Enum\USB` directly is fast
too, and yields **every device ever connected** — including ones that are not
now — along with the COM number under `Device Parameters\PortName`.

---

## F8. How board-identify, the existing Linux-side tool, is designed

Python with uv. It uses a three-layer identity model.

| Layer | What it is | Example |
| --- | --- | --- |
| Port | a transient kernel node | `/dev/ttyUSB2` |
| Transport | the USB adapter or bridge | CH340, FTDI, WCH-Link |
| Target | the microcontroller board behind it | ESP32-S3, CH32X035 |

Identifier format: `<variant>-<unique-id>`

```text
esp32-s3-7cdfa1123456
ch32x035c8t6-1ff9abcd880ebc48
wch-link-fc928f068181
```

One port can carry several identifiers at once: the probe itself, and the board
behind it.

Its README also sets out why the standard mechanisms break under WSL + USB/IP.

- A Windows bus id does not exist inside Linux, so it cannot go in a udev rule
- A forwarded device appears on a virtual host controller, so `ID_PATH` and
  `/dev/serial/by-path/` come out as `platform-vhci_hcd.0-usb-0:1:1.0`, where the
  port number derives from **the order things were attached**
- `/dev/serial/by-id/` only works for adapters that report a serial number

### The side effects of probing

As its README states plainly, a probe **disturbs its target**.

- `esptool` drives DTR/RTS to drop into the bootloader, so **the firmware
  restarts**
- Attaching to a WCH-Link **halts the target core**
- Serial output during a probe is lost

→ **Requirement**: the interface must not probe on a timer. Probing is limited to
an explicit action, or to the one moment a device arrives.

---

## F9. wsl-usb-gui, the tool in use today

- Python with tkinter, made into a single exe with PyOxidizer, distributed as MSI
- `wsl_usb_gui/usb_monitor.py`, `wsl_usb_gui/win_usb_inspect/winusbclasses.py`
- Its auto-attach profiles are **VID/PID based**

→ That it cannot tell several CH340s apart is not a gap in the implementation but
**a consequence of looking only at VID/PID**. As F3 shows, on a device with no
serial number nothing finer than VID/PID is available from the Windows side at
all, so nothing short of probing solves it.

---

## What the measurements require of the design

1. **Never persist a bus id, a COM number or a `/dev/ttyUSB*`** (F1)
2. **Join to usbipd on InstanceId** (F4)
3. **Identify a physical port by LocationPaths** (F2)
4. **A device with no serial number can only be identified by probing** (F3)
5. **Probing is destructive, so it must not run continuously** (F8)
   → it runs on an explicit action, and in the grace window just after an arrival
6. **Enumerate through the native API** (F7)
7. **A WCH-Link works on WCH's own driver; never ask for a driver swap** (F6)
8. **An attached device cannot be reached from Windows** (F4)
   → so a probe can only happen before an attach
9. **bind needs administrator rights, attach does not** (F5)
   → so the interface as a whole is never run elevated

---

## References

- [board-identify](https://github.com/tanakamasayuki/board-identify)
- [usbipd-win](https://github.com/dorssel/usbipd-win) / [Automation wiki](https://github.com/dorssel/usbipd-win/wiki/Automation)
- [wsl-usb-gui](https://gitlab.com/alelec/wsl-usb-gui)
- [WinUSB Device (Microsoft Learn)](https://learn.microsoft.com/en-us/windows-hardware/drivers/usbcon/automatic-installation-of-winusb)
- [WCID Devices (libwdi wiki)](https://github.com/pbatard/libwdi/wiki/WCID-Devices)
- [wlink (ch32-rs)](https://github.com/ch32-rs/wlink)
