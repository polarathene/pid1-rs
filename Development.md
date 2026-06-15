# Testing

All of these tests have been automated in [`pid1/tests/sanity.rs`](./pid1/tests/sanity.rs).

The [`pid1/examples` directory](./pid1/examples) contains several programs used for testing this library.

- `simple.rs`: A program that demonstrates the usage of the `pid1-rs` library.
- `zombie.rs`: Creates a zombie process.
- `sigterm_handler.rs`: A program that has a `SIGTERM` handler and exits upon receiving the signal.
- `sigterm_loop.rs`: A buggy program that does not exit on `SIGTERM`.
- `dumb_shell.rs`: An alternative shell for testing, since `bash` automatically reaps child processes.

## Environment setup

- Run the test program via the just recipe: `build-image`
- You can get shell access to it via the recipe: `exec-shell`

  ```console
  $ just exec-shell
  $ docker exec -it pid1rs sh

  $ ps aux
  PID   USER     TIME  COMMAND
      1 root      0:05 /simple
      7 root      0:00 /simple
      8 root      0:00 sh
    14 root      0:00 [date]
    15 root      0:00 ps aux
  ```

# Tests

## Basic functionality

```console
$ just test

$ docker run --name pid1rs pid1rstest
pid1-rs: Process running as PID 1
pid1-rs: Process not running as Pid 1: PID 7
In the simple process, going to sleep. Process ID is 7
Args: ["/simple"]
Wed Sep 27 08:29:35 UTC 2023
Wed Sep 27 08:29:37 UTC 2023
Wed Sep 27 08:29:39 UTC 2023
```

Ensure that the process exits with a status code of `0`. The test above confirms the following:

- The `simple` program was executed as PID 1.
- It relaunched itself as PID 7, which then executed various child processes.

## Zombie process

1. Run the `run-image` recipe:

    ```console
    $ just run-image
    $ docker rm pid1rs || exit 0
    pid1rs

    $ docker run --name pid1rs pid1rstest /simple --sleep
    pid1-rs: Process running as PID 1
    pid1-rs: Process not running as Pid 1: PID 7
    In the simple process, going to sleep. Process ID is 7
    Args: ["/simple", "--sleep"]
    Wed Sep 27 08:34:57 UTC 2023
    Wed Sep 27 08:34:59 UTC 2023
    Wed Sep 27 08:35:01 UTC 2023
    Going to sleep 500 seconds
    Wed Sep 27 08:35:03 UTC 2023
    ```

2. While it's executing, open a new shell and run the recipe to create a zombie process:

    ```console
    $ just run-zombie

    $ docker exec pid1rs zombie
    Process ID is 9
    Parent process: going to sleep and exit
    ```

3. You should see the following logs from the `simple` process, indicating that a process has been reaped:

    ```console
    Wed Sep 27 09:40:56 UTC 2023
    Reaped pid: 15
    ```

## Child Exit code status propagation

1. Run the `run-image` recipe:

    ```console
    $ just run-image
    $ docker rm pid1rs || exit 0
    pid1rs

    $ docker run --name pid1rs pid1rstest /simple --sleep
    pid1-rs: Process running as PID 1
    pid1-rs: Process not running as Pid 1: PID 7
    In the simple process, going to sleep. Process ID is 7
    Args: ["/simple", "--sleep"]
    Wed Sep 27 08:34:57 UTC 2023
    Wed Sep 27 08:34:59 UTC 2023
    Wed Sep 27 08:35:01 UTC 2023
    Going to sleep 500 seconds
    Wed Sep 27 08:35:03 UTC 2023
    ```

2. Run the `exec-shell` recipe and perform the test:

    ```console
    $ just exec-shell
    $ docker exec -it pid1rs sh
    $ ps -aux
    USER         PID %CPU %MEM    VSZ   RSS TTY      STAT START   TIME COMMAND
    root           1  0.0  0.0    996     4 pts/0    Ss+  12:34   0:00 /simple --sleep
    root           7  0.0  0.0    976     4 pts/0    S+   12:34   0:00 /simple --sleep
    root           8  0.0  0.0      0     0 pts/0    Z+   12:34   0:00 [date] <defunct>
    root           9  0.0  0.0   1672  1048 pts/1    Ss   12:34   0:00 sh
    root          14  0.0  0.0      0     0 pts/0    Z+   12:34   0:00 [date] <defunct>
    root          15  0.0  0.0      0     0 pts/0    Z+   12:34   0:00 [date] <defunct>
    root          16  0.0  0.0      0     0 pts/0    Z+   12:34   0:00 [date] <defunct>
    root          17  0.0  0.0   2460  1608 pts/1    R+   12:34   0:00 ps -aux

    $ kill -12 7
    ```

3. Check the logs of the `run-image` recipe to find the exit status code:

    ```console
    error: Recipe `run-image` failed on line 21 with exit code 140
    ```

    The exit code 140 is correct (128 + 12).

## SIGINT / SIGTERM handling

The goal of this test is to verify that the child process's `SIGTERM` handler is called if it is defined.

These are the signal codes:

- `SIGINT`: 2
- `SIGTERM`: 15

1. Execute the recipe `sigterm-test`:

    ```console
    $ just sigterm-test
    $ docker rm pid1rs || exit 0
    pid1rs

    $ docker run --name pid1rs pid1rstest sigterm_handler
    pid1-rs: Process running as PID 1
    pid1-rs: Process not running as Pid 1: PID 7
    This APP can be killed by SIGTERM (15)
    ```

2. Now, send `SIGTERM` to the PID 1 process:

    ```shell
    just send-sigterm
    ```

3. Confirm from the logs that the `SIGTERM` handler was called and that the process exited with a status of `0`:

    ```console
    App got SIGTERM 15, going to exit
    ```

Now, perform the same test with `SIGINT`. You can confirm that nothing is printed,
as this signal is not handled by the application.

## SIGTERM ignore

This test is for an application that ignores `SIGTERM` and continues running.
This library should be able to forcibly kill such processes.

1. Execute the recipe `sigloop-test`:

    ```console
    $ just sigloop-test
    $ docker rm pid1rs || exit 0
    pid1rs

    $ docker run --name pid1rs pid1rstest sigterm_loop
    pid1-rs: Process running as PID 1
    pid1-rs: Process not running as Pid 1: PID 7
    This APP ignores SIGTERM (15)
    ```

2. Now, send `SIGTERM` to the PID 1 process:

    ```shell
    just send-sigterm
    ```

3. You will see that it would have exited:

    ```console
    App got SIGTERM 15, but will not exit
    App got SIGTERM 15, but will not exit
    error: Recipe `sigloop-test` failed on line 44 with exit code 137
    ```

You can confirm from the status code (137) that the child process was killed by
`SIGKILL` (signal 9, exit code 128 + 9), because the application ignored `SIGTERM`.
