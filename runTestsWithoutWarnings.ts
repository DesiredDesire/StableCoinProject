import { spawn } from 'child_process';

const forbiddenRegexps = [
  /Unable to find handler for subscription/,
  /has multiple versions, ensure that there is only one installed/,
  /Either remove and explicitly install matching versions or dedupe using your package manager/,
  /The following conflicting packages were found/,
  /cjs \d[.]\d[.]\d/,
];

const runTests = async () => {
  const command = 'npx';
  const args = ['redspot', 'test', '--no-compile'];
  return new Promise((resolve, reject) => {
    const process = spawn(command, args, { stdio: ['inherit', 'inherit', 'pipe'] });
    // process.stdout?.on('data', (data) => {
    //   if (forbiddenRegexps.every((reg) => !data.toString().match(reg))) console.log(data.toString());
    // });
    process.stderr?.on('data', (data) => {
      if (forbiddenRegexps.every((reg) => !data.toString().match(reg))) console.log(data.toString());
    });
    process.on('exit', function (code) {
      resolve(code);
    });
    process.on('error', function (err) {
      reject(err);
    });
  });
};

const argvObj = process.argv.reduce((acc, val, index) => {
  if (val.substring(0, 2) !== '--') return acc;
  acc[val.substring(2)] = process.argv[index + 1];
  return acc;
}, {} as Record<string, unknown>);

(async (args: Record<string, unknown>) => {
  if (require.main !== module) return;
  await runTests();
  process.exit(0);
})(argvObj).catch((e) => {
  console.log('ERROR');
  console.error(e);
  process.exit(0);
});
