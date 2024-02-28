![Looming Juggler](www/juggler_title.png)

# Juggler in Rust

## Introduction

This is a recreation of the classic 1986 Amiga Juggler demo in the
[Rust](https://www.rust-lang.org/) programming language, mainly intended
to run on the [Raspberry Pi](https://www.raspberrypi.com/) series of
single board computers. Juggler was a famous animation that demonstrated
the use of raytracing algorithms on a home computer.

## Backstory

In January 2024, I decided to start learning the Rust programming
language in earnest. This required a project that was interesting enough
to keep me motivated—something neither too trivial nor too complicated.

As it happens, an article featured on
[Hackaday](https://hackaday.com/2024/01/26/a-zx-spectrum-raytracer-in-basic/)
gave an idea for the perfect project. In the article, Gabriel Gambetta
builds a raytracer in ZX Spectrum BASIC. Gabriel had written an
excellent book that goes into details on how to build your own raytracer
and rasterizer, called [Computer Graphics from
Scratch](https://gabrielgambetta.com/computer-graphics-from-scratch/).

While the Rust implementation of a simple raytracer was getting along
nicely, I stumbled into another article: history of Eric Graham's Amiga
Juggler animation / demo, [curated by Ernie
Wright](http://www.etwright.org/cghist/juggler.html). In the article,
there was a link to an ADF file, which
[BigBaldGeek](https://www.dottyflowers.com/#mozTocId334772) had
extracted from an original _Ray Tracer 1.0_ disk. The disk had the
geometry data for the juggler "robot" figure. I simply had to plug the
data in my raytracer, to see how fast it would run on today's computers…

## Getting Started

You can download one of the [prebuilt
binaries](https://github.com/unfastener/juggler-in-rust/releases) for
32- and 64-bit Raspberry Pi computers.

Otherwise, you have to compile the program on your computer. In case you
are new to building and running Rust programs, below are some basic
instructions.

### Building and Running

⚠️ **Note:** For any Raspberry Pi model with less than 4 GB of memory,
it is necessary to increase the swap size beyond the default 100 MB to
successfully complete the build process. You can do this by setting
`CONF_SWAPSIZE` to `2048` in the _/etc/dphys-swapfile_ configuration
file – which increases the swap size to 2 GB – and rebooting.

To build the program from scratch, first make sure you have a working
[Rust build environment](https://www.rust-lang.org/learn/get-started).
Then, clone the repository to your computer (do not type the `$` in the
commands below):

```
$ git clone https://github.com/unfastener/juggler-in-rust.git
```

To build the release binary:

```
$ cd juggler-in-rust
$ cargo build --release
```

This may take a while. See section [Performance](#Performance) below for
typical build times.

You can then run the program:

```
$ cargo run --release
```

, or alternatively:

```
$ target/release/juggler-in-rust
```

### Controls

The program has a few keyboard controls:

- `1`, `2`, `3`, `4`, `5`: Control juggler speed

- `6`, `7`, `8`, `9`, `0`: Control camera direction and speed

- `f`, `F11`: Toggle fullscreen

- `q`, `Esc`: Quit program

- `b`: Toggle "extra geometry"

## Technical Details

According to Eric Graham, the author of the original Juggler demo, a
stock Amiga 1000 with 512 kB of memory would take about an hour to
render each frame. The original animation consists of 24 frames with a
fixed camera pointed at the juggler.

In contrast, this implementation runs in real time on a recent Raspberry
Pi computer. I chose 24 fps to be the target framerate, as the [world
agrees](https://youtu.be/28S47EE_opA?si=ci15gu37hJugGVW8&t=226) that it
is the best framerate. The program does a few test renders on startup to
select a suitable render size that meets or exceeds the target
framerate. In addition to the juggling animation, the camera also
rotates around the juggler.

This is a pure-CPU raytracer implementation. It uses the
[softbuffer](https://crates.io/crates/softbuffer) and
[winit](https://crates.io/crates/winit) Rust crates for displaying the
window, and [vecmath](https://crates.io/crates/vecmath) for the vector
mathematics. No GPU resources are used. A CPU-only nearest-neighbor
interpolation algorithm is used to scale the rendered image to the
output window size.

There are as many render threads as there are (logical) cores available.

### Performance

Here's how the program runs on various Raspberry Pi versions:

| Board     | Cores          | Frequency | RAM    | Disk      | Full Release Build Time | Display | Render Size | FPS   |
| --------- | -------------- | --------- | ------ | --------- | ----------------------- | ------- | ----------- | ----- |
| Pi 5      | 4 × Cortex-A76 | 2400 MHz  | 8 GB   | USB-3 SSD | 1m52s                   | Wayland | 320×320     | 30–32 |
| Pi 4      | 4 × Cortex-A72 | 1800 MHz  | 8 GB   | USB-3 SSD | 5m40s                   | Wayland | 256×256     | 23–25 |
| Pi 3B+    | 4 × Cortex-A53 | 1400 MHz  | 1 GB   | USB-2 SSD | 12m22s                  | X11     | 160×160     | 24–26 |
| Pi Zero W | 1 × ARM11      | 1000 MHz  | 512 MB | Micro-SD  | 1h29m55s                | X11     | 80×80       | 4–5   |

These test results are with Pi OS 12 (Bookworm) 64-bit, except for the
Pi Zero W with Raspberry Pi OS 11 (Bullseye) 32-bit.

As you can see, full build times can be quite significant for Rust
programs, due to the large number of dependencies that the crates pull
in.

### Caveats

There are some notable differences to the original raytraced scene:

- The movement of the juggler and the juggling balls was recreated from
  scratch. As far as I know, the original movement data for the Juggler
  animation was never released.

- I had to lean the juggler back a little bit, so that the balls don't
  fly through its chest. This only becomes apparent when the camera moves
  around the juggler.

- The floor is true checkerboard, because it just looks nicer, in my
  opinion. The original Juggler demo had a checkerboard that was
  mirrored along the X and Z axes.

- Lights don't work exactly the same way as in the original animation. I
  didn't review the original code to see how it differs to the algorithms
  in Gabriel Gambetta's book. This is most apparent on the red highlights
  on the juggler's chest, and the light falloff of the ground in the
  distance.

- The image is rendered as a square to simplify the calculations related
  to the camera and viewport. This approach is subject to future
  improvements.

- There is no sound, for now. It would no doubt pull in even more
  dependencies.

## Lessons Learned

Coming from C and Python background, here are some of the things I
learned:

- Rust has a large ecosystem of packages (_crates_), and Cargo makes
  using them straightforward.

- Using an editor that has [rust-analyzer](https://github.com/rust-lang/rust-analyzer) integrated –
  for real-time error checking – is great for learning.

- [ChatGPT 4](https://chat.openai.com/) has been a great help and can tremendously speed
  up the learning process for Rust (or any programming language, probably).

- Compiling Rust takes a beefy machine, because:

  - Even a simple project like this one pulls in a large number of
    dependencies;
  - Almost everything gets compiled from source and linked together into
    a single binary;
  - It seems there are no binary libraries in Rust, as such.

- Rust's cross-platform support seems competent.

  - Next stop: bare-metal Rust on ARM Cortex-M!

- Writing a basic raytracer is simpler than I thought.
  - Seeing a real-time raytraced scene appear after just a few hours of
    work – running on an $80 computer, no less – is wild!

## Acknowledgements

I would like to thank the following people for providing the online
resources that this project builds on:

- [Gabriel Gambetta](https://gabrielgambetta.com/)—for the Computer
  Graphics from Scratch book.
- [Ernie Wright](http://www.etwright.org/)—for documenting the history
  of the Amiga Juggler demo.
- [BigBaldGeek](https://www.dottyflowers.com/)—for finding the original
  Ray Tracer 1.0 disk and reading it into an ADF.

And, of course, special thanks to Eric Graham for creating the original
Juggler demo!

## License

"The Unlicense"

This is free and unencumbered software released into the public domain.

Anyone is free to copy, modify, publish, use, compile, sell, or
distribute this software, either in source code form or as a compiled
binary, for any purpose, commercial or non-commercial, and by any means.

In jurisdictions that recognize copyright laws, the author or authors of
this software dedicate any and all copyright interest in the software to
the public domain. We make this dedication for the benefit of the public
at large and to the detriment of our heirs and successors. We intend
this dedication to be an overt act of relinquishment in perpetuity of
all present and future rights to this software under copyright law.

THE SOFTWARE IS PROVIDED “AS IS”, WITHOUT WARRANTY OF ANY KIND, EXPRESS
OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF
MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT.
IN NO EVENT SHALL THE AUTHORS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING
FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
DEALINGS IN THE SOFTWARE.
