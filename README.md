my taiko sim made in rust. why? idk. enjoy!

required files/folders: (in game root)
 - fonts/main.ttf (Roboto was used in dev)
 - audio
    - don.wav
    - kat.wav

required deps:
 - cmake
 - clang

TODO:
timing hit window during gameplay (oh god)
dropdown menu item
letter ranking

order leaderboard by score
order sets by title
order maps by diff

background images? (requires some work)
skip intro (needs new audio engine)
replays

proper depths for scrollable item drawing
optimize many functions now that drawing has been reworked

clean up imports
move common consts to main
refactor code a bit to make it prettier

only load taiko/convert maps

handle peppy download moment


maybe todo:
 - read osu replays
 - online leaderboard (needs dropdown first)
 - multiplayer
 - mods