'use strict';

const { Command } = require('commander');
const { makeKenvCommand } = require('./commands/kenv');

const program = new Command();

program
  .name('karluiz-tool')
  .description('CLI for karluiz tools')
  .version('1.0.0');

program.addCommand(makeKenvCommand());

module.exports = program;
