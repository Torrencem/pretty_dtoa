
# Pretty dtoa

Configurable float and double printing. ``pretty_dtoa`` Comes with tons of options for configuring different aspects of displaying floats.

This crate uses a fork of the [ryu crate](https://github.com/dtolnay/ryu) that makes some of the internal modules public. It uses the fast ryu algorithm to generate a "floating decimal", or a floating point with radix 10, and then it uses formatting rules particular to the configuration to create a formatted string.

This module is slightly slow (usually between 1.5x and 2.5x slower than the default Display implementation for f64). Benchmarks can be run with ``cargo bench``.

Consider using ``pretty_dtoa`` if:

* Float printing is not a huge bottleneck for your application, or a small slowdown of float formatting is otherwise acceptable

* The default behavior of ``Display`` and alternative float printing libraries like ``ryu`` is not ideal for one reason or another

