'use strict';

const { Command } = require('commander');
const { fetchSecrets } = require('../lib/api');

/**
 * Register the `kenv get` sub-command onto the parent `kenv` command.
 */
function makeKenvCommand() {
  const kenv = new Command('kenv');
  kenv.description('Interact with the karluiz kenv secrets service');

  kenv
    .command('get')
    .description('Retrieve secrets for an application and environment')
    .requiredOption('-a, --app <app>', 'Application name')
    .requiredOption('-e, --env <env>', 'Environment (e.g. prod, staging, dev)')
    .option(
      '-k, --api-key <key>',
      'API key for authentication (overrides KENV_API_KEY env var)'
    )
    .option('--json', 'Output the full JSON response instead of plain values')
    .action(async (opts) => {
      const apiKey = opts.apiKey || process.env.KENV_API_KEY;

      if (!apiKey) {
        console.error(
          'Error: API key is required. Pass --api-key or set the KENV_API_KEY environment variable.'
        );
        process.exitCode = 1;
        return;
      }

      try {
        const result = await fetchSecrets({
          app: opts.app,
          env: opts.env,
          apiKey,
        });

        if (opts.json) {
          console.log(JSON.stringify(result, null, 2));
        } else if (typeof result === 'object' && result !== null) {
          for (const [key, value] of Object.entries(result)) {
            console.log(`${key}=${value}`);
          }
        } else {
          console.log(result);
        }
      } catch (err) {
        console.error(`Error: ${err.message}`);
        process.exitCode = 1;
      }
    });

  return kenv;
}

module.exports = { makeKenvCommand };
