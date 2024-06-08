# dub-rs
Open source osu!bancho solution that written in rust.

## How to setup?

### Requirements
1. VPS with that has opened 80 and 443 ports and supports docker
2. Domain

```sh
$ git clone git@github.com:osu-lisek/dub-rs.git
$ scripts/preapre.sh
$ docker compose build
$ docker compose up -d
```