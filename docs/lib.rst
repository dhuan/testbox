Testbox Library
###############

exec
====

Executes a shell command, receiving an object containing the output data and
the command's exit status.

.. code::

    print("The current folder is: " .. exec("pwd").stdout)

The returned object is structured as follows:

.. code::

   stdout: A string containing the output from stdout.
   stderr: A string containing the output from stderr.
   status: The exit status code from the executed command.


