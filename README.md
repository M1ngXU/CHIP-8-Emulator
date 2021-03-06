# CHIP-8-Emulator
This emulator supports [CHIP-8](https://en.wikipedia.org/wiki/CHIP-8) and [SUPERCHIP](https://groups.google.com/g/comp.sys.handhelds/c/sDY9zFb6KUo/m/JcYBK2_yerMJ) games.

# Installation
Since Sdl2 is used, you'll need the [Sdl2 runtime library](https://www.libsdl.org/download-2.0.php) for you system next to your executable file.
You can find the executable file either in the [releases](https://github.com/M1ngXU/CHIP-8-Emulator/releases/) or you have to build it with `cargo build --release`.

# Emulate a game
In a terminal, run `chip8-emulator.exe path/to/binary`. You can find many binaries [here](https://github.com/badlogic/chip8/blob/master/roms/).

Before the first run starts, it might take some time for the antivirus-software to check everything. Before the first run, a `saves` folder is created for quicksaves.

# Keyboard Layout
Since all input is in `hex`, the following keyboard layout is used with ([Scancodes](https://en.wikipedia.org/wiki/Scancode) for the US keyboard):

| 1 | 2 | 3 | 4 |
| --- | --- | --- | --- |
| Q | W | E | R |
| A | S | D | F |
| Z | X | C | V |

which is interpreted as:

| 1 | 2 | 3 | C |
| --- | --- | --- | --- |
| 4 | 5 | 6 | D |
| 7 | 8 | 9 | E |
| A | 0 | B | F |

For the emulator, there are the following Keyboard-Shortcuts:

| Scancode | Description |
| --- | --- |
| Esc | Pauses/Unpauses the game. |
| F1 | Resets speed to 100%. |
| F2 | Increases the emulation speed by 20%. |
| F3 | Decreases the emulation speed by 20%. |
| F4 | Enter/Leaves the cheat mode. |
| F5 | Quicksaves the emulation state (to the `saves` folder). |
| F8 | Quickloads the newest quicksave (from the `saves` folder). |

# Cheat Mode
Some games depend on collision detection (like [breakout](https://github.com/badlogic/chip8/blob/master/roms/breakout.rom)), so in the `cheat mode`, drawing onto the screen is possible.

The left mouse button turns a pixel `on`, while the right mouse button turns it `off`. While drawing the game is paused.

# Bugs
If you think you encountered a bug, you can open an issue. Make sure to include a bin/savegame and the OS.
