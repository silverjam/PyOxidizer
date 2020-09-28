.. _config_type_python_packaging_policy:

=========================
``PythonPackagingPolicy``
=========================

When building a Python binary, there are various settings that control which
Python resources are added, where they are imported from, and other various
settings. This collection of settings is referred to as a *Python Packaging
Policy*. These settings are represented by the ``PythonPackagingPolicy`` type.

Instances of ``PythonPackagingPolicy`` have the following read-write
attributes:

``bytecode_optimize_level_zero``
   (``bool``) Whether to add Python bytecode at optimization level 0 (the
   default optimization level the Python interpreter compiles bytecode for).

``bytecode_optimize_level_one``
   (``bool``) Whether to add Python bytecode at optimization level 1.

``bytecode_optimize_level_two``
   (``bool``) Whether to add Python bytecode at optimization level 2.

``extension_module_filter``
   (``string``) The filter to apply to determine which extension modules to add.
   The following values are recognized:

   ``all``
      Every named extension module will be included.

   ``minimal``
      Return only extension modules that are required to initialize a
      Python interpreter. This is a very small set and various functionality
      from the Python standard library will not work with this value.

   ``no-libraries``
      Return only extension modules that don't require any additional libraries.

      Most common Python extension modules are included. Extension modules
      like ``_ssl`` (links against OpenSSL) and ``zlib`` are not included.

   ``no-gpl``
      Return only extension modules that do not link against GPL licensed
      libraries.

      Not all Python distributions may annotate license info for all extensions
      or the libraries they link against. If license info is missing, the
      extension is not included because it *could* be GPL licensed. Similarly,
      the mechanism for determining whether a license is GPL is based on an
      explicit list of non-GPL licenses. This ensures new GPL licenses don't
      slip through.

   Default is ``all``.

``include_distribution_sources``
   (``bool``) Whether to add source code for Python modules in the Python
   distribution.

   Default is ``True``.

``include_distribution_resources``
   (``bool``) Whether to add Python package resources for Python packages
   in the Python distribution.

   Default is ``False``.

``include_non_distribution_sources``
   (``bool``) Whether to add source code for Python modules not in the Python
   distribution.

``include_test``
   (``bool``) Whether to add Python resources related to tests.

   Not all files associated with tests may be properly flagged as such.
   This is a best effort setting.

   Default is ``False``.

``resources_policy``
   (``string``) The policy to apply when adding resources to the produced
   instance.

   See :ref:`config_python_resources_policy` for documentation on allowed
   values.

   Default is ``in-memory-only``.

``PythonPackagingPolicy`` instances have the following read-only attributes:

``preferred_extension_module_variants``
   ``(dict<string, string>)`` Mapping of extension module name to variant name.

   This mapping defines which preferred named variant of an extension module
   to use. Some Python distributions offer multiple variants of the same
   extension module. This mapping allows defining which variant of which
   extension to use when choosing among them.

   Keys set on this dict are not reflected in the underlying policy. To set
   a key, call the ``set_preferred_extension_module_variant()`` method.

``PythonPackagingPolicy`` instances have the following methods:

``register_resource_callback(func)``
   This method registers a Starlark function to be called when resource objects
   are created. The passed function receives 2 arguments: this
   ``PythonPackagingPolicy`` instance and the resource (e.g.
   ``PythonSourceModule``) that was created.

   The purpose of the callback is to enable Starlark configuration files to
   mutate resources upon creation so they can globally influence how those
   resources are packaged.

``set_preferred_extension_module_variant(name, value)``
   This method will set a preferred Python extension module variant to
   use. See the documentation for ``preferred_extension_module_variants``
   above for more.