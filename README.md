![Rust](https://github.com/nricciar/rigcycle/workflows/Rust/badge.svg)

# rigcycle

Used to control a group of receivers in SparkSDR to switch between defined profiles (e.g. day/night).

## Setup

Create `~/.rigcycle/config.yaml` with a list of defined receiver profiles.

### Example `~/.rigcycle/config.yaml` file

```
---
receivers:
  "localhost:51111":
    day:
      freq: 28074000
      mode: FT8
    night:
      freq: 3573000
      mode: FT8
```

## Run

```
$ rigcycle run day
```
