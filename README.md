
# Pretty dtoa

Configurable float and double printing. ``pretty_dtoa`` Comes with tons of options for configuring different aspects of displaying floats.

This crate uses a fork of the [ryu crate](https://github.com/dtolnay/ryu) that makes some of the internal modules public.

This module is fairly slow at the moment (between 1.5x and 4x slower than the default Display implementation for f64), and benchmarks can be run with ``cargo bench``.

Consider using ``pretty_dtoa`` if:

* Float printing is not a huge bottleneck for your application, or a slowdown of float formatting is acceptable

* The default behavior of ``Display`` and alternative float printing libraries like ``ryu`` is not ideal for one reason or another

