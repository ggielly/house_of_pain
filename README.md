# House of Pain

**_Where digital suffering is a feature, not a bug._**

## What is this?

Welcome to the House of Pain, a meticulously engineered simulator with a singular, noble purpose: to be absolutely, utterly useless. 

It is a monument to the question "we can, but should we?", and the answer is a resounding "probably not". This project was born from a desire to explore the outer limits of pointlessness, wrapped in a perfectly safe and performant Rust codebase.

## So, what does it *do*?

That's the beauty of it. It simulates... things. Vaguely. With graphics that technically exist. The core experience is a journey into the void, a testament to the art of doing nothing, beautifully.

### "Features"

**Groundbreakingly Pointless**: We've pushed the boundaries of modern technology to create an experience with no discernible goal.
**Built with Rust**: Because your existential dread should be memory-safe and blazingly fast.
**Minimalist Graphics**: Courtesy of our `pain_graphics` engine, you'll be treated to visuals that make you appreciate the sheer blackness of your monitor.
**Guaranteed Time Sink**: Have a deadline? A project to finish? Open the House of Pain and watch that productivity vanish.

## Building the Pain

This project requires SDL2. Please see `README_SDL2_WINDOWS.md` for setup instructions.

```bash
# Build the monument to futility
cargo build --release

# Unleash the "experience"
cargo run --release

$env:LIB = 'C:\src\house_of_pain\vcpkg\installed\x64-windows\lib;' + $env:LIB; $env:PATH = 'C:\src\house_of_pain\vcpkg\installed\x64-windows\bin;' + $env:PATH; cargo clean; cargo build -v
```

## Contributing

Got an idea to make it even more exquisitely pointless? We're hesitantly listening.

Please open an issue to discuss your vision of enhanced suffering. Pull requests that accidentally add a useful feature will be rejected with extreme prejudice.

## License

Licensed under the MIT License. Feel free to inflict this pain on others.
