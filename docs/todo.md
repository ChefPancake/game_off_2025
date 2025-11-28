# TODO

This is the remaining work for finishing the game

- [ ] MVP
    - [ ] Fix ghost spawning
        - It seems like the target ghost is not on the field at all
    - [x] Make firing the wave a click instead of the spacebar
    - [ ] Pull in the new images
        - [ ] Make sure that all clickable areas are now accurate
        - [ ] Rebuild the new buttons
            - Things should be separated out now so they'll have to be dynamically constructed
    - [ ] Get the inverter working
        - [ ] Update the sprite based on state
    - [ ] Track number of capture charges
        - [x] Count internally
        - [ ] Display charges
        - [x] Lose the game when charges hits 0 and there are still targets on the field
        - [x] Win the game when all target ghosts are captured
    - [ ] Count points for each ghost gotten
        - [x] Count internally
        - [ ] Display points
        - [ ] Going below 0 ends the game? maybe
    - [x] Make ghosts wander off the edges
    - [ ] Lose the game if any target ghosts wander off the edge
    - [x] Lock up UI while ghosts are wandering
    - [ ] Display target ghost in the window
    - [ ] Remove debug boxes
    - [ ] Respond to game win/lose
- [ ] Ideal
    - [ ] Handle 0th lane to not stick ghost behind the target
    - [ ] Make loading screen
    - [ ] Make Levels
        - [ ] Make the buttons do less to begin with, then ramp up
    - [ ] Make ghost particles
    - [ ] Make wave particles
    - [ ] Track number of fired waves, dish out bonuses at level end
    - [ ] Add ghost shadows
        - Make sure they're on a lower z_pos than the ghosts so they never sit on top of each other
    - [ ] Add flash for capture
    - [ ] Add music
    - [ ] Add sounds
        - [ ] button clicking
        - [ ] switch flipping
        - [ ] Turning on the device
        - [ ] Ghosts leaving
        - [ ] Ghosts captured

