
# Pretty dtoa

Configurable float and double printing. ``pretty_dtoa`` Comes with tons of options for configuring different aspects of displaying floats.

This crate uses a fork of the [ryu crate](https://github.com/dtolnay/ryu), that makes some of the internal modules public.

This module is fairly slow at the moment (around 3-4x slower than the default Display implementation for f64), benchmarks can be run with ``cargo bench``.

todo before release:
Have tests for ftoa
