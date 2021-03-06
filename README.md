# Visonic-rs - a MQTT gateway to DSC like security panels

## Build

to build x86_64 docker image
```
make
```

to build for your arch with pre installed rust/cargo
```
cargo build --release
```

## Running

sample config [vs.toml](./vs.toml)

### Running in Docker
[docker-compose.yml](./docker-compose.yml)

### Running systemd
[visonic.service](./visonic.service)

## MQTT Commands
Command ARM the security, this will trigger exit sequence
```
mosquitto_pub -t /alarm/neo/cmd -m AWAY
```

Disarm
```
mosquitto_pub -t /alarm/neo/cmd -m DISARM
```

[Rest of the supported commands](./src/main.rs#L88)

## armv7 raspberry
docker image provided contains both x86_64 and armv7 binaries. For rpi
override command to
```
command: /visonic/visonic-arm
```

## License
[GPL V3](https://www.gnu.org/licenses/gpl-3.0.html)

## TODO
~~* armv7 build~~

## Credits
Communication protocol with tycomonitor.com has been borrowed from https://github.com/And3rsL/VisonicAlarm2