#!/usr/bin/env node
'use strict';

const version = 'v0.0.1'
const { ArgumentParser } = require('argparse');
//const { version } = require('./package.json');

const parser = new ArgumentParser({
  description: 'Argparse example'
});

parser.add_argument('-v', '--version', { action: 'version', version });
parser.add_argument('-f', '--foo', { help: 'foo bar' });
parser.add_argument('-b', '--bar', { help: 'bar foo' });
parser.add_argument('--baz', { help: 'baz bar' });

console.dir(parser.parse_args());