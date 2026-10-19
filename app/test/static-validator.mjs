import { execFileSync } from "node:child_process";
// CSP forbids dynamic Function/eval; Node's equivalent enforces the same contract.
execFileSync(
  process.execPath,
  [
    "--disallow-code-generation-from-strings",
    "--input-type=module",
    "-e",
    `import validate from './src/domain/paper-validator.js';import {readFileSync} from 'node:fs';const p=JSON.parse(readFileSync('test/fixtures/dev-reading.json'));if(!validate(p)||validate({}))process.exit(1);console.log('Static validator passes with code generation forbidden');`,
  ],
  { stdio: "inherit" },
);
