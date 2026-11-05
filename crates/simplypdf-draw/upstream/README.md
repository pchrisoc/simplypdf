# LibreOffice Draw UI reference

This directory contains Draw's classic menu bar and all 48 toolbar XML files under `sd/uiconfig/sdraw` from LibreOffice 26.8.0.3, commit `bce0998afefdbc355585ca324285661a2170ba77`. Paths here are relative to the upstream `sd` directory. Each file was verified byte-for-byte against its Git blob ID in the pinned tree. [Pinned UI source](https://github.com/LibreOffice/core/tree/bce0998afefdbc355585ca324285661a2170ba77/sd/uiconfig/sdraw)

The XML preserves menu ordering, nested menus, command IDs, separators, and toolbar visibility attributes. The Rust viewer does not load these definitions at runtime. Command implementations, localization, icons, context menus, and the Notebookbar layout are separate upstream sources and port tasks.

Original file headers are preserved. See the [crate license texts and attribution](../licenses) and the [port ledger](../../../docs/libreoffice-draw-port.md).
