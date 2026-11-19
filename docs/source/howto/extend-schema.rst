Extend the schema
=================


A third party adds a parameter without taking a new ordinal on ``RgConf``.


Companion schema
================


A compiled companion, such as ``gpridge.capnp``, imports ``rgconf.capnp`` and stores its extra fields beside ``canonical``. That is the typed form. The file id of the companion is its own. ``RgConf`` ordinals stay put.


Namespaced keys
===============


A key that no companion declared is a ``ThirdPartyExtension`` on ``RgConf.extensions`` (@39) and on ``RgConfPatch.extensions`` (@24). The canonical name is ``ext.<namespace>.<name>``.


.. code-block:: ini

   [ext.acme]
   alpha = 1.5


The resolver keeps that key, marks it mutable for a live edit, and does not invent a unit. Validation of bounds stays with the companion schema when the field is declared there.


What not to do
==============


Do not insert a field into the middle of ``RgConf``. Do not reuse an ordinal. Do not put a second copy of ``refine`` or ``thermostat`` at the top level. Those domains already have one owner.
