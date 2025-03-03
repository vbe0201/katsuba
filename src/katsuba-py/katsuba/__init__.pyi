class KatsubaError(Exception):
    """Base class for errors in katsuba.
    
    Errors from the Rust side that cannot be mapped to a more
    appropriate builtin exception type will be reported using
    this type.
    """
    ...
