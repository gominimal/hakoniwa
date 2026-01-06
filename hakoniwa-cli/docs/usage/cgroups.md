# Usage - Control Groups

## --cgroup-cpu-shares

CPU shares

## --cgroup-cpu-period

CPU CFS period to be used for hardcapping

```console,ignore
$ hakoniwa run --cgroup-cpu-period 1000000 --cgroup-cpu-quota 1000000 -- stress -c $(nproc --all)
stress: info: [1] dispatching hogs: 8 cpu, 0 io, 0 vm, 0 hdd
...
```

## --cgroup-cpu-quota

CPU CFS hardcap limit

## --cgroup-memory-limit

Memory limit, in bytes

```console,ignore
$ hakoniwa run --cgroup-memory-limit 536870912 --cgroup-memory-reservation 536870912 --cgroup-memory-swap 536870912 -- stress --vm 4 --vm-bytes 256M
stress: info: [1] dispatching hogs: 0 cpu, 0 io, 4 vm, 0 hdd
stress: FAIL: [1] (425) <-- worker 5 got signal 9
stress: WARN: [1] (427) now reaping child worker processes
stress: FAIL: [1] (461) failed run completed in 0s
```

## --cgroup-memory-reservation

Memory soft limit, in bytes

## --cgroup-memory-swap

Memory+Swap limit, in bytes

## --cgroup-pids-limit

PID limit

```console,ignore
$ hakoniwa run -w . --cgroup-pids-limit 2 -- /bin/python3 ./tests/fixtures/scripts/fork-bomb.py
Traceback (most recent call last):
  File "/home/johndoe/Code/JohnDoe/foss/hakoniwa/hakoniwa-cli/./tests/fixtures/scripts/fork-bomb.py", line 4, in <module>
    os.fork()
    ~~~~~~~^^
BlockingIOError: [Errno 11] Resource temporarily unavailable
```
