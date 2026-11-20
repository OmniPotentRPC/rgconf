Resolve a client file
=====================


This walk resolves an eOn client file with the ``eonLegacyIni`` profile.


Write the file
==============



.. code-block:: ini

   [Main]
   job = minimization
   temperature = 300
   
   [Optimizer]
   opt_method = cg
   converged_force = 0.03
   max_move = 0.2
   
   [Refine]
   opt_method = fire
   threshold = 0.2
   
   [Nudged Elastic Band]
   images = 7


``images`` counts intermediate images. The resolver adds the two endpoints, so 7 becomes a total of 9.


Resolve it
==========



.. code-block:: bash

   rgconf resolve --profile eonLegacyIni --ini client.ini


The printed line for the band force is derived from the optimizer force, because that profile inherits the optimizer tolerance when the band value is unset. ``[Refine]`` is routed to ``optimizer.handoff`` while the job is minimization. A nudged elastic band job routes the same section to ``band.refine.handoff``.


Ask for the proof
=================



.. code-block:: bash

   rgconf explain --profile eonLegacyIni --ini client.ini


``explain`` appends the resolved fields to an RFC 9162 log and checks the inclusion proof of the first leaf with ``rfc9162::verify_inclusion``.
