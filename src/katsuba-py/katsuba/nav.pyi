"""
This module offers functionality for reading KingsIsle Navigation
Graph (NAV) files. See :class:`NavigationGraph` and
:class:`ZoneNavigationGraph` for more details.
"""

from os import PathLike
from typing import Self

class NavigationNode:
    """
    A node in a :class:`NavigationGraph`.

    This class has no public constructor. Instances are obtained from
    :attr:`NavigationGraph.nodes`.
    """

    @property
    def location(self) -> list[float]:
        """
        Gets the location of the node.
        """

    @property
    def id(self) -> int:
        """
        Gets the unique identifier of the node.
        """

class NavigationLink:
    """
    A link between two :class:`NavigationNode` objects in a graph.

    This class has no public constructor. Instances are obtained from
    :attr:`NavigationGraph.links`.
    """

    @property
    def first(self) -> int:
        """
        Gets the :attr:`NavigationNode.id` of the first node.
        """

    @property
    def second(self) -> int:
        """
        Gets the :attr:`NavigationNode.id` of the second node.
        """

class NavigationGraph:
    """
    A graph of navigation nodes and their interconnections.

    These are usually found as ``zone.nav`` files in zone archives and
    describe the walkable paths within a single zone.

    This class has no public constructor. To obtain an instance, use
    :meth:`parse` or :meth:`open`.
    """

    @classmethod
    def parse(cls, data: bytes) -> Self:
        """
        Parses a navigation graph from the given bytes.

        :param data: The raw file contents.
        :return: The parsed graph.
        :raises KatsubaError: The data is malformed.
        """

    @classmethod
    def open(cls, path: str | PathLike[str]) -> Self:
        """
        Reads and parses a navigation graph from a given path.

        :param path: The filesystem path to the file.
        :return: The parsed graph.
        :raises OSError: Failed to read the given file.
        :raises KatsubaError: The file is malformed.
        """

    @property
    def nodes(self) -> list[NavigationNode]:
        """
        Gets the nodes of the graph.
        """

    @property
    def links(self) -> list[NavigationLink]:
        """
        Gets the links between the nodes of the graph.
        """

class ZoneNavigationGraph:
    """
    A navigation graph across zones.

    This is usually found as ``zonemap.nav`` in the ``Root.wad`` archive.
    Every node represents a zone, whose name is looked up by using the
    node's :attr:`NavigationNode.id` in :attr:`graph` as an index into
    :attr:`zone_names`.

    This class has no public constructor. To obtain an instance, use
    :meth:`parse` or :meth:`open`.
    """

    @classmethod
    def parse(cls, data: bytes) -> Self:
        """
        Parses a zone navigation graph from the given bytes.

        :param data: The raw file contents.
        :return: The parsed graph.
        :raises KatsubaError: The data is malformed.
        """

    @classmethod
    def open(cls, path: str | PathLike[str]) -> Self:
        """
        Reads and parses a zone navigation graph from a given path.

        :param path: The filesystem path to the file.
        :return: The parsed graph.
        :raises OSError: Failed to read the given file.
        :raises KatsubaError: The file is malformed.
        """

    @property
    def graph(self) -> NavigationGraph:
        """
        Gets the graph of zones.
        """

    @property
    def zone_names(self) -> list[str]:
        """
        Gets the names of the zones covered by the graph.
        """
