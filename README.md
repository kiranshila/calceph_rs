# CALCEPH_rs

A safe Rust interface to the [CALCEPH](https://www.imcce.fr/inpop/calceph) planetary ephemeris library.

This library is faster, thread-safe, and lighter weight than JPL's SPICE, but is less fully featured. However, this is still very useful for the likes of calculating things like telescope pointings.

## A Note About the Interface

Unlike other wrapper libraries, this high-level interface does not try to match the API of the C library. This is because most of CALCEPH violates the Rust practice of "invalid state should be unrepresentable".
For example, the `calceph_compute_units` function accepts units that are created by the addition of constant integers. Of course, *most* integers are invalid, leading to the unfortunate consequence of needing runtime correctness checks.
Further, the kind of target and center positions are sometimes not valid for the types requested.
Here, the functions that capture drastically different behavior and input types are broken out into actually different functions that are correct at compile time.
We can't change the C code, but at least we know we're correct.

## Thread Safety

According to the [docs](https://calceph.imcce.fr/docs/4.0.0/html/c/calceph.multiple.cusage.html#thread-notes), functions that access `t_calcephbin`
or `CalcephBin` in Rust, are threadsafe if `calceph_isthreadsafe` returns a non-zero value.
This is a little strange as there must be a call to `calceph_prefetch` before that.
So, we end up with runtime-dependent thread-safety - which is against Rust's requirements for `Send`.

So, if you want `CalcephBin` to be `Send`, here we use a feature `threadsafe` that performs these checks in the constructor so you will only get a `CalcephBin` iff the requirements are actually met.
However, this does require that prefetch call (now included in the constructor), which may be problematic for exceptionally large files.
