"""
Miscellaneous utilities to enhance the user experience.
"""

from collections.abc import Buffer

def string_id(input: str | Buffer) -> int:
    """
    Hashes the given data using KingsIsle's StringId algorithm.

    This function is used by Wizard101 to hash type names for the
    ObjectProperty system.

    .. note::

        While this function accepts all types of buffers, it is
        primarily intended to be a string hashing routine.

        Results matching KingsIsle's implementation are only
        guaranteed for inputs in the printable ASCII range.

    :param input: The input string to hash.
    :return: The resulting hash value.
    """

def djb2(input: str | Buffer) -> int:
    """
    Hashes the given data using the DJB2 algorithm.

    This function is used by Pirate101 to hash type names for the
    ObjectProperty system. It is used by both Wizard101 and Pirate101
    to hash property names.

    :param input: The input string to hash.
    :return: The resulting hash value.
    """
