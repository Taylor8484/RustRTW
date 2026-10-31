Wonderland
==========

A port of Return to Wonderland Platinum to a new engine for Windows 10+
and Linux. It plays Return to Wonderland's levels and those of The
Original Wonderland.

THE GAME DATA IS NOT INCLUDED. Levels, graphics, music and sound belong to
Midnight Synergy. You need your own copy of the games.


Setting up
----------

ASSETS.txt lists the files to copy from your games. They go in the
assets/RTWP and assets/TOW folders beside the wonderland program. TOW is
optional. Then start the program.


Playing
-------

Arrow keys move, and Esc opens the in-game menu.

A controller also works (Xbox names here, PlayStation and Nintendo in the
same places, except that Nintendo confirms with A on the right):

    D-pad or left stick         move
    right stick                 look around
    Y                           switch between players
    A / B                       confirm / back
    Start                       in-game menu
    X                           delete a player
    LB, RB / LT, RT             custom level pages / sort

Only Xbox controllers have been tried. PlayStation and Nintendo controllers
are untested, so please report how they behave on the forums.

If the game does not start on Windows, look for wonderland-error.txt beside
the program.

Options:

    --windowed                  stay in a window
    --pillarbox                 keep the original's 4:3 picture
    --backend dx12|vulkan|gl    choose the graphics API
    --assets <folder>           use another game data folder

On Linux the game needs the ALSA and udev libraries, and Vulkan or OpenGL
graphics drivers.


Licences
--------

license.txt holds the terms of the Wonderland source code this port is
built on. The port is free and may not be sold, and may only be shared
on the Wonderland community forums. Use it at your own risk: community
code is not checked by Midnight Synergy.

Controller icons are Input Prompts by Kenney (www.kenney.nl), CC0.

THIRD-PARTY.txt lists the open source libraries in this program, with
their licences. The default interface font is Fira Mono (SIL Open Font
License 1.1).
