.. _api:

API Reference
=============

.. module:: katsuba

The following section provides an overview of the katsuba Python API. Links
to other parts of the documentation are provided when additional context is
useful.

Please note that this page does not host any Rust API reference. Rust users
are encouraged to build the dedicated rustdoc themselves.

Overview
--------

.. autoapimodule:: katsuba
    :no-index:

Exceptions
----------

As a rule of thumb, all errors that are reported from the Rust side are mapped
to appropriate builtin exception types (e.g. :external:exc:`OSError` for I/O
related issues). Individual API references provide further exception details.

Errors without an appropriate Python counterpart are reported under the umbrella
of :exc:`KatsubaError`.

.. autoapiexception:: KatsubaError

Utilities
---------

.. autoapimodule:: katsuba.utils

.. autoapifunction:: string_id

.. autoapifunction:: djb2

ObjectProperty
--------------

.. autoapimodule:: katsuba.op

.. autoapidata:: STATEFUL_FLAGS

.. autoapidata:: COMPACT_LENGTH_PREFIXES

.. autoapidata:: HUMAN_READABLE_ENUMS

.. autoapidata:: WITH_COMPRESSION

.. autoapidata:: FORBID_DELTA_ENCODE

.. autoapiclass:: Object
    :show-inheritance:
    :members: type_hash
    :special-members:

.. autoapiclass:: TypeList
    :members:

.. autoapiclass:: SerializerOptions
    :members:

.. autoapiclass:: Serializer
    :members:

KIWAD Archives
--------------

.. autoapimodule:: katsuba.wad

.. autoapiclass:: Archive
    :members:
    :special-members:
