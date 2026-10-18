"""
This module implements the ObjectProperty serialization system.

It processes the proprietary binary format used by KingsIsle to distribute C++
objects over the network or as persistent game files. The heart of the module
is :class:`Serializer`.
"""

from collections.abc import Iterable, Mapping
from os import PathLike
from pathlib import Path
from typing import Any, Literal, Self, Sequence, overload

STATEFUL_FLAGS: Literal[1]
"""
Flag bit for use in :attr:`SerializerOptions.flags`.

It is usually necessary to deserialize data with the same flags that were also
used for serialization. This leads to the issue of two parties having to
coordinate which flags they use for data exchange.

Using this flag will cause the mask of serializer flags to be encoded into the
data stream directly, which in turn also allows deserializers to pick up on
them automatically.
"""

COMPACT_LENGTH_PREFIXES: Literal[2]
"""
Flag bit for use in :attr:`SerializerOptions.flags`.

Length prefixes of lists and strings in the serialized output are compressed to
save space, whenever possible.
"""

HUMAN_READABLE_ENUMS: Literal[4]
"""
Flag bit for use in :attr:`SerializerOptions.flags`.

Enums will be serialized as a human-readable string rather than their integral
value. This ensures previously serialized data can still be correctly
deserialized even after the enum definition changes.
"""

WITH_COMPRESSION: Literal[8]
"""
Flag bit for use in :attr:`SerializerOptions.flags`.

The serialized data stream will be compressed to save space.
"""

FORBID_DELTA_ENCODE: Literal[16]
"""
Flag bit for use in :attr:`SerializerOptions.flags`.

Forbids the omission of properties that support delta encoding from the
serialized stream.

Delta encoding refers to certain properties which can be omitted from the
serialized stream when the value has not been altered since the last time
the object was serialized before.
"""

class TypeList:
    """
    A manifest of C++ type information used for ObjectProperty serialization.

    Type lists are obtained from a running game using
    `wiztype <https://github.com/wizspoil/wiztype>`_ for Wizard101, or
    `arrtype <https://github.com/wizspoil/arrtype>`_ for Pirate101. See these
    external projects for further information and usage instructions.

    .. note::

        This class supports all versions of the JSON format produced by the
        ``wiztype`` and ``arrtype`` projects.

    The resulting JSON file contains all type information that can be extracted
    from metadata directly embedded into the game clients. This metadata is used
    by the games to enable runtime reflection and serialization of the described
    classes, which allows us to leverage them for the same purpose.

    A :class:`TypeList` is therefore almost always used in conjunction with a
    :class:`Serializer` instance to serialize or deserialize :class:`Object`
    values.

    :param data: The JSON data of a type list as a string.
    """

    def __init__(self, data: str) -> None: ...

    @classmethod
    def open(cls, path: str | PathLike[str]) -> Self:
        """
        Reads a type list in JSON format from a given path.

        :param path: The filesystem path where the type list resides.
        :return: The list instance.
        """

    @classmethod
    def open_many(cls, paths: Sequence[str | PathLike[str]]) -> Self:
        """
        Reads type lists in JSON from all given paths and merges them into one
        :class:`TypeList` instance.

        .. warning::

            Duplicate type definitions in the input files are silently discarded
            during the merge process.

        :param paths: A sequence of paths to read.
        :return: The merged type list.
        """

    def name_for(self, type_hash: int) -> str:
        """
        Translates a type hash to a type name.

        This is primarily useful to convert :attr:`Object.type_hash` values
        obtained through :meth:`Serializer.deserialize` into human-readable
        strings.

        :param type_hash: The hash to translate.
        :return: The corresponding type name.
        :raises KeyError: No matching entry found in the type list.
        """

