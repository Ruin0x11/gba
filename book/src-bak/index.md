# Ch 3: Memory and Objects

Alright so we can do some basic "movement", but we left a big trail in the video
memory of everywhere we went. Most of the time that's not what we want at all.
If we want more hardware support we're going to have to use a new video mode. So
far we've only used Mode 3, but modes 4 and 5 are basically the same. Instead,
we'll switch focus to using a tiled graphical mode.

First we will go over the complete GBA memory mapping. Part of this is the
memory for tiled graphics, but also things like all those IO registers, where
our RAM is for scratch space, all that stuff. Even if we can't put all of them
to use at once, it's helpful to have an idea of what will be available in the
long run.

Tiled modes bring us three big new concepts that each have their own complexity:
tiles, backgrounds, and objects. Backgrounds and objects both use tiles, but the
background is for creating a very large static space that you can scroll around
the view within, and the objects are about having a few moving bits that appear
over the background. Careful use of backgrounds and objects is key to having the
best looking GBA game, so we won't even be able to cover it all in a single
chapter.

And, of course, since most games are pretty boring if they're totally static
we'll touch on the kinds of RNG implementations you might want to have on a GBA.
Most general purpose RNGs that you find are rather big compared to the amount of
memory we want to give them, and they often use a lot of `u64` operations, so
they end up much slower on a 32-bit machine like the GBA (you can lower 64-bit
ops to combinations of 32-bit ops, but that's quite a bit more work). We'll
cover a few RNG options that size down the RNG to a good size and a good speed
without trading away too much in terms of quality.

To top it all off, we'll make a simple "memory game" sort of thing. There's some
face down cards in a grid, you pick one to check, then you pick the other to
check, and then if they match the pair disappears.

## Drawing Priority

Both backgrounds and objects can have "priority" values associated with them.
TONC and GBATEK have _opposite_ ideas of what it means to have the "highest"
priority. TONC goes by highest numerical value, and GBATEK goes by what's on the
z-layer closest to the user. Let's list out the rules as clearly as we can:

* Priority is always two bits, so 0 through 3.
* Priority conceptually proceeds in drawing passes that count _down_, so any
  priority 3 things can get covered up by priority 2 things. In truth there's
  probably depth testing and buffering stuff going on so it's all one single
  pass, but conceptually we will imagine it happening as all of the 3 elements,
  then all of 2, and so on.
* Objects always draw over top of backgrounds of equal priority.
* Within things of the same type and priority, the lower numbered element "wins"
  and gets its pixel drawn (bg0 is favored over bg1, obj0 is favored over obj1,
  etc).
