# Peregrine

This is a space-themed video game made by Jack Dinsmore.

It's coded in Rust using the Tethys engine, which I also made. This engine is built on WebGPU (cross-platform), which I highly recommend. It is not designed to work on the web, though in principle it could.

## Description

The game is inspired by Space Engineers, Rogue Systems Hardware, Children of a Dead Earth, KSP, Factorio, Starbound, and several other games. It attempts to be a space sim set in a sci-fi world in which you can design your own ships and fly around the galaxy, completing missions, shooting down criminals, and building an empire.

The success of your missions depends on the unique design of your ship because the game simulates your ship's behavior to high accuracy. Bullets damage specific components rather than decrementing a generic "hull health," meaning that you have to think carefully about placing your ship components. Heat exchange, aerodynamics, and radar reflectivity will also be simulated.

Other space games such as KSP and Space Engineers do this, but their ships are non-rigid which leads to computational limits on ship sizes. Peregrine is designed to be computationally efficient. A key step on the way to efficiency is assuming rigid ships, which makes the physics engine much simpler. Many properties such as ship aerodynamics to be pre-computed and saved to further reduce runtime calculations.

## Todo

### Part placement
* Allow placement of parts

### Ship aesthetics
* Ship exteriors
* Panels
* Thrusters

### Wiring
* Internal linking
* Controls

### Planets
* Terrain rendering
* Terrain level of detail
* Sky box

### Physics
* Gravity
* Collisions

### Space stations
* Make one
* Docking and undocking
* UI for ship design at a hangar bay

That concludes a release.

## Optimizations
* Modify the loading macro to only load each material once

## Instructions for exporting from blender
* Use Wavefront (experimental) format
* Use x forward, z up
* You probably want selected only
* Select UV coordinates, normals, materials, and triangulated mesh
* Select Material Groups