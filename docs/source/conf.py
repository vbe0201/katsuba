# Configuration file for the Sphinx documentation builder.
#
# For the full list of built-in configuration values, see the documentation:
# https://www.sphinx-doc.org/en/master/usage/configuration.html

from pathlib import Path

ROOT = Path(__file__).parent.parent.parent

# -- Project information -----------------------------------------------------
# https://www.sphinx-doc.org/en/master/usage/configuration.html#project-information

project = 'katsuba'
copyright = '2026, Valentin B'
author = 'Valentin B.'

# -- General configuration ---------------------------------------------------
# https://www.sphinx-doc.org/en/master/usage/configuration.html#general-configuration

nitpicky = True
nitpick_ignore = [
    ('py:class', 'dict[str, Any]'),
]

extensions = [
    'sphinx.ext.autodoc',
    'sphinx.ext.extlinks',
    'sphinx.ext.intersphinx',
    'sphinx.ext.napoleon',
    'sphinx_codeautolink',
    'sphinx_copybutton',
    'autoapi.extension',
]

autodoc_member_order = 'bysource'
autodoc_typehints = 'description'
autodoc_inherit_docstrings = False

intersphinx_mapping = {
    'python': ('https://docs.python.org/3', None),
}

extlinks = {
    'issue': ('https://github.com/vbe0201/katsuba/issues/%s', 'issue %s'),
}

codeautolink_global_preface = '''
import katsuba
from katsuba import *
'''

autoapi_dirs = [str(ROOT / 'src' / 'katsuba-py')]
autoapi_file_patterns = ['*.pyi']
autoapi_add_toctree_entry = False
autoapi_generate_api_docs = False
autoapi_python_class_content = 'class'
autoapi_options = [
    'members',
    'undoc-members',
    'show-inheritance',
    'show-module-summary',
]

templates_path = ['_templates']
exclude_patterns = ['Thumbs.db', '.DS_Store']



# -- Options for HTML output -------------------------------------------------
# https://www.sphinx-doc.org/en/master/usage/configuration.html#options-for-html-output

html_theme = 'sphinx_rtd_theme'
html_static_path = ['_static']
html_css_files = ["styles.css"]
