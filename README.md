My taiko sim made in rust. why? idk. enjoy!
Join our Discord server! https://discord.gg/PGa6XY7mKC

required deps:
 - windows:
   - cmake

 - linux (some may be incorrect, i'll double check when i have time)
   - gcc
   - cmake
   - libasound2-dev
   - pkg-config
   - libssl-dev
   - xorg-dev
   - libxcb-shape0
   - libxcb-render0
   - libxcb-fixes0

How to build:
 - install rust (https://rustup.rs/)
 - add nightly toolchain (required until iter_mut is added to stable)
   - rustup toolchain add nightly
   - rustup override set nightly

 - switch to animations branch (optional but preferred)
   - git checkout animations

 - build and run
  - cargo run --release
   

TODO:
- // UI
 - dropdown menu item
 - notification system
  
- // Gameplay
 - letter ranking
 - spectator
 - multiplayer (oh boy lmao)
 - online leaderboard
 - online replays (should come with ^, might be best to make an online_score_menu menu to distinguish between local and online scores)

- // New Audio Engine
 - handle headphones being unplugged (might require a dropdown to select the output device)

- // Code
 - better error handling/messages
 - handle peppy direct download moment (might be best if notifs exist first)
 - depths that actually make sense
 - make renderables a param instead of returning a new list
 - pass the whole keys list instead of one key at a time
  
maybe todo:
 - profiler
 - read osu replays
 - mods (shouldnt be too bad for some)