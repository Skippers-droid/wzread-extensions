const fs = require('fs');
const path = require('path');
const os = require('os');
const esbuild = require('esbuild');
const { scanDirectoryForMaliciousFiles } = require('./security');

function loadConfig(configPath) {
  if (!fs.existsSync(configPath)) {
    console.warn('extension.maker.json not found in root, using default settings');
    return {
      extensionsDir: './extensions',
      outputDir: './dist/extensions',
      minFiles: 1,
      scriptType: 'js',
      concurrency: 4,
      strictSecurity: true
    };
  }

  try {
    const config = JSON.parse(fs.readFileSync(configPath, 'utf-8'));
    return {
      extensionsDir: config.extensionsDir || './extensions',
      outputDir: config.outputDir || './dist/extensions',
      minFiles: config.minFiles || 1,
      scriptType: config.scriptType || 'js',
      concurrency: config.concurrency || 4,
      strictSecurity: config.strictSecurity !== false
    };
  } catch (error) {
    console.error('Failed to parse extension.maker.json:', error);
    process.exit(1);
  }
}

async function bundleExtension(extensionPath, stagingDir, config) {
  const extName = path.basename(extensionPath);
  console.log(`Bundling extension: ${extName}`);

  if (config.strictSecurity) {
    console.log(`   Running security scan on ${extName}...`);
    const suspiciousFiles = scanDirectoryForMaliciousFiles(extensionPath, config.scriptType);
    
    if (suspiciousFiles.length > 0) {
      console.error(`   Security violations found in ${extName}:`);
      for (const file of suspiciousFiles) {
        console.error(`    File: ${file.file}`);
        for (const violation of file.violations) {
          console.error(`      - ${violation}`);
        }
      }
      console.error(`  Extension ${extName} blocked due to security violations`);
      return false;
    }
    console.log(`  Security scan passed for ${extName}`);
  }

  const extFile = path.join(extensionPath, 'index.js');
  if (!fs.existsSync(extFile)) {
    console.error(`Extension file not found: ${extFile}`);
    return false;
  }

  let pkg = {};
  const pkgPath = path.join(extensionPath, 'package.json');
  if (fs.existsSync(pkgPath)) {
    pkg = JSON.parse(fs.readFileSync(pkgPath, 'utf-8'));
  }

  const extStagingDir = path.join(stagingDir, extName);
  if (!fs.existsSync(extStagingDir)) {
    fs.mkdirSync(extStagingDir, { recursive: true });
  }

  const bundleOutput = path.join(extStagingDir, 'index.cjs');
  try {
    await esbuild.build({
      entryPoints: [extFile],
      bundle: true,
      minify: true,
      format: 'cjs',
      platform: 'node',
      target: 'node20',
      outfile: bundleOutput,
      external: ['electron', 'better-sqlite3'],
      logLevel: 'silent'
    });
  } catch (error) {
    console.error(`Failed to bundle ${extName}:`, error);
    return false;
  }

  const metaData = {
    name: pkg.name || extName,
    version: pkg.version || '1.0.0',
    description: pkg.description || '',
    author: pkg.author || '',
    main: 'index.cjs',
    bundled: true,
    bundledAt: new Date().toISOString()
  };

  let coverFound = false;
  try {
    const extContent = fs.readFileSync(extFile, 'utf-8');
    const coverMatch = extContent.match(/cover:\s*['"]([^'"]+)['"]/);
    if (coverMatch) {
      const coverPath = coverMatch[1];
      const fullCoverPath = path.join(extensionPath, coverPath);
      if (fs.existsSync(fullCoverPath)) {
        const ext = path.extname(fullCoverPath);
        const destPath = path.join(extStagingDir, `cover${ext}`);
        fs.copyFileSync(fullCoverPath, destPath);
        metaData.cover = `cover${ext}`;
        coverFound = true;
        console.log(`  ✓ Cover image found: ${coverPath}`);
      } else {
        console.warn(`  ⚠ Cover specified in extension_info but not found: ${coverPath}`);
      }
    }
  } catch (error) {}

  if (!coverFound) {
    console.warn(`  ⚠ No cover image found for extension: ${extName}`);
  }

  const assetsDir = path.join(extensionPath, 'assets');
  if (fs.existsSync(assetsDir)) {
    const assetDestDir = path.join(extStagingDir, 'assets');
    fs.mkdirSync(assetDestDir, { recursive: true });
    
    const copyDir = (src, dest) => {
      const items = fs.readdirSync(src);
      for (const item of items) {
        const srcPath = path.join(src, item);
        const destPath = path.join(dest, item);
        const stat = fs.statSync(srcPath);
        if (stat.isDirectory()) {
          fs.mkdirSync(destPath, { recursive: true });
          copyDir(srcPath, destPath);
        } else {
          fs.copyFileSync(srcPath, destPath);
        }
      }
    };
    copyDir(assetsDir, assetDestDir);
    metaData.assets = 'assets';
  }

  const metaFile = path.join(extStagingDir, 'extension.json');
  fs.writeFileSync(metaFile, JSON.stringify(metaData, null, 2));

  console.log(`Extension staged: ${extName}`);
  return true;
}

