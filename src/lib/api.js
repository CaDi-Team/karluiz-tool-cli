'use strict';

const https = require('https');
const http = require('http');

const BASE_URL = 'https://karluiz.com/api/env/orbital';

/**
 * Fetch secrets from the karluiz kenv API.
 * @param {object} options
 * @param {string} options.app   - Application name
 * @param {string} options.env   - Environment (e.g. prod, staging)
 * @param {string} options.apiKey - Bearer token for authentication
 * @returns {Promise<object>} Parsed JSON response
 */
function fetchSecrets({ app, env, apiKey }) {
  const url = new URL(BASE_URL);
  url.searchParams.set('app', app);
  url.searchParams.set('env', env);

  return new Promise((resolve, reject) => {
    const protocol = url.protocol === 'https:' ? https : http;
    const options = {
      hostname: url.hostname,
      port: url.port || (url.protocol === 'https:' ? 443 : 80),
      path: `${url.pathname}${url.search}`,
      method: 'GET',
      headers: {
        Authorization: `Bearer ${apiKey}`,
        Accept: 'application/json',
      },
    };

    const req = protocol.request(options, (res) => {
      let data = '';
      res.on('data', (chunk) => {
        data += chunk;
      });
      res.on('end', () => {
        if (res.statusCode < 200 || res.statusCode >= 300) {
          return reject(
            new Error(`Request failed with status ${res.statusCode}: ${data}`)
          );
        }
        try {
          resolve(JSON.parse(data));
        } catch {
          resolve(data);
        }
      });
    });

    req.on('error', (err) => reject(err));
    req.end();
  });
}

module.exports = { fetchSecrets, BASE_URL };
