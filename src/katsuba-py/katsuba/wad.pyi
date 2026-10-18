"""
This module offers functionality for reading and writing the
KingsIsle WAD archive format. See :class:`Archive` for more
details.
"""

from collections.abc import Iterator
from os import PathLike
from pathlib import Path
from typing import Self

from katsuba.op import Object, Serializer

class Archive:
    """
    Representation of a KingsIsle WAD archive file.

    WAD archive files carry the ``.wad`` file ending and serve as an
    archive format for bundling and distributing all game assets. This
    class offers an immutable interface for accessing and extracting
    the contents of such archives.

    .. note::

        Other games, most notably Doom, use archive formats which are
        known as WAD also. The overlap in naming is purely coincidental
        and this class can only be used to interface with the files
        specifically found in KingsIsle games.

    This class has no public constructor. To obtain an instance, use
    :meth:`heap` or :meth:`mmap` to open an existing file. Both methods
    validate file ranges and CRCs internally.
    """

    @classmethod
    def heap(cls, path: str | PathLike[str]) -> Self:
        """
        Opens an archive file and reads its contents into memory.

        This will always read the whole file into memory upfront, so
        memory usage is proportional to the size of the archive.
        Prefer :meth:`mmap` for large archive files.

        :param path: Path to the archive file to open.
        :return: The archive instance.
        :raises OSError: Failed to read the given file.
        :raises KatsubaError: File is corrupt or invalid.
        """

    @classmethod
    def mmap(cls, path: str | PathLike[str]) -> Self:
        """
        Opens an archive file using memory-mapped I/O.

        Data from the archive file is loaded on demand, which reduces
        load times and memory usage. However, the backing file may not
        be deleted or externally modified while the archive is open.

        :param path: Path to the archive file to open.
        :return: The archive instance.
        :raises OSError: Failed to read the given file.
        :raises KatsubaError: File is corrupt or invalid.
        """

    def __len__(self) -> int:
        """
        Returns how many files are stored inside the archive.

        :return: The total number of files.
        """

    def __contains__(self, path: str) -> bool:
        """
        Checks if the given file path is present in the archive.

        :param path: Path to check, relative to the archive.
        :return: Whether the file is present.
        """

    def __getitem__(self, path: str) -> bytes:
        """
        Extracts the contents of a file in the archive.

        :param path: The file path relative to the archive.
        :return: The uncompressed file contents.
        :raises KeyError: The file is not present in the archive.
        :raises KatsubaError: Corrupt or missing file contents.
        """

    def __iter__(self) -> Iterator[str]:
        """
        Iterates all files stored inside the archive.

        :return: Iterator yielding file paths from the archive.
        """

    def iter_glob(self, pattern: str) -> Iterator[str]:
        """
        Iterates all files inside the archive that match the given
        glob pattern.

        :param pattern: Glob pattern to filter paths by.
        :return: Iterator yielding file paths from the archive.
        :raises KatsubaError: Glob pattern was invalid.
        """

    def deserialize(self, file: str, serializer: Serializer) -> Object:
        """
        Deserializes a file in the archive with the given serializer.

        This is a convenience method to extract and deserialize
        ObjectProperty state in a single operation. The configuration
        of the provided :class:`~katsuba.op.Serializer` instance is preserved
        across every call.

        .. note::

            In the common case of ObjectProperty state starting with
            a magic ``BINd`` file header, the serializer configuration
            is temporarily adapted to fulfill the request.

        :param file: The file path relative to the archive.
        :param serializer: The serializer instance to use.
        :return: The deserialized value.
        :raises OSError: I/O error while trying to read data.
        :raises KatsubaError: Unknown type or invalid data format.
        """
