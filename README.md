# log-light-state

*log-light-state* is a tool to generate a CSV of all your hue light
states every minuit. The intention is that you'd use that log for
training neural networks and similar.

## Usage

``` shell
log-light-state --help
log-light-state 0.1.0
billie


USAGE:
    log-light-state <hue-user> [output]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

ARGS:
    <hue-user>    Username to use on the hue hub to interact with the API
    <output>      Where to write the CSV to. Put - for stdout [default: /dev/stdout]
```
