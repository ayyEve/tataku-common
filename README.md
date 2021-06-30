My taiko sim made in rust. why? idk. enjoy!
  
required files/folders: (in game root)
 - fonts/main.ttf (Roboto was used in dev)
 - audio
    - don.wav
    - kat.wav
    - bigdon.wav
    - bigkat.wav
  
required deps:
 - cmake
  

TODO:
- // UI
- dropdown menu item
- notification system
- proper depths for scrollable item drawing
- background images? (textures are borked atm)
- add back buttons to menus lol
  
- // Gameplay
- letter ranking
- timing hit window during gameplay
- replays
- skip intro (needs new audio engine)
  
- // Code
- refactor code a bit to make it prettier
- better error handling/messages
- handle peppy direct download moment (might be best if notifs exist first)
  
maybe todo:
 - profiler
 - read osu replays
 - online leaderboard (needs dropdown to select leaderboard mode)
 - multiplayer
 - mods (shouldnt be too bad for some)
 - handle headphones being unplugged (might require a dropdown to select the output device, but that would need a new audio engine)
 - online stuff