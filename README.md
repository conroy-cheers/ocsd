# OCSD
This crate provides utilities to enable reporting and monitoring of OCSD sensor values
from the host OS.

Credit to [ilo4_unlock](https://github.com/kendallgoto/ilo4_unlock) which made this
reverse-engineering effort possible.

## Disclaimer
*Protocol documentation for OCSD is not publicly available.
All of this has been reverse-engineered from a single ML350 Gen9. It has not been
tested on any other hardware! I cannot guarantee that it fully complies with the
OCSD protocol, and results on your server may vary.*

## What's OCSD?
Some HPE servers from Gen8 onwards are equipped with a feature called
Option Card Sensor Data (OCSD), also referred to as "Sea of Sensors".
OCSD allows temperature data to be sent from option cards (e.g. RAID controller
daughterboards, PCIe expansion cards) to iLO/BIOS for fan control and monitoring.

## OK, cool, so why do we need this crate?
Ordinarily, supported option cards will directly report temperatures via OCSD
without any involvement from the host OS, and the server will respond by controlling
the fans accordingly.

In the case of unsupported option cards, the server may do one of the following:
- Assume that the card is running *very hot* and spin up the fans to deafening levels
  at all times
- Ignore the card's existence entirely. In the case of passively cooled cards (e.g.
  unsupported server GPUs), this leads to thermal throttling due to insufficient fan
  speed at high load.

Ideally, when installing an unsupported option card, we would just modify its firmware
to report temperatures directly to the OCSD buffer. Unfortunately, this would be
really difficult.

As an alternative, this crate allows the host OS to take the reported temperatures
available from existing drivers, and forward them to the OCSD buffer so they can
be used by the iLO controller for reporting and fan control.
It also allows for reading reported temperatures for supported devices directly out
of the OCSD buffer, although there are probably better ways of getting that data.
