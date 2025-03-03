"""
katsuba
~~~~~~~

A Python library for interfacing with KingsIsle file formats.
This implements ergonomic bindings to the ``katsuba`` Rust
crates which are responsible for the heavy lifting.
"""

class KatsubaError(Exception):
    """
    Base class for errors in katsuba.
    
    Errors from the Rust side that cannot be mapped to a more
    appropriate builtin exception type will be reported using
    this type.
    """