class SerializerOptions:
    """
    Configurable options to customize the behavior of a :class:`Serializer`.

    :param flags: The mask of serializer flags to use. See :attr:`flags` for
                  more details.
    :param property_mask: The filter mask for properties to use. See
                          :attr:`property_mask` for more details.
    :param shallow: Whether the shallow data format should be used. See
                    :attr:`shallow` for more details.
    :param manual_compression: Whether an object stream should be compressed
                               manually. See :attr:`manual_compression` for
                               more details.
    :param recursion_limit: The recursion limit to enforce while serializing
                            nested lists and objects. See :attr:`recursion_limit`
                            for more details.
    :param skip_unknown_types: Whether unknown types should be ignored by the
                               serializer. See :attr:`skip_unknown_types` for
                               more details.
    """

    def __init__(
        self,
        *,
        flags: int = 0,
        property_mask: int = 7,
        shallow: bool = True,
        manual_compression: bool = False,
        recursion_limit: int = 127,
        skip_unknown_types: bool = False
    ) -> None: ...

    @property
    def flags(self) -> int:
        """
        Gets or sets the serializer flags to use.

        Serializer flags are a bit mask comprised of any combination of the
        following flags:

        - :data:`STATEFUL_FLAGS`
        - :data:`COMPACT_LENGTH_PREFIXES`
        - :data:`HUMAN_READABLE_ENUMS`
        - :data:`WITH_COMPRESSION`
        - :data:`FORBID_DELTA_ENCODE`

        Every flag influences how the ObjectProperty binary format is
        interpreted. See their respective docs for more detail.

        It is generally necessary to use the same flags as the serializer to
        deserialize an object. There are no fixed rules for how these flags
        are chosen, so they must be determined individually per object.
        """
        ...

    @flags.setter
    def flags(self, v: int) -> None: ...

    @property
    def property_mask(self) -> int:
        """
        Gets or sets a bit mask of property flags to skip during serialization
        and deserialization.

        Property flags are metadata associated with every property in the
        :class:`TypeList`. They are stored in the ``flags`` JSON key of every
        property.

        During serialization and deserialization, properties that do not
        fulfill ``(property.flags & property_mask) == property_mask`` are skipped.
        This mechanism can thus be used to conditionally exclude certain
        properties from serialization by their flag mask.

        The default value matches KingsIsle's default. The mask used for
        deserialization should match the one that wa used for serialization.
        """
        ...

    @property_mask.setter
    def property_mask(self, v: int) -> None: ...

    @property
    def shallow(self) -> bool:
        """
        Gets or sets whether the shallow serialization format should be applied.

        In shallow mode, all properties of a type are written in their sequential
        order without further metadata. It is therefore necessary to know the
        exact type definition and the :attr:`property_mask` that was used for
        serialization to interpret the data correctly.

        In deep mode, every property is tagged with its written size and an
        identifying hash. This guards against future changes to the original
        type definition (such as changing the C++ type of a property or
        reordering the properties of the type).

        The shallow format therefore trades robustness of serialized blobs
        for savings in binary size and is commonly found in networking. Most
        persistent object state on the other hand prefers to use the deep
        format instead.
        """
        ...

    @shallow.setter
    def shallow(self, v: bool) -> None: ...

    @property
    def manual_compression(self) -> bool:
        """
        Gets or sets whether serialized binary state uses a manual layer of
        compression.

        Despite the overlap with the :data:`WITH_COMPRESSION` flag bit (see
        :attr:`flags`), setting this option to ``True`` causes a distinct
        change to the binary format.

        In practice, only one of these two options should be used to employ
        compression.
        """
        ...

    @manual_compression.setter
    def manual_compression(self, v: bool) -> None: ...

    @property
    def recursion_limit(self) -> int:
        """
        Gets or sets the recursion limit for serialization and deserialization.

        This value specifies the maximum number of recursive nesting for lists
        and objects. When exhausted, the operation will be aborted and an
        exception is raised.

        This is used to prevent potentially malicious input from crashing the
        application by overflowing the stack. The default limit is fine for
        most users.
        """
        ...

    @recursion_limit.setter
    def recursion_limit(self, v: int) -> None: ...

    @property
    def skip_unknown_types(self) -> bool:
        """
        Gets or sets whether unknown types should be ignored during
        deserialization.

        Unknown types are objects which are missing an entry in
        :class:`TypeList`. An accurate interpretation of the serialized data is
        therefore not possible.

        In deep mode specifically (i.e. :attr:`shallow` is ``False``), any
        unknown type can safely be ignored and substituted with ``None``.

        For serialization, this property has no effect.
        """
        ...

    @skip_unknown_types.setter
    def skip_unknown_types(self, v: bool) -> None: ...

