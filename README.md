# AudioNimbus Demo

Experience immersive spatial audio through this interactive demo of [`audionimbus`](https://github.com/MaxenceMaire/audionimbus), a Rust wrapper for Steam Audio's spatial audio capabilities.

Walk through three distinct acoustic environments, each showcasing various spatial audio features of `audionimbus`.

You can also watch the [walkthrough video](https://www.youtube.com/watch?v=zlhW1maG0Is).

![Screenshot](./docs/screenshot.png)

## Prerequisites

Before running the demo, you must set up Steam Audio as detailed in the [`audionimbus` documentation](https://github.com/MaxenceMaire/audionimbus/tree/master/audionimbus#installation).

## Running the Demo

```bash
cargo run                    # Level 1 (Reflections)
cargo run --features direct  # Level 2 (Direct Sound)
cargo run --features reverb  # Level 3 (Reverb)
```

## Levels

### Level 1: Reflections (`cargo run`)

Navigate meandering corridors where sound reflects off the walls.
Hear how the sound remains audible despite the source being completely occluded.

### Level 2: Direct Sound (`cargo run --features direct`)

Experience precise directional audio with:
- Head-Related Transfer Function (HRTF) rendering and ambisonics for accurate directional cues
- Physical occlusion
- Natural distance attenuation

### Level 3: Reverb (`cargo run --features reverb`)

Explore a vast, resonant chamber that demonstrates reverberation and dynamic acoustic changes as you move around the space.

## Controls

- **Movement**: W (forward), A (left), S (backward), D (right)
- **Move Faster**: Hold Shift
- **Look around**: Mouse movement
