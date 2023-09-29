#!/usr/bin/env node

var shell = require('shelljs');

if (!shell.which('git')) {
  shell.echo('installing git...');
  shell.exec('type -P apt-get && apt-get install git || type -P brew && brew install git')
} else {
  //console.log(shell.which('git'));
}
if (!shell.which('npm')) {
  shell.echo('installing npm...');
  shell.exec('type -P apt-get && apt-get install npm || type -P brew && brew install npm')
} else {
  //console.log(shell.which('npm'));
}
if (!shell.which('jq')) {
  shell.echo('installing jq...');
  shell.exec('type -P apt-get && apt-get install jq || type -P brew && brew install jq')
} else {
  //console.log(shell.which('jq'));
}
if (!shell.which('make')) {
  shell.echo('installing make...');
  shell.exec('type -P apt-get && apt-get install make || type -P brew && brew install make')
} else {
  //console.log(shell.which('make'));
}
if (!shell.which('cmake')) {
  shell.echo('installing cmake...');
  shell.exec('type -P apt-get && apt-get install cmake || type -P brew && brew install cmake')
} else {
  //console.log(shell.which('cmake'));
}
if (!shell.which('gnostr')) {
  shell.echo('installing gnostr...');
  shell.exec('mkdir -p .gnostr');
  shell.exec('cd .gnostr && \
  git init && \
  git add remote origin https://github.com/gnostr-org/gnostr.git && \
  git pull -f origin master');
  shell.exec('cd .gnostr && make gnostr-all && make gnostr-install')
    if (!shell.which('gnostr-act')) {}
    if (!shell.which('gnostr-blockheight')) {}
    if (!shell.which('gnostr-cat')) {}
    if (!shell.which('gnostr-cli')) {}
    if (!shell.which('gnostr-client')) {}
    if (!shell.which('gnostr-command')) {}
    if (!shell.which('gnostr-dataurl')) {}
    if (!shell.which('gnostr-fs')) {}
    if (!shell.which('gnostr-function')) {}
    if (!shell.which('gnostr-get-relays')) {}
    if (!shell.which('gnostr-git')) {}
    if (!shell.which('gnostr-git-instweb')) {}
    if (!shell.which('gnostr-git-log')) {}
    if (!shell.which('gnostr-sha256')) {}
} else {
  console.log(shell.which('gnostr'));
  console.log(shell.which('gnostr-act'));
  console.log(shell.which('gnostr-blockheight'));
  console.log(shell.which('gnostr-cat'));
  console.log(shell.which('gnostr-cli'));
  console.log(shell.which('gnostr-client'));
  console.log(shell.which('gnostr-command'));
  console.log(shell.which('gnostr-dataurl'));
  console.log(shell.which('gnostr-fs'));
  console.log(shell.which('gnostr-function'));
  console.log(shell.which('gnostr-get-relays'));
  console.log(shell.which('gnostr-git'));
  console.log(shell.which('gnostr-git-instaweb'));
  console.log(shell.which('gnostr-git-log'));
  console.log(shell.which('gnostr-sha256'));
}



var body = shell.exec('gnostr --sec $(gnostr-sha256)');
//console.log(body);


const { argv } = require('node:process');
const process = require('node:process');
const path = require('node:path');
const { cwd } = require('node:process');
// console.log(`cwd=${cwd()}`);

// print process.argv
argv.forEach((val, index) => {

  if (`${val}` == `-h`)
  { console.log(`HELP!!`); }
  else
  { console.log(`${index}: ${val}`); }

});
