# Katsuba

Python bindings to the [Katsuba](https://github.com/vbe0201/katsuba) libraries.

## Error handling

Most errors are represented either as native Python exceptions where it makes
sense, or as a `katsuba.KatsubaError` for custom errors from the Rust side.

## Bindings

### `katsuba.op`

Bindings to core functionality from the `katsuba-object-property` crate.

```py
from katsuba.op import *
from katsuba.utils import string_id

# Open a type list from file system
type_list = TypeList.open("types.json")

# Configure serializer options
opts = SerializerOptions()
opts.flags |= STATEFUL_FLAGS
opts.shallow = False

# Construct the serializer
ser = Serializer(opts, type_list)

# Deserialize a file
with open("TemplateManifest.xml", "rb") as f:
    manifest = f.read()
    assert manifest[:4] == b"BINd"

manifest = ser.deserialize(manifest[4:])

# Make sure we deserialized the right object:
assert manifest.type_hash == string_id("class TemplateManifest")

# Iterate the templates in the resulting object:
for location in manifest["m_serializedTemplates"]:
    print(f"Template {location['m_id']} at {location['m_filename']}")
```

### `katsuba.wad`

Bindings to core functionality from the `katsuba-wad` crate.

```py
from katsuba.op import Serializer
from katsuba.wad import Archive

# See `katsuba.op` above.
s = Serializer(...)

# Open an archive memory-mapped:
a = Archive.mmap("/path/to/Root.wad")

print(f"{len(a)} files in archive!")

# Deserialize a file out of the given archive:
if "TemplateManifest.xml" in a:
    a.deserialize("TemplateManifest.xml", s)

# Iterate over files in the archive and get their contents:
for path in a:
    data = a[path]

# With a glob pattern for filtering files:
for path in a.iter_glob("ObjectData/**/*.xml"):
    data = a[path]
```

### `katsuba.utils`

Bindings to useful components from the `katsuba-utils` crate.

For the time being, this features the hash functions `djb2` and `string_id`.
