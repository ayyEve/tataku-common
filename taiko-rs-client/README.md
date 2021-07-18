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
 - map backgrounds in-game
 - scrollable area dragger
  
- // Gameplay
 - letter ranking
 - spectator
 - multiplayer (oh boy lmao)
 - online leaderboard
 - online replays (should come with ^, might be best to make an online_score_menu menu to distinguish between local and online scores)

- // New Audio Engine
 - skip intro
 - handle headphones being unplugged (might require a dropdown to select the output device)
 - store the currently playing music track in audio. should resolve a ton of issues relating to audio playing when it shouldnt, and also make volume tracking much easier
 - dont load all audio with the beatmap, only load it when it should be played, and unload it when not playing
 - hitsound volume *= timing section volume

- // Code
 - refactor code a bit to make it prettier
 - better error handling/messages
 - handle peppy direct download moment (might be best if notifs exist first)
 - separate replays from scores (would help loading)
 - depths that actually make sense
  
maybe todo:
 - profiler
 - read osu replays
 - mods (shouldnt be too bad for some)