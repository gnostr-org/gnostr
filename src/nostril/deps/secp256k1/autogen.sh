#!/bin/sh
set -e
autoconf -if --warnings=all || autoreconf -if --warnings=all
