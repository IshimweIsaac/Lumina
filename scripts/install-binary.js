const fs = require('fs');
const path = require('path');
const https = require('https');
const os = require('os');

const DOWNLOAD_BASE_URL = 'https://lumina-lang.web.app';
const VERSION = '1.8.0';

const platform = os.platform(); // 'linux', 'darwin', 'win32'
const arch = os.arch(); // 'x64', 'arm64'

const binaryMap = {
  'linux-x64': 'lumina-linux-x64',
  'linux-arm64': 'lumina-linux-arm64',
  'darwin-x64': 'lumina-macos-x64',
  'darwin-arm64': 'lumina-macos-arm64',
  'win32-x64': 'lumina-windows-x64.exe'
};

const key = `${platform}-${arch}`;
const remoteName = binaryMap[key];

if (!remoteName) {
  console.error(`Unsupported platform/architecture: ${key}`);
  process.exit(1);
}

const downloadUrl = `${DOWNLOAD_BASE_URL}/${remoteName}`;
const localBinPath = path.join(__dirname, '..', 'bin', 'lumina-bin');

if (platform === 'win32') {
  localBinPath += '.exe';
}

console.log(`Downloading Lumina v${VERSION} for ${key}...`);

const file = fs.createWriteStream(localBinPath);
https.get(downloadUrl, (response) => {
  if (response.statusCode !== 200) {
    console.error(`Failed to download binary: HTTP ${response.statusCode}`);
    process.exit(1);
  }
  response.pipe(file);
  file.on('finish', () => {
    file.close();
    if (platform !== 'win32') {
      fs.chmodSync(localBinPath, '755');
    }
    console.log('Successfully installed Lumina binary.');
  });
}).on('error', (err) => {
  fs.unlink(localBinPath);
  console.error(`Error downloading binary: ${err.message}`);
  process.exit(1);
});
