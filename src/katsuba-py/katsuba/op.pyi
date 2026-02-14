"""
katsuba.op
~~~~~~~~~~

Implements support for ObjectProperty binary deserialization.
This is a wrapper over the `katsuba-object-property` Rust crate.
"""

from os import PathLike
from pathlib import Path
from typing import Any, Literal, Self, Sequence

STATEFUL_FLAGS: Literal[1 << 0]
COMPACT_LENGTH_PREFIXES: Literal[1 << 1]
HUMAN_READABLE_ENUMS: Literal[1 << 2]
WITH_COMPRESSION: Literal[1 << 3]
FORBID_DELTA_ENCODE: Literal[1 << 4]

class TypeList:
    """
    A list of runtime type information for deserialization.

    Type lists can be obtained from the game using the wiztype
    project. Every type list is a JSON file describing the
    embedded C++ types of the game binary.

    :param data: The JSON data of a type list as a string.
    """
    def __new__(cls, data: str) -> Self: ...

    @classmethod
    def open(cls, path: str | PathLike | Path) -> Self:
        """
        Opens a type list file on filesystem.

        This is preferred over calling the constructor for big files
        because the data is parsed in a streaming fashion then.

        :param path: The filesystem path where the type list resides.
        :return: The list instance.
        """

    @classmethod
    def open_many(cls, paths: Sequence[str | PathLike | Path]) -> Self:
        """
        Opens all given JSON files and merges them into one type list.

        :param paths: A sequence of paths to open.
        :return: The merged list instance.
        """

    def name_for(self, type_hash: int) -> str:
        """
        Translates a type hash to a type name.

        :param type_hash: The hash to look up.
        :return: The corresponding type name.
        :raises KeyError: The hash is not found in the type list.
        """

class SerializerOptions:
    """
    Customization options for serializer behavior.
    """
    def __new__(cls) -> Self: ...

    @property
    def flags(self) -> int: ...
    @flags.setter
    def flags(self, v: int) -> None: ...

    @property
    def property_mask(self) -> int: ...
    @property_mask.setter
    def property_mask(self, v: int) -> None: ...

    @property
    def shallow(self) -> bool: ...
    @shallow.setter
    def shallow(self, v: bool) -> None: ...

    @property
    def manual_compression(self) -> bool: ...
    @manual_compression.setter
    def manual_compression(self, v: bool) -> None: ...

    @property
    def recursion_limit(self) -> int: ...
    @recursion_limit.setter
    def recursion_limit(self, v: int) -> None: ...

    @property
    def skip_unknown_types(self) -> bool: ...
    @skip_unknown_types.setter
    def skip_unknown_types(self, v: bool) -> None: ...

class Object(dict[str, Any]):
    """
    A deserialized object.

    Subclasses :class:`dict` so all properties are accessible as dict keys.
    The ``type_hash`` attribute identifies the object's C++ type.

    :param type_hash: The type hash identifying the object's C++ type.
    """
    def __new__(cls, type_hash: int) -> Self: ...

    @property
    def type_hash(self) -> int: ...

class Serializer:
    """A serializer for the ObjectProperty system.

    This implements deserialization of objects from the binary
    format used by KingsIsle in game assets and networking.

    :param opts: The options to configure serializer behavior.
    :param types: The type list for lookup during deserialization.
    """
    def __new__(cls, opts: SerializerOptions, types: TypeList) -> Self: ...

    def deserialize(self, data: bytes) -> Object:
        """
        Deserializes the given binary data to an object.

        :param data: The raw data to deserialize.
        :return: The resulting object value.
        :raises OSError: I/O error occurred while trying to read data.
        :raises KatsubaError: Unknown type or invalid data format.
        """
