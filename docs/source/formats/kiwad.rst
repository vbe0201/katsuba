.. _kiwad:

WAD Archives
------------

WAD is the most foundational and well-explored format in KingsIsle games.
Many independent tools have supported them for years due to their simplicity,
but few tools go beyond that.

Synopsis
~~~~~~~~

WAD is a proprietary archive format used by KingsIsle to distribute virtually
all game assets. Think of a ZIP archive but ✨ magic ✨. The game client loads
all of its data directly from WAD archives without ever extracting them to disk.

.. note::

    There is a naming overlap of the format with other games, most notably
    `Doom <https://doom.fandom.com/wiki/WAD>`_. KingsIsle may have derived
    inspiration from this format due to its popularity, but there is no
    relation otherwise.

A low-level file format specification for the curious can be found
`here <https://kronos-project.github.io/grimoire/internals/kiwad.html>`_.

Location
~~~~~~~~

All WAD files are located inside the installation directory of your game, in
the ``Data/GameData`` subdirectory.

Many of them are downloaded by the patch client after logging into your game,
some are downloaded on the fly while you are playing due to their large size.
If you've ever seen boxes standing around in houses which eventually turn into
something else, that is an example of WAD contents being downloaded dynamically.

.. note::

    If you notice particular data is missing during your work, it is a good idea
    to make sure you actually have it downloaded.

Some particular files worth mentioning:

.. list-table::
    :widths: 45 55
    :header-rows: 1

    * - Archive file
      - Contents
    * - ``Root.wad``
      - Data definitions for gear, spells, mounts, pets, mobs, but also locale
        data and other core game assets.
    * - ``_Shared-WorldData.wad``
      - Graphical assets such as equipment icons, spell icons, GUI definitions.
    * - ``Mob-WorldData.wad``\ /\ ``Mob2-WorldData.wad``
      - 3D models for pets, mounts, invisible gear (e.g. rings), and NPCs.
    * - ``Music-WorldData.wad``
      - All game music.

Workflow
~~~~~~~~

In order to inspect or modify any of the game assets, it is necessary to unpack
(and potentially repack) archives. The ``katsuba`` application provides just the
right tools for the job.

.. warning::

    Since WAD archives can technically store anything, including malware, it is
    important to work with archives obtained from trusted sources only.

In order to unpack a file (``Root.wad`` in this case), use the following command:

.. code-block::

    katsuba wad unpack path/to/Root.wad --out output

You should now see an ``output/Root`` directory containing all the files in the
original ``Root.wad`` and you can start studying and editing them.

.. note::

    When making changes, it is a good idea to keep a backup of the original file
    in case you mess something up and want to go back.

When you're done and want to repack the archive again, use the following command:

.. code-block::

    katsuba wad pack output/Root --flags 1 -o Root.wad

If you would like to extract only a certain subset of files in the archive, that
is possible two by using a
`UNIX-style glob pattern <https://en.wikipedia.org/wiki/Glob_(programming)>`_\ :

.. code-block::

    katsuba wad unpack path/to/Root.wad --glob "**/*Messages.xml" -o outdir

This command unpacks only those files from ``Root.wad`` whose names end with
``Messages.xml``.
