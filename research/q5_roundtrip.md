# Q5 round trip

2026-10-13, release build, Apple laptop, `eigenx` from the review branch.
Key generation used `--jobs 8`. Encapsulation and decapsulation are one thread.

| Step | real (s) | user (s) |
|---|---|---|
| keygen | 9824.93 | 8089.64 |
| encaps | 92.78 | 12.08 |
| decaps | 213.50 | 160.26 |

`cmp` of the two shared-secret files succeeded.

Keygen wall time is about 2 h 44 min. User time is below 8× wall time: the
eight attempts do not stay saturated for the whole search. Decapsulation is
the theta chain (about 3.5 min), not the earlier Mumford timing.