class Object(dict[str, Any]):
    """
    Representation of an object in the ObjectProperty system.

    Objects are obtained either through deserialization with
    :meth:`Serializer.deserialize` or through manual construction (e.g. for
    serialization with :meth:`Serializer.serialize`). Think of them as
    instantiations of types defined in the :class:`TypeList` your program is
    working with.

    As a subclass of :class:`dict`, this type supports all operations a regular
    dictionary also supports and maps property names to values using the
    following value conversion guidelines:

    .. list-table::
        :widths: 35 65
        :header-rows: 1

        * - Python type
          - C++ type (as specified in the type list)
        * - :class:`bool`
          - ``bool``
        * - :class:`int`
          - Any kind of integer (``char``, ``short``, ``unsigned int``, ``bi4``, etc.)
        * - :class:`float`
          - ``float`` or ``double``
        * - :class:`str`
          - ``std::string`` (encoded as UTF-8) or ``std::wstring`` (encoded as UTF-16)
        * - :class:`bytes`
          - ``std::string`` or ``std::wstring`` (raw bytes)
        * - :class:`dict` with ``r``, ``g``, ``b``, ``a`` keys
          - ``class Color``
        * - :class:`dict` with ``x``, ``y``, ``z`` keys
          - ``class Vector3D``
        * - :class:`dict` with ``i``, ``j``, ``k`` keys, eaching being a list with 3 row elements
          - ``class Matrix3x3``
        * - :class:`dict` with ``pitch``, ``roll``, ``yaw`` keys
          - ``class Euler``
        * - :class:`dict` with ``x``, ``y``, ``z``, ``w`` keys
          - ``class Quaternion``
        * - :class:`dict` with ``width``, ``height`` keys
          - ``class Size<T>``
        * - :class:`dict` with ``x``, ``y`` keys
          - ``class Point<T>``
        * - :class:`dict` with ``left``, ``top``, ``right``, ``bottom`` keys
          - ``class Rect<T>``
        * - :class:`list`
          - ``std::list<T>`` or ``std::vector<T>``
        * - :class:`Object`
          - Another object (or any pointer type pointing to one)
        * - :data:`None`
          - ``null`` value for pointers or an unknown object

    Any arguments after ``type_hash`` are forwarded to :class:`dict`'s own
    constructor, all overloads supported.

    :param type_hash: The type hash identifying the object's C++ type in
                      the :class:`TypeList`. See :mod:`katsuba.utils` for
                      an overview of functions that can be used to convert
                      a type name string into a supported hash.
    :type type_hash: int
    """

    @overload
    def __init__(self, type_hash: int) -> None: ...

    @overload
    def __init__(
        self, type_hash: int, mapping: Mapping[str, Any], /, **kwargs: Any
    ) -> None: ...

    @overload
    def __init__(
        self, type_hash: int, iterable: Iterable[tuple[str, Any]], /, **kwargs: Any
    ) -> None: ...

    @overload
    def __init__(self, type_hash: int, **kwargs: Any) -> None: ...

    @property
    def type_hash(self) -> int:
        """
        The type hash value associated with this object.

        This property is read-only.
        """
        ...

