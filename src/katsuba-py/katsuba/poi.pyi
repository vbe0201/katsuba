"""
This module offers functionality for reading KingsIsle Point of
Interest (POI) files. See :class:`Poi` for more details.
"""

from os import PathLike
from typing import Self

class Point:
    """
    An event point inside a :class:`Poi` file.

    This class has no public constructor. Instances are obtained from
    :attr:`Poi.goals`.
    """

    @property
    def no_quest_helper(self) -> bool:
        """
        Gets whether the quest helper ignores this point.
        """

    @property
    def zone_id(self) -> int:
        """
        Gets the ID of the zone this point is part of.

        This is an index into :attr:`Poi.zone_names`.
        """

    @property
    def template_id(self) -> int:
        """
        Gets the template ID associated with this point.
        """

    @property
    def location(self) -> list[float]:
        """
        Gets the location of this point.
        """

    @property
    def interactable(self) -> bool:
        """
        Gets whether this point is an interactable NPC.
        """

    @property
    def collectable(self) -> bool:
        """
        Gets whether this point is a collectable item.
        """

class Teleporter:
    """
    A teleporter entry between zones in a :class:`Poi` file.

    This class has no public constructor. Instances are obtained from
    :attr:`Poi.teleporters`.
    """

    @property
    def destination(self) -> str:
        """
        Gets the name of the destination zone.
        """

    @property
    def position(self) -> list[float]:
        """
        Gets the position of the teleporter in its zone.
        """

class Poi:
    """
    Representation of a Point of Interest file.

    This is usually found as ``poi.dat`` in the ``Root.wad`` archive and
    describes locations of quest goals and teleporters between zones.

    Zones are identified by an index into :attr:`zone_names` in
    :attr:`Point.zone_id`, but by :func:`katsuba.utils.string_id` of
    their names in the keys of all other mappings.

    This class has no public constructor. To obtain an instance, use
    :meth:`parse` or :meth:`open`.
    """

    @classmethod
    def parse(cls, data: bytes) -> Self:
        """
        Parses a POI file from the given bytes.

        :param data: The raw file contents.
        :return: The parsed file.
        :raises KatsubaError: The data is malformed.
        """

    @classmethod
    def open(cls, path: str | PathLike[str]) -> Self:
        """
        Reads and parses a POI file from a given path.

        :param path: The filesystem path to the file.
        :return: The parsed file.
        :raises OSError: Failed to read the given file.
        :raises KatsubaError: The file is malformed.
        """

    @property
    def zone_names(self) -> list[str]:
        """
        Gets the names of all zones described by this file.
        """

    @property
    def goals(self) -> dict[int, Point]:
        """
        Gets a mapping of goal IDs to their respective points.
        """

    @property
    def interactive_goals(self) -> dict[int, list[int]]:
        """
        Gets a mapping of zone name hashes to lists of interactable template IDs.
        """

    @property
    def teleporters(self) -> dict[int, list[Teleporter]]:
        """
        Gets a mapping of zone name hashes to the teleporters in that zone.
        """

    @property
    def goal_adjectives(self) -> dict[int, list[int]]:
        """
        Gets a mapping of goal IDs to goal adjectives.
        """

    @property
    def zone_mobs(self) -> dict[int, list[str]]:
        """
        Gets a mapping of zone name hashes to the mob assets in that zone.
        """
