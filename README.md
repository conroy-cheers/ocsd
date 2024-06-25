# OCSD

Rust library for reading and/or writing OCSD temperature reports from userspace
on compatible HPE servers.

Credit to [ilo4_unlock](https://github.com/kendallgoto/ilo4_unlock) which made this
reverse-engineering effort possible.

On ML350 Gen9, `cat /proc/iomem` yields:
```
791ff000-7b5fefff : ACPI Non-volatile Storage
7b5ff000-7b7fefff : ACPI Tables
7b7ff000-7b7fffff : System RAM
```

According to the `ocsd header` command, the OCSD buffer starts at `0x791f6000`,
which is inside the "ACPI Non-volatile Storage" region.
