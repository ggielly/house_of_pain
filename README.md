# House of Pain

**_Where digital suffering is a feature, not a bug._**

## What is this?

Welcome to the House of Pain — actually a proudly niche simulator of yeast and sourdough starter dynamics. In plain French: c'est un simulateur de levure/levain pour faire du pain virtuel. Oui, vraiment.

This project models fermentation, proofing, and all the tiny, passive-aggressive decisions a starter makes while you stare at it and wonder if you fed it enough. It's built in Rust because your virtual levain deserves safety, performance, and an existential crisis with no UB.

## So, what does it *do*?

It simulates the life and times of a bread starter: hydration, temperature, feeding schedule, bubble formation, gluten network optimism, collapse, and the slow, dramatic arc from flour+water to virtual loaf. There are optional graphics for watching the spores of hope rise and fall, and plenty of sliders to lovingly destroy your sourdough's self-esteem.

### Features

- Realistic-ish fermentation model: yeast and lactic bacteria argue in a mathematically plausible way.
- Feeding schedule simulation: nurture or neglect your starter — consequences are simulated with merciless accuracy.
- Temperature and hydration controls: watch your dough bloom or sulk depending on thermostat cruelty.
- `pain_graphics` viewer: minimalist visuals so you can judge your loaf without distraction.
- Save/Load starter states: preserve traumatic histories, or reset and repeat the cycle of hope.
- Optional noise, wobble and micro-bubbles: because presentation matters.

### Work in progress

- [ ] Improved starter-evolution model (long-term microflora drift)
- [ ] More realistic gluten network visualization

## Building the Pain

This project requires SDL2. Please see `README_SDL2_WINDOWS.md` for setup instructions.

```powershell
# Build the (very serious) sourdough simulator
$env:LIB = 'C:\src\house_of_pain\vcpkg\installed\x64-windows\lib;' + $env:LIB
$env:PATH = 'C:\src\house_of_pain\vcpkg\installed\x64-windows\bin;' + $env:PATH
cargo build -v

# Run the simulator and watch your starter judge you
cargo run
```

## Contributing

Have an idea to make the starter fussier or more forgiving? Open an issue or a PR. Useful features will be reluctantly accepted, and genuinely helpful improvements may prompt suspicious rejoicing.

## License

Licensed under the MIT License. Share the pain, not the blame.
