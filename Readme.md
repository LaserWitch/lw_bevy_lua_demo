# Repo status:

In addition to the notes below, this repository currently reflects extracted portions of a larger project workspace, and are undergoing revision. There's a somewhat excessive mess and lack of documentation as a result.

It also now is designed for compilation on wsl and may have filepath troubles if built directly in windows. Sorry.

# lw_luademo

This repo is my exploratory example usage of [bevy_mod_scripting](https://github.com/makspll/bevy_mod_scripting)(BMS), a scripting crate for the [Bevy engine](https://bevyengine.org/). BMS is somewhat language agnostic, but I am dealing specifically with Lua.

My intended usage deviates from the behavior in BMS example code in a few ways.
* BMS works via scripts attached to entities, isolated from other scripts. I want 'global' scripts that inter-operate within the Lua environment.
* BMS wipes out the script state on a reload, I want to be able to retain stateful scripted behavior while live-developing them.
* I want to use the `require` package loading feature of Lua to access scripts already loaded into the game's asset manager, rather than relying on file system access as BMS currently supports.

To figure these things out I set up a simple pseudo-gameplay system and worked towards replacing it with a Lua script. This has been accomplished. I'm posting this to get feedback on my implementation, but anyone wanting to use it as a starting point for their own use projects, under the customary Rust MIT/Apache dual licensing. Please be aware this is pretty rough code in my opinion, there's many places where I just bashed together something that worked at the moment so I could focus on something else.

## Fennel
One complication is I want to use [Fennel](https://fennel-lang.org/) alongside Lua. Fennel is a lisp compiled in and to Lua, so it interoperates freely with Lua functions.

This is mostly just because I like writing fennel, and I think it's compiler is useful for catching potential errors. I have both lua and fennel example files included in the assets dir.

Right now the fennel compiler is included in the assets directory, but in time I'll probably inline it further and work on error reporting and debugging output.
