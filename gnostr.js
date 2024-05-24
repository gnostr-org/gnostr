#!/usr/bin/env node

var shell = require('shelljs');

if (!shell.which('git')) {
  console.log(shel.which('git'));
  shell.echo('Sorry, this script requires git');
  shell.exit(1);
}
// console.log(shell.which('git'));
if (!shell.which('gnostr')) {
  console.log(shell.which('gnostr'));
  shell.echo('Sorry, this script requires gnostr');
  shell.exit(1);
}
// console.log(shell.which('gnostr'));
if (!shell.which('gnostr-post-event')) {
  console.log(shell.which('gnostr-post-event'));
  shell.echo('Sorry, this script requires gnostr-post-event');
  shell.exit(1);
}
// console.log(shell.which('gnostr-proxy'));
if (!shell.which('gnostr-proxy')) {
  console.log(shell.which('gnostr-proxy'));
  shell.echo('Sorry, this script requires gnostr');
  shell.exit(1);
}
// console.log(shell.which('gnostr-proxy'));
if (!shell.which('gnostr-sha256')) {
  console.log(shell.which('gnostr-sha256'));
  shell.echo('Sorry, this script requires gnostr-sha256');
  shell.exit(1);
}
// console.log(shell.which('gnostr-sha256'));

const { argv } = require('node:process');
const process = require('node:process');
const path = require('node:path');
const { cwd } = require('node:process');
// console.log(`cwd=${cwd()}`);

function help() {
    //console.log("gnostr-js");
  return 0;
}

function isHex(num) {
  return Boolean(num.match(/^0x[0-9a-f]+$/i))
}

function utf8ToHex(str) {
  return Array.from(str).map(c =>
    c.charCodeAt(0) < 128 ? c.charCodeAt(0).toString(16) :
    encodeURIComponent(c).replace(/\%/g,'').toLowerCase()
  ).join('');
}

function hexToUtf8(hex) {
  return decodeURIComponent('%' + hex.match(/.{1,2}/g).join('%'));
}

// print process.argv
argv.forEach((val, index) => {

    var secret  = {};
    var content = {};
    var tag     = {};
    var tags    = {};
    //console.log(`${index}`);

  if (`${index}` == 0)
    {
        //console.log(`${index}`);
        var ret = help();
        //console.log(`${ret}`);
        return 0;

    } else {}
  if (`${index}` == 1)
    {
        //console.log(`${index}`);
        var ret = help();
        //console.log(`${ret}`);
        return 0;

    } else {}
  if (`${index}` == 2)
    {
        if (`${val}` == `-h` || `${val}` == `--help`)
        {
            //console.log(`${index}: ${val}`);
            console.log(`HELP!!`);
            var ret = help();
            //console.log(`${ret}`);
            return 0;
        } else {}
    }
  if (`${index}` == 2)
    {
  if (`${val}` == `-v` || `${val}` == `--version`)
    {
        //console.log(`${index}: ${val}`);
        console.log(`VERSION!!`);
        return 0;

    } else {}

  if (`${val}` == `--sec` || `${val}` == `--secret`)
    {
        //const secret = utf8ToHex(process.argv[3] || 'e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855');
        const secret = (process.argv[3] || 'e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855');
        //console.log(typeof secret);
        const result = secret.padStart(64, '0');
        //console.log(typeof result);
        //console.log(isHex(result)) //=> true
        const resultHex = utf8ToHex(result);
        console.log(typeof resultHex);
        console.log(isHex(resultHex)) //=> true

        //console.log(isHex('0x1a')) //=> true
        //console.log(isHex('16'))   //=> false
        console.log('secret: ', result);
        return 0;

    } else {}

        //if ( body == "") {
        //var body = shell.exec('gnostr --sec $(gnostr-sha256) 2>/dev/null');
        //body = shell.exec('gnostr --sec $(gnostr-sha256)'); return 0;

        //console.log(body);
        //console.log(`${index}: ${val}`);

        //}
        //help();

        }
});
