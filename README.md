# log-light-state

*log-light-state* is a tool to generate a CSV of all your hue light
states every minuit. The intention is that you'd use that log for
training neural networks and similar.

## Usage

``` shell
log-light-state 0.1.2
Billie Thompson <billie+git-mit@billiecodes.com>
log-light-state is a tool to generate a CSV of all your hue light states every minuit. The intention is that you'd use
that log for training neural networks and similar.

USAGE:
    log-light-state [OPTIONS] --hue-user <hue-user> --lat <latitude> --lon <longitude> --openweather-api-key <openweather-api-key>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -u, --hue-user <hue-user>                          Username to use on the hue hub to interact with the API
    -a, --lat <latitude>                               The latitude where the readings are being taken from
    -l, --lon <longitude>                              The longitude where the readings are being taken from
    -w, --openweather-api-key <openweather-api-key>    API key to use to interact with the OpenWeather API
    -o, --output <output>                              Where to write the CSV to [default: /dev/stdout]
    -p, --poll-every-seconds <poll>                    Poll every seconds [default: 60]
```
