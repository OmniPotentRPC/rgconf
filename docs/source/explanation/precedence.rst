Precedence
==========


A field starts at its schema literal with status ``defaultValue``. A profile function then replaces it with ``profileDefault`` or, when the value is copied from another field, ``derivedFromUnset``. A user layer that sets the field wins over both. A higher user layer replaces a lower one and the status becomes ``layeredOverride``. One user layer is ``explicitlySet``. A winning deprecated alias is ``deprecatedAlias``. A ``ParamEdit`` is ``mutatedMidRun`` at the live edit layer.

``restartOnly`` fields reject a live edit. A checkpoint restart may change them. ``mutableLive`` edits record no hook. ``betweenStepsWithHook`` edits record the hook for that key: LBFGS history, the preconditioner, climbing images, band spacing, the thermostat, the potential, the dimer, or the cell.

Environment names are ``RGCONF_`` plus the canonical key, with a capital letter after a lower case letter gaining an underscore. ``band.imageCount`` is ``RGCONF_BAND_IMAGE_COUNT``. That variable is the total image count. The ``--images`` flag is the intermediate count and adds two.
