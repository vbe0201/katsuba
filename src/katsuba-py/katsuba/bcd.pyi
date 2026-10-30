"""
This module offers support for KingsIsle Binary Collision Data
(BCD) files. See :class:`Bcd` for more details.
"""

from os import PathLike
from typing import Final, Self

type Vec3 = list[float]

OBJECT: Final[int]
"""Collision flag bit for solid objects."""

WALKABLE: Final[int]
"""Collision flag bit for walkable ground."""

HITSCAN: Final[int]
"""Collision flag bit for hitscan geometry."""

LOCAL_PLAYER: Final[int]
"""Collision flag bit for the local player."""

WATER: Final[int]
"""Collision flag bit for water."""

CLIENT_OBJECT: Final[int]
"""Collision flag bit for client-side objects."""

TRIGGER: Final[int]
"""Collision flag bit for trigger volumes."""

FOG: Final[int]
"""Collision flag bit for fog."""

GOO: Final[int]
"""Collision flag bit for goo."""

FISH: Final[int]
"""Collision flag bit for fishing areas."""

MUCK: Final[int]
"""Collision flag bit for muck."""

class Face:
    """
    A triangle in a :class:`ProxyMesh`.

    This class has no public constructor. Instances are obtained from
    :attr:`ProxyMesh.faces`.
    """

    @property
    def face(self) -> list[int]:
        """
        Gets the indices of the triangle's corners in
        :attr:`ProxyMesh.vertices`.
        """

    @property
    def normal(self) -> Vec3:
        """
        Gets the normal vector of the triangle.
        """

class GeomParams:
    """
    Parameters for the geometric shape of a :class:`ProxyGeometry`.

    Every shape is represented by its own subclass, which allows for
    ``match`` statements on the shape type.
    """

    class Box(GeomParams):
        """Box-shaped geometry."""

        @property
        def length(self) -> float: ...
        @property
        def width(self) -> float: ...
        @property
        def depth(self) -> float: ...

    class Ray(GeomParams):
        """Ray-shaped geometry."""

        @property
        def position(self) -> float: ...
        @property
        def direction(self) -> float: ...
        @property
        def length(self) -> float: ...

    class Sphere(GeomParams):
        """Sphere-shaped geometry."""

        @property
        def radius(self) -> float: ...

    class Cylinder(GeomParams):
        """Cylinder-shaped geometry."""

        @property
        def radius(self) -> float: ...
        @property
        def length(self) -> float: ...

    class Tube(GeomParams):
        """Tube-shaped geometry."""

        @property
        def radius(self) -> float: ...
        @property
        def length(self) -> float: ...

    class Plane(GeomParams):
        """Plane-shaped geometry."""

        @property
        def normal(self) -> Vec3: ...
        @property
        def distance(self) -> float: ...

    class Mesh(GeomParams):
        """
        Mesh geometry.

        The mesh data is stored in :attr:`Collision.mesh`.
        """

class ProxyGeometry:
    """
    Representation of any geometric shape.

    This class has no public constructor. Instances are obtained from
    :attr:`Collision.geometry`.
    """

    @property
    def name(self) -> str:
        """
        Gets the name of the shape.
        """

    @property
    def rotation(self) -> list[Vec3]:
        """
        Gets the rows of the shape's rotation matrix.
        """

    @property
    def location(self) -> Vec3:
        """
        Gets the location of the shape.
        """

    @property
    def scale(self) -> float:
        """
        Gets the scaling factor of the shape.
        """

    @property
    def material(self) -> str:
        """
        Gets the material name of the shape.
        """

    @property
    def params(self) -> GeomParams:
        """
        Gets the parameters of the geometric shape.
        """

class ProxyMesh:
    """
    Representation of an arbitrary triangle mesh.

    This class has no public constructor. Instances are obtained from
    :attr:`Collision.mesh`.
    """

    @property
    def vertices(self) -> list[Vec3]:
        """
        Gets the vertices of the mesh.
        """

    @property
    def faces(self) -> list[Face]:
        """
        Gets the triangles of the mesh.
        """

class Collision:
    """
    An individual collision shape and its associated metadata.

    This class has no public constructor. Instances are obtained from
    :attr:`Bcd.collisions`.
    """

    @property
    def category_flags(self) -> int:
        """
        Gets the category flags of the shape.

        This is a mask of the collision flag bits in this module, such
        as :data:`WALKABLE`.
        """

    @property
    def collision_flags(self) -> int:
        """
        Gets the collision flags of the shape.

        This is a mask of the collision flag bits in this module, such
        as :data:`OBJECT`.
        """

    @property
    def mesh(self) -> ProxyMesh | None:
        """
        Gets the mesh data of the shape, if it is a
        :class:`GeomParams.Mesh`.
        """

    @property
    def geometry(self) -> ProxyGeometry:
        """
        Gets the geometric data of the shape.
        """

class Bcd:
    """
    Representation of a Binary Collision Data file.

    These are usually found as ``collision.bcd`` files in zone archives
    and describe the collision shapes of a zone.

    This class has no public constructor. To obtain an instance, use
    :meth:`parse` or :meth:`open`.
    """

    @classmethod
    def parse(cls, data: bytes) -> Self:
        """
        Parses a BCD file from the given bytes.

        :param data: The raw file contents.
        :return: The parsed file.
        :raises KatsubaError: The data is malformed.
        """

    @classmethod
    def open(cls, path: str | PathLike[str]) -> Self:
        """
        Reads and parses a BCD file from a given path.

        :param path: The filesystem path to the file.
        :return: The parsed file.
        :raises OSError: Failed to read the given file.
        :raises KatsubaError: The file is malformed.
        """

    @property
    def collisions(self) -> list[Collision]:
        """
        Gets all collision shapes in the file.
        """
