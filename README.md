# Visonic-rs, is a MQTT gateway to DSC like security panels

Command ARM the security, this will trigger exit sequence
```
mosquitto_pub -t /alarm/neo/cmd -m AWAY
```

Disarm
```
mosquitto_pub -t /alarm/neo/cmd -m DISARM
```

## License
[GPL V3](https://www.gnu.org/licenses/gpl-3.0.html)

## Credits
Communication protocol with tycomonitor.com has been borrowed from https://github.com/And3rsL/VisonicAlarm2