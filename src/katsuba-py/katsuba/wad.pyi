"""
katsuba.wad
~~~~~~~~~~~

Implements functionality for interfacing with KIWAD archives.
This is a wrapper over the `katsuba-wad` Rust crate.
"""

from os import PathLike
from pathlib import Path
from typing import Iterator, Self

from katsuba.op import LazyObject, Serializer

class Archive:
    """
    A KingsIsle WAD archive.

    This class allows users to open archive files and read
    the data inside.
    """

    @classmethod
    def heap(cls, path: str | Path | PathLike) -> Self:
        """
        Opens an archive file and reads its contents into memory.

        Prefer :meth:`mmap` for very large files.

        :param path: The filesystem path to open the archive file at.
        :return: The archive instance.
        :raises OSError: Failed to read the archive file.
        :raises KatsubaError: Failed to parse the archive file.
        """

    @classmethod
    def mmap(cls, path: str | Path | PathLike) -> Self:
        """
        Opens and memory-maps an archive file at the given path.

        :param path: The filesystem path to open the archive file at.
        :return: The archive instance.
        :raises OSError: Failed to read the archive file.
        :raises KatsubaError: Failed to parse the archive file.
        """

    def __len__(self) -> int:
        """
        Indicates how many files are inside the archive.

        :return: The number of files.
        """

    def __contains__(self, path: str) -> bool:
        """
        Checks if the given file path is contained in the archive.

        :param path: The file path relative to the archive.
        :return: Whether the file is there or not.
        """

    def __getitem__(self, path: str) -> bytes:
        """
        Gets the contents of a file in the archive.

        :param path: The file path relative to the archive.
        :return: The decompressed contents of the file.
        :raises KeyError: The file was not found inside the archive.
        """

    def __iter__(self) -> Iterator[str]:
        """
        Iterates the files inside the archive.

        :return: An iterator yielding file paths from the archive.
        """

    def iter_glob(self, pattern: str) -> Iterator[str]:
        """
        Iterates the files matching the given glob pattern.

        :param pattern: The glob pattern to filter by.
        :return: An iterator yielding file paths from the archive.
        :raises KatsubaError: The glob pattern was invalid.
        """

    def deserialize(self, file: str, s: Serializer) -> LazyObject:
        """
        Deserializes a file in the archive with the given serializer.
        
        :param file: The file path relative to the archive.
        :param s: The serializer instance to use.
        :return: The deserialized object value.
        :raises OSError: I/O error occurred while trying to read data.
        :raises KatsubaError: Unknown type or invalid data format.
        """
