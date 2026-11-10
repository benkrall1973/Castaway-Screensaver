# Castaway_Screensaver V1.0.3 for Windows

This folder contains the Castaway_Screensaver application package:

- `Castaway_Screensaver.exe` — configure and preview the screensaver.
- `Castaway_Screensaver.scr` — installable Windows screensaver.
- `Castaway_Screensaver.ini` - saved settings, Favorites, and editable scene text.
- `INSTALL.md` - this guide.

## Install

1. Extract this folder to a permanent location. Keep these four files together.
2. Create a `data` folder beside them. From a lawfully obtained, already
   installed or extracted Johnny Castaway folder, copy **all** files into
   `data`. This also works for an authorized Internet/archive download once it
   has been extracted into an installation folder. The folder must contain
   `RESOURCE.MAP` and `RESOURCE.001`; keeping the original sound and support
   files with them is recommended. Do not use the legacy 16-bit installer
   executable by itself.
3. Double-click `Castaway_Screensaver.exe` to configure display options and preview
   changes.
4. Right-click `Castaway_Screensaver.scr` and choose **Install**.
5. In Windows Screen Saver Settings, select **Castaway_Screensaver**.

Windows stores the selected SCR path and normally runs it from this folder. If
you move the entire folder later, right-click the SCR and choose **Install**
again. Do not move only the SCR to `System32`: it needs its sibling EXE and INI.

## Configure

Use `Castaway_Screensaver.exe` for all settings. In Windows Screen Saver Settings,
the **Settings...** button opens the same configuration application in a window.
Changes save immediately to `Castaway_Screensaver.ini` beside the EXE and SCR.

The actual SCR behaves like a normal screensaver: mouse movement, a click, or a
key press closes it. That is intentional.

## Original data

This public package does **not** include Sierra/Dynamix resource files. Castaway_Screensaver needs a legally obtained original installation folder to display the
animation. Copy the folder's complete contents into the `data` subfolder
described above, or place them beside the EXE/SCR. Do not redistribute those
resource files without the required rights.

## License and attribution

Castaway_Screensaver is GPL-3.0-or-later software and a fork of
[Wilson Reborn by Antigerme](https://github.com/antigerme/wilson-reborn).
The complete matching source for this release is included in this repository.
The original Johnny Castaway artwork, animation, music, and sound data remain
copyright Sierra On-Line / Dynamix.
