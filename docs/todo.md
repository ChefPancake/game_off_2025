# TODO

This is the remaining work for finishing the game

- [x] MVP
    - [x] Fix ghost spawning
        - It seems like the target ghost is not on the field at all
    - [x] Make firing the wave a click instead of the spacebar
    - [x] Pull in the new images
    - [x] Make sure that all clickable areas are now accurate
    - [x] Rebuild the new buttons
        - Things should be separated out now so they'll have to be dynamically constructed
    - [x] Auto-adjust camera scale with window resizes
    - [x] Get the inverter working
        - [x] Update the sprite based on state
    - [x] Track number of capture charges
        - [x] Count internally
        - [x] Display charges
        - [x] Lose the game when charges hits 0 and there are still targets on the field
        - [x] Win the game when all target ghosts are captured
    - [x] Count reputation for each ghost gotten
        - [x] Count internally
        - [x] Display reputation
        - [x] Going below 0 ends the game? maybe
    - [x] Make ghosts wander off the edges
    - [x] Lose the game if any target ghosts wander off the edge
    - [x] Lock up UI while ghosts are wandering
    - [x] Display target ghost in the window
    - [x] Remove debug boxes
    - [x] Respond to game win/lose
        - [x] Lock everything down and spawn the splash screen
    - [x] Get new frame
    - [x] Put backing 
- [ ] Ideal
    - [x] Reformat images to be smaller
    - [x] Randomize which button does what
    - [x] Handle 0th lane to not stick ghost behind the target
    - [x] Make loading screen
    - [x] Make Levels
        - [ ] Make the buttons do less to begin with, then ramp up
    - [x] Make ghost particles
        - [x] Make soul particle
        - [x] Make burst particles
    - [x] Make wave particles
        - [ ] Rumble the remote
    - [x] Add ghost shadows
        - Make sure they're on a lower z_pos than the ghosts so they never sit on top of each other
    - [ ] Add Z-sorting for ghosts in the field
        - make their z component a function of their y component
    - [x] Add flash for capture
    - [x] Add music
    - [ ] Add sounds
        - [ ] button clicking
        - [ ] switch flipping
        - [x] Turning on the device
        - [ ] Ghosts leaving
        - [ ] Ghosts captured
    - [ ] Add parallax to the remote

## Changing the rules

Right now, the game isn't fun. We have a limited number of capture charges
(default 3), but an infinite number of remote charges. We're going to flip this
around, so then the game is about discovering what the buttons do in the most
efficient way possible. This also gives more meaning to the strength dial.

Once levels/rounds exist, we can keep the reputation from the previous levels
while restoring the number of charges.

The changes necessary are:
- [x] Change default charges to 10
- [x] Modify the Run button to decrease the number of charges if there are any waveforms activated
- [x] Modify the Capture button to not decrease the number of charges
- [x] When losing, send the ghosts offscreen
