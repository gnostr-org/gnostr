#!/usr/bin/env node

var shell = require('shelljs');

if (!shell.which('git')) {
  console.log(shel.which('git'));
  shell.echo('Sorry, this script requires git');
  shell.exit(1);
}
// console.log(shell.which('git'));
if (!shell.which('gnostr')) {
  console.log(shel.which('gnostr'));
  shell.echo('Sorry, this script requires gnostr');
  shell.exit(1);
}
// console.log(shell.which('gnostr'));
if (!shell.which('gnostr-proxy')) {
  console.log(shel.which('gnostr-proxy'));
  shell.echo('Sorry, this script requires gnostr');
  shell.exit(1);
}
// console.log(shell.which('gnostr-proxy'));
if (!shell.which('gnostr-sha256')) {
  console.log(shel.which('gnostr-sha256'));
  shell.echo('Sorry, this script requires gnostr-sha256');
  shell.exit(1);
}
// console.log(shell.which('gnostr-sha256'));

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
