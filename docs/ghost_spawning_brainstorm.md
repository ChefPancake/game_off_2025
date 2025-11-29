# Ghost Spawning Brainstorm
    
We know what the target ghost is, which is a unique pair of body and hat tags.
We should make sure that there is a variant of ghost that has the same body,
but a different hat as well as a variant that has the same hat but a different
body. Then we should make sure that there is another ghost for each of those
that differs only in the way that they are common to the target.

Ex: 
- Target: body1, hat1
- Var1: body1, hat2
- Var2: body2, hat1
- Var1.1: body3, hat2
- Var2.1: body2, hat3

Can maybe just start with the first 3 for "level 1", and add more variants as
we crank up the difficulty

...except that we need the buttons to all do something. Can we do enough with 5
buttons and 4 tags?

What if we reduced the number of buttons for lower difficulties? While we only
have the one level, we should make sure that all 5 buttons can have value, so
we'll probably have to start with the 5 variant scheme to begin with. Once
level-making exists, then we can dial it back and ramp things up as the levels
progress.

6 tags, 5 buttons. At least two buttons need to have 2 tags, so long as we
maintain that one of the tags in these pairs doesn't appear in any other button

This should probably just be hardcoded at this point. The tags that are actually
selected can be the thing that changes.

- b0: body1, hat3
- b1: body1, hat2
- b2: body2
- b3: body3
- b4: hat1

