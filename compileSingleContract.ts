import { exec, spawn } from 'child_process';
import fs from 'fs-extra';
import path from 'path';
import glob from 'glob';
import util from 'node:util';
const execPromise = util.promisify(exec);

const createFileWithDirectoriesSync = (filePath: string, data: string) => {
  fs.ensureFileSync(filePath);
  fs.writeFileSync(filePath, data);
};

const getContractsFolderPath = (contractsRootPath: string, contractName: string) => {
  const paths = glob.sync(`${contractsRootPath}/**/Cargo.toml`);
  for (const p of paths) {
    const data = fs.readFileSync(p);
    if (data.includes(`[package]\nname = "${contractName}"`)) {
      console.log(`Found contract ${contractName}!`);
      return path.dirname(p);
    }
  }
  throw new Error(`Contract ${contractName} not found`);
};

const compileContract = async (contractPath: string) => {
  const command = 'cargo';
  const args = ['+nightly', 'contract', 'build'];
  console.log(`compiling contract...`);

  return new Promise((resolve, reject) => {
    const process = spawn(command, args, { cwd: contractPath, stdio: 'inherit' });
    process.stdout?.on('data', (data) => {
      console.log(data);
    });
    process.stderr?.on('data', (data) => {
      console.log(data);
    });
    process.on('exit', function (code) {
      resolve(code);
    });
    process.on('error', function (err) {
      reject(err);
    });
  });
};

const copyArtifacts = async (contractPath: string, contractName: string) => {
  const artifactsCompileOutputPath = path.join(contractPath, 'target', 'ink');
  const artifactsOutputPath = path.join('artifacts');
  console.log('Copying artifacts...');
  fs.ensureDirSync(artifactsOutputPath);
  fs.copyFileSync(
    path.join(artifactsCompileOutputPath, `${contractName}.contract`),
    path.join(artifactsOutputPath, `${contractName}.contract`)
  );
  fs.copyFileSync(path.join(artifactsCompileOutputPath, `metadata.json`), path.join(artifactsOutputPath, `${contractName}.json`));
};

const argvObj = process.argv.reduce((acc, val, index) => {
  if (val.substring(0, 2) !== '--') return acc;
  acc[val.substring(2)] = process.argv[index + 1];
  return acc;
}, {} as Record<string, unknown>);

(async (args: Record<string, unknown>) => {
  if (require.main !== module) return;
  const contractsRootPath = './contracts';
  const contractName = (args['name'] as string) ?? process.argv[2];
  const contractFolderPath = getContractsFolderPath(contractsRootPath, contractName);
  await compileContract(contractFolderPath);
  copyArtifacts(contractFolderPath, contractName);
  console.log('Success!');
  process.exit(0);
})(argvObj).catch((e) => {
  console.log('ERROR');
  console.error(e);
  process.exit(0);
});
