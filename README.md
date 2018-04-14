# Kafka Topic Analyzer

[![Build Status](https://travis-ci.org/xenji/kafka-topic-analyzer.svg?branch=master)](https://travis-ci.org/xenji/kafka-topic-analyzer)
[![Current version on crates.io](https://img.shields.io/crates/v/kafka-topic-analyzer.svg)](https://crates.io/crates/kafka-topic-analyzer)
[![Build status](https://ci.appveyor.com/api/projects/status/2rkq0ydhexjwfgy8/branch/master?svg=true)](https://ci.appveyor.com/project/xenji/kafka-topic-analyzer/branch/master)

A CLI tool that gathers statistics about a Apache Kafka topic by reading
it from beginning to end and counting various metrics.

## Usage
    $> ./kafka-topic-analyzer -h
    Kafka Topic Analyzer

    USAGE:
        kafka-topic-analyzer --bootstrap-server <BOOTSTRAP_SERVER> --topic <TOPIC>

    FLAGS:
        -h, --help       Prints help information
        -V, --version    Prints version information

    OPTIONS:
        -b, --bootstrap-server <BOOTSTRAP_SERVER>    Bootstrap server(s) to work with, comma separated
        -t, --topic <TOPIC>                          The topic to analyze

## Example output
![Screenshot from a terminal that shows an example of the output](demo_output.png "Shows a sample output of the tool")

## Installation

By now, the only way of installing it is building it from source or
via `cargo install kafka-topic-analyzer`. In both cases you need to
have the rust toolchain installed via [http://rustup.rs/](http://rustup.rs/).
