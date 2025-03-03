"""
katsuba.utils
~~~~~~~~~~~~~

Provides selected utilities from the `katsuba-utils` crate
which are thought to be broadly useful to library users.
"""

def string_id(input: str | bytes | bytearray) -> int:
    """
    Hashes the given data using KingsIsle's StringId.

    :param input: The input string to hash.
    :return: The resulting hash value.
    """

def djb2(input: str | bytes | bytearray) -> int:
    """
    Hashes the given data using the DJB2 algorithm.

    :param input: The input string to hash.
    :return: The resulting hash value.
    """