async function bundleAllExtensions(config) {
  const extensionsDir = path.resolve(config.extensionsDir);
  const outputDir = path.resolve(config.outputDir);
  const minFiles = config.minFiles;
  const scriptType = config.scriptType;
  const concurrency = config.concurrency;

  try {
    if (!fs.existsSync(extensionsDir)) {
      console.error(`Extensions directory not found: ${extensionsDir}`);
      return;
    }

    const items = fs.readdirSync(extensionsDir);
    const directories = items.filter(item => {
      const fullPath = path.join(extensionsDir, item);
      return fs.statSync(fullPath).isDirectory();
    });

    if (directories.length === 0) {
      console.log('No extensions found in extensions directory');
      return;
    }

    console.log(`Found ${directories.length} extension(s) to bundle\n`);
    
    const stagingDir = path.join(os.tmpdir(), 'extension-staging');
    if (fs.existsSync(stagingDir)) {
      fs.rmSync(stagingDir, { recursive: true, force: true });
    }
    fs.mkdirSync(stagingDir, { recursive: true });

    const validExtensions = [];

    for (const dir of directories) {
      const extPath = path.join(extensionsDir, dir);
      const jsFiles = getJavaScriptFiles(extPath, scriptType);
      
      if (jsFiles.length < minFiles) {
        console.log(`Skipping ${dir}: found ${jsFiles.length} ${scriptType} file(s), minimum required: ${minFiles}`);
        continue;
      }

      console.log(`Processing ${dir}: found ${jsFiles.length} ${scriptType} file(s)`);
      validExtensions.push(extPath);
    }

    if (validExtensions.length === 0) {
      console.log('No valid extensions to bundle');
      return;
    }

    console.log(`\nBundling ${validExtensions.length} extension(s) with concurrency: ${concurrency}\n`);

    let successCount = 0;
    let failCount = 0;
    const results = [];

    for (let i = 0; i < validExtensions.length; i += concurrency) {
      const batch = validExtensions.slice(i, i + concurrency);
      const batchPromises = batch.map(async (extPath) => {
        const extName = path.basename(extPath);
        const result = await bundleExtension(extPath, stagingDir, config);
        if (result) {
          successCount++;
          let pkg = {};
          const pkgPath = path.join(extPath, 'package.json');
          if (fs.existsSync(pkgPath)) {
            pkg = JSON.parse(fs.readFileSync(pkgPath, 'utf-8'));
          }
          results.push({
            name: pkg.name || extName,
            version: pkg.version || '1.0.0',
            description: pkg.description || '',
            author: pkg.author || ''
          });
        } else {
          failCount++;
        }
      });
      await Promise.all(batchPromises);
      
      console.log(`\nBatch ${Math.floor(i / concurrency) + 1} completed (${Math.min(i + concurrency, validExtensions.length)}/${validExtensions.length})`);
    }

    console.log(`\nBundling complete: ${successCount} succeeded, ${failCount} failed`);

    if (successCount === 0) {
      console.log('No extensions were bundled successfully');
      return;
    }

    if (!fs.existsSync(outputDir)) {
      fs.mkdirSync(outputDir, { recursive: true });
    }

    for (const ext of results) {
      const extStagingDir = path.join(stagingDir, ext.name);
      const extOutputDir = path.join(outputDir, ext.name);
      
      if (fs.existsSync(extOutputDir)) {
        fs.rmSync(extOutputDir, { recursive: true, force: true });
      }
      fs.mkdirSync(extOutputDir, { recursive: true });
      
      const items = fs.readdirSync(extStagingDir);
      for (const item of items) {
        const srcPath = path.join(extStagingDir, item);
        const destPath = path.join(extOutputDir, item);
        fs.copyFileSync(srcPath, destPath);
      }
    }

    await createExtensionManifest(outputDir, results);

    try {
      fs.rmSync(stagingDir, { recursive: true, force: true });
    } catch (error) {
      console.warn('Could not clean up staging directory:', error.message);
    }

    console.log('\nAll extensions bundled successfully!');
    console.log(`Manifest created: ${path.join(outputDir, 'wzread.mf.json')}`);
  } catch (error) {
    console.error('Error bundling extensions:', error);
  }
}

function getJavaScriptFiles(dir, scriptType) {
  const files = [];
  const items = fs.readdirSync(dir);
  
  for (const item of items) {
    const fullPath = path.join(dir, item);
    const stat = fs.statSync(fullPath);
    
    if (stat.isDirectory()) {
      files.push(...getJavaScriptFiles(fullPath, scriptType));
    } else if (stat.isFile()) {
      const ext = item.split('.').pop();
      if (scriptType === 'js' && (ext === 'js' || ext === 'mjs' || ext === 'cjs')) {
        files.push(fullPath);
      } else if (scriptType === 'ts' && (ext === 'ts' || ext === 'tsx')) {
        files.push(fullPath);
      } else if (scriptType === 'all') {
        files.push(fullPath);
      }
    }
  }
  
  return files;
}

async function createExtensionManifest(outputDir, extensions) {
  const manifest = {
    extensions: extensions.map(ext => ({
      name: ext.name,
      version: ext.version,
      description: ext.description,
      author: ext.author
    }))
  };

  const manifestPath = path.join(outputDir, 'wzread.mf.json');
  fs.writeFileSync(manifestPath, JSON.stringify(manifest, null, 2));
}

const configPath = path.join(process.cwd(), 'extension.maker.json');
const config = loadConfig(configPath);

console.log('Configuration:');
console.log(`  Extensions Directory: ${config.extensionsDir}`);
console.log(`  Output Directory: ${config.outputDir}`);
console.log(`  Minimum Files: ${config.minFiles}`);
console.log(`  Script Type: ${config.scriptType}`);
console.log(`  Concurrency: ${config.concurrency}`);
console.log(`  Strict Security: ${config.strictSecurity}`);
console.log('');

bundleAllExtensions(config).catch(console.error);