class Serializer:
    """
    A serializer for the ObjectProperty binary format.

    It manages a :class:`TypeList` which dynamically provides C++ type
    definitions used by KingsIsle titles. These type definitions serve
    as blueprints for converting instances of such types to and from
    the binary format.

    The format is used for persistent object state distributed in game
    assets and to transfer C++ objects over the network. Instances of
    :class:`SerializerOptions` are used in individual scenarios to
    customize the produced binary format.

    These objects are represented by the dictionary-like :class:`Object`
    for convenient interfacing from Python, see the documentation for
    more details.

    :param opts: The options to configure serializer behavior.
    :param types: The type list for lookup during (de)serialization.
    """

    def __init__(self, opts: SerializerOptions, types: TypeList) -> None: ...

    @property
    def options(self) -> SerializerOptions:
        """
        Gets or sets the current serializer configuration.

        .. warning::

            The returned value is a copy. Trying to modify its individual
            fields does not alter the configuration of the serializer. Use
            the other :class:`Serializer` properties for that instead.
        """

    @options.setter
    def options(self, v: SerializerOptions) -> None: ...

    @property
    def flags(self) -> int:
        """
        Gets or sets :attr:`SerializerOptions.flags` of the current serializer
        configuration.
        """
        ...

    @flags.setter
    def flags(self, v: int) -> None: ...

    @property
    def property_mask(self) -> int:
        """
        Gets or sets :attr:`SerializerOptions.property_mask` of the current
        serializer configuration.
        """
        ...

    @property_mask.setter
    def property_mask(self, v: int) -> None: ...

    @property
    def shallow(self) -> bool:
        """
        Gets or sets :attr:`SerializerOptions.shallow` of the current serializer
        configuration.
        """
        ...

    @shallow.setter
    def shallow(self, v: bool) -> None: ...

    @property
    def manual_compression(self) -> bool:
        """
        Gets or sets :attr:`SerializerOptions.manual_compression` of the current
        serializer configuration.
        """
        ...

    @manual_compression.setter
    def manual_compression(self, v: bool) -> None: ...

    @property
    def recursion_limit(self) -> int:
        """
        Gets or sets :attr:`SerializerOptions.recursion_limit` of the current
        serializer configuration.
        """
        ...

    @recursion_limit.setter
    def recursion_limit(self, v: int) -> None: ...

    @property
    def skip_unknown_types(self) -> bool:
        """
        Gets or sets :attr:`SerializerOptions.skip_unknown_types` of the current
        serializer configuration.
        """
        ...

    @skip_unknown_types.setter
    def skip_unknown_types(self, v: bool) -> None: ...

    def deserialize(self, data: bytes) -> Object | None:
        """
        Tries to deserialize the given binary data to an object.

        It is important to configure appropriate :class:`SerializerOptions`
        for the type of format that should be deserialized. These are
        determined on an individual basis and must match the configuration
        used during serialization in most cases.

        :param data: The raw data to deserialize.
        :return: The resulting :class:`Object`.
        :raises OSError: Corrupt or truncated data stream received.
        :raises KatsubaError: Unknown type or invalid data format.
        """

    def serialize(self, obj: Object | None) -> bytes:
        """
        Serializes the given object to a stream of :class:`bytes`.

        For repeated calls, prefer :meth:`serialize_into` with a
        :class:`bytearray` retained across calls to save the
        allocation of a new :class:`bytes` object every time.

        :param obj: The object to serialize, or ``None`` to indicate
                    a missing value.
        :return: The serialized bytes.
        :raises KatsubaError: Unknown type or mismatched property.
        """

    def serialize_into(self, obj: Object | None, buf: bytearray) -> None:
        """
        Serializes the given object directly into ``buf``, overwriting
        its previous contents.

        :param obj: The object to serialize, or ``None`` to indicate
                    a missing value.
        :param buf: The buffer to serialize into.
        :raises KatsubaError: Unknown type or mismatched property.
        """
