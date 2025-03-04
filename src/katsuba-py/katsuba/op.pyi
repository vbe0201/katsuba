"""
katsuba.op
~~~~~~~~~~

Implements support for ObjectProperty binary deserialization.
This is a wrapper over the `katsuba-object-property` Rust crate.
"""

from os import PathLike
from pathlib import Path
from typing import Any, Iterator, Literal, Self, Sequence

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

    @property
    def djb2_only(self) -> bool: ...
    @djb2_only.setter
    def djb2_only(self, v: bool) -> None: ...

class LazyList:
    """
    A list storing deserialized ObjectProperty values.

    List elements are lazily resolved from Rust to Python types
    when they are accessed. This is where the name comes from.
    """

    def __len__(self) -> int:
        """
        Counts the elements in the list.

        :return: The element count.
        """

    def __getitem__(self, idx: int) -> Any:
        """
        Resolves a list element at the given index.

        :param idx: The positive index into the list.
        :return: The value stored at this index in the list.
        :raises IndexError: The index is out of range.
        """

    def __iter__(self) -> Iterator[Any]:
        """
        Iterates over all values in the list.

        :return: An iterator that yields all values.
        """

class LazyObject:
    """
    A container storing deserialized ObjectProperty objects.

    An object is type-erased and only identified by its unique
    hash value. It holds several properties representing the
    class members of the C++ type.

    Properties are lazily resolved from Rust to Python types
    when they are accessed. This is where the name comes from.
    """

    @property
    def type_hash(self) -> int:
        """
        Gets the type hash that corresponds to the stored object.

        :return: The hash value.
        """

    def __len__(self) -> int:
        """
        Gets the number of properties stored in this object.

        :return: The property count.
        """

    def __contains__(self, property: str) -> bool:
        """
        Whether a given property is stored in this object.

        :param property: The name of the property.
        :return: Whether the property is there or not.
        """

    def __getitem__(self, property: str) -> Any:
        """
        Accesses the value for a given property in this object.

        :param property: The name of the property.
        :return: The value of the property.
        :raises KeyError: The property is not contained in the object.
        """

    def get(self, property: str) -> Any | None:
        """
        Tries to get the value of a given property in this object.

        :param property: The name of the property.
        :return: The value of the property, or ``None`` if not found.
        """

    def items(self, types: TypeList) -> Iterator[tuple[str, Any]]:
        """
        Iterates over all the properties inside this object.

        :param types: A type list that provides information for this type.
        :return: An iterator yielding pairs of property name and value.
        """

class Serializer:
    """A serializer for the ObjectProperty system.
    
    This implements deserialization of objects from the binary
    format used by KingsIsle in game assets and networking.

    :param opts: The options to configure serializer behavior.
    :param types: The type list for lookup during deserialization.
    """
    def __new__(cls, opts: SerializerOptions, types: TypeList) -> Self: ...

    def deserialize(self, data: bytes) -> LazyObject:
        """
        Deserializes the given binary data to an object.

        :param data: The raw data to deserialize.
        :return: The resulting object value.
        :raises OSError: I/O error occurred while trying to read data.
        :raises KatsubaError: Unknown type or invalid data format.
        """

class Vec3:
    """
    Representation of a 3D vector.
    """

    @property
    def x(self) -> float: ...
    @x.setter
    def x(self, v: float) -> None: ...

    @property
    def y(self) -> float: ...
    @y.setter
    def y(self, v: float) -> None: ...

    @property
    def z(self) -> float: ...
    @z.setter
    def z(self, v: float) -> None: ...

class Quaternion:
    """
    Representation of a quaternion.
    """

    @property
    def x(self) -> float: ...
    @x.setter
    def x(self, v: float) -> None: ...

    @property
    def y(self) -> float: ...
    @y.setter
    def y(self, v: float) -> None: ...

    @property
    def z(self) -> float: ...
    @z.setter
    def z(self, v: float) -> None: ...

    @property
    def w(self) -> float: ...
    @w.setter
    def w(self, v: float) -> None: ...

class Matrix:
    """
    Representation of a 3x3 matrix.
    """

    @property
    def i(self) -> tuple[float, float, float]: ...
    @i.setter
    def i(self, v: tuple[float, float, float]) -> None: ...

    @property
    def j(self) -> tuple[float, float, float]: ...
    @j.setter
    def j(self, v: tuple[float, float, float]) -> None: ...

    @property
    def k(self) -> tuple[float, float, float]: ...
    @k.setter
    def k(self, v: tuple[float, float, float]) -> None: ...

class Euler:
    """
    Representation of an Euler value.
    """

    @property
    def pitch(self) -> float: ...
    @pitch.setter
    def pitch(self, v: float) -> None: ...

    @property
    def yaw(self) -> float: ...
    @yaw.setter
    def yaw(self, v: float) -> None: ...

    @property
    def roll(self) -> float: ...
    @roll.setter
    def roll(self, v: float) -> None: ...

class PointInt:
    """
    A 2D point with integer coordinates.
    """

    @property
    def x(self) -> int: ...
    @x.setter
    def x(self, v: int) -> None: ...

    @property
    def y(self) -> int: ...
    @y.setter
    def y(self, v: int) -> None: ...

class PointFloat:
    """
    A 2D point with float coordinates.
    """

    @property
    def x(self) -> float: ...
    @x.setter
    def x(self, v: float) -> None: ...

    @property
    def y(self) -> float: ...
    @y.setter
    def y(self, v: float) -> None: ...

class SizeInt:
    """
    A 2D size characterized by width and height.
    """

    @property
    def width(self) -> int: ...
    @width.setter
    def width(self, v: int) -> None: ...

    @property
    def height(self) -> int: ...
    @height.setter
    def height(self, v: int) -> None: ...

class RectInt:
    """
    A 2D rectangle with integer coordinates.
    """

    @property
    def left(self) -> int: ...
    @left.setter
    def left(self, v: int) -> None: ...

    @property
    def top(self) -> int: ...
    @top.setter
    def top(self, v: int) -> None: ...

    @property
    def right(self) -> int: ...
    @right.setter
    def right(self, v: int) -> None: ...

    @property
    def bottom(self) -> int: ...
    @bottom.setter
    def bottom(self, v: int) -> None: ...

class RectFloat:
    """
    A 2D rectangle with float coordinates.
    """

    @property
    def left(self) -> float: ...
    @left.setter
    def left(self, v: float) -> None: ...

    @property
    def top(self) -> float: ...
    @top.setter
    def top(self, v: float) -> None: ...

    @property
    def right(self) -> float: ...
    @right.setter
    def right(self, v: float) -> None: ...

    @property
    def bottom(self) -> float: ...
    @bottom.setter
    def bottom(self, v: float) -> None: ...

class Color:
    """
    A RGBA color.
    """

    @property
    def r(self) -> int: ...
    @r.setter
    def r(self, v: int) -> None: ...

    @property
    def g(self) -> int: ...
    @g.setter
    def g(self, v: int) -> None: ...

    @property
    def b(self) -> int: ...
    @b.setter
    def b(self, v: int) -> None: ...

    @property
    def a(self) -> int: ...
    @a.setter
    def a(self, v: int) -> None: ...
