My taiko sim made in rust. why? idk. enjoy!
  
required files/folders: (in game root)
 - fonts/main.ttf (Roboto was used in dev)
 - audio
    - don.wav
    - kat.wav
  
required deps:
 - cmake
 - clang (maybe not now that rustcord was nuked)
  

TODO:
- UI
- dropdown menu item
- notification system
- proper depths for scrollable item drawing
  
- order leaderboard by score
- order sets by title (oh god)
  
- // Gameplay
- letter ranking
- timing hit window during gameplay (oh god)
- background images? (requires some work)
- skip intro (needs new audio engine)
- replays
- static sv
- sv multiplier
  
- // Code
- clean up imports
- move common consts to main
- refactor code a bit to make it prettier
- better error handling/messages
- optimize many functions now that drawing has been reworked
- only load taiko/convert maps
- handle peppy download moment
  
maybe todo:
 - read osu replays
 - online leaderboard (needs dropdown first)
 - multiplayer
 - mods
