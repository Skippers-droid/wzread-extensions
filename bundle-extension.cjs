// bundle-extension.cjs
const fs = require('fs');
const path = require('path');
const os = require('os');
const { execSync } = require('child_process');

function loadConfig(configPath) {
  if (!fs.existsSync(configPath)) {
    console.warn('extension.maker.json not found in root, using default settings');
    return {
      extensionsDir: './extension',
      outputDir: './bundled-extensions',
      minFiles: 1,
      scriptType: 'rs',
      concurrency: 4,
      platform: process.platform === 'win32' ? 'windows' : 'linux',
      githubRepo: null
    };
  }

  try {
    const config = JSON.parse(fs.readFileSync(configPath, 'utf-8'));
    return {
      extensionsDir: config.extensionsDir || './extension',
      outputDir: config.outputDir || './bundled-extensions',
      minFiles: config.minFiles || 1,
      scriptType: config.scriptType || 'rs',
      concurrency: config.concurrency || 4,
      platform: config.platform || (process.platform === 'win32' ? 'windows' : 'linux'),
      githubRepo: config.githubRepo || null
    };
  } catch (error) {
    console.error('Failed to parse extension.maker.json:', error);
    process.exit(1);
  }
}

function ensureMainRs(extensionPath) {
  const srcDir = path.join(extensionPath, 'src');
  if (!fs.existsSync(srcDir)) {
    fs.mkdirSync(srcDir, { recursive: true });
  }

  const mainRs = path.join(srcDir, 'main.rs');
  const indexRs = path.join(srcDir, 'index.rs');
  const libRs = path.join(srcDir, 'lib.rs');

  if (fs.existsSync(indexRs) && !fs.existsSync(mainRs)) {
    fs.copyFileSync(indexRs, mainRs);
    console.log(`  ✓ Created main.rs from index.rs`);
    return mainRs;
  }

  if (fs.existsSync(libRs) && !fs.existsSync(mainRs)) {
    fs.copyFileSync(libRs, mainRs);
    console.log(`  ✓ Created main.rs from lib.rs`);
    return mainRs;
  }

  return mainRs;
}

async function buildExtension(extensionPath, config) {
  const extName = path.basename(extensionPath);
  console.log(`\nBuilding extension: ${extName}`);

  const cargoToml = path.join(extensionPath, 'Cargo.toml');
  if (!fs.existsSync(cargoToml)) {
    console.error(`  ✗ Cargo.toml not found for ${extName}`);
    return null;
  }

  ensureMainRs(extensionPath);

  let extFile = path.join(extensionPath, 'src', 'main.rs');
  if (!fs.existsSync(extFile)) {
    console.error(`  ✗ main.rs not found for ${extName}`);
    return null;
  }

  let pkg = {};
  const pkgPath = path.join(extensionPath, 'package.json');
  if (fs.existsSync(pkgPath)) {
    pkg = JSON.parse(fs.readFileSync(pkgPath, 'utf-8'));
  }

  const isWindows = config.platform === 'windows';
  const exeName = isWindows ? `${extName}.exe` : extName;

  try {
    console.log(`  Compiling ${extName} for ${config.platform}...`);
    
    const target = isWindows ? 'x86_64-pc-windows-msvc' : 'x86_64-unknown-linux-gnu';
    
    execSync(`cargo build --release --target ${target} -p ${extName}`, {
      cwd: path.dirname(cargoToml),
      stdio: 'inherit',
      timeout: 120000
    });

    let sourceExe = path.join(extensionPath, 'target', target, 'release', exeName);
    if (!fs.existsSync(sourceExe)) {
      const fallback = path.join(extensionPath, 'target', 'release', exeName);
      if (fs.existsSync(fallback)) {
        sourceExe = fallback;
      } else {
        console.error(`  ✗ Executable not found for ${extName}`);
        return null;
      }
    }

    console.log(`  ✓ Compiled successfully`);

    const stagingDir = path.join(os.tmpdir(), 'extension-staging', extName);
    if (fs.existsSync(stagingDir)) {
      fs.rmSync(stagingDir, { recursive: true, force: true });
    }
    fs.mkdirSync(stagingDir, { recursive: true });

    const destExe = path.join(stagingDir, exeName);
    fs.copyFileSync(sourceExe, destExe);
    if (!isWindows) {
      fs.chmodSync(destExe, 0o755);
    }

    const metaData = {
      name: pkg.name || extName,
      version: pkg.version || '1.0.0',
      description: pkg.description || '',
      author: pkg.author || '',
      executable: `./${extName}/${exeName}`,
      scriptPath: `./${extName}/${exeName}`,
      platform: config.platform,
      bundled: true,
      bundledAt: new Date().toISOString(),
      methods: ['search', 'manga_info', 'extension_info', 'chapter', 'getChapterImages', 'getPopular', 'getLatest', 'getFiltered']
    };

    let coverFound = false;
    let iconFound = false;
    
    try {
      const extContent = fs.readFileSync(extFile, 'utf-8');
      const coverMatch = extContent.match(/cover:\s*['"]([^'"]+)['"]/);
      if (coverMatch) {
        const coverPath = coverMatch[1];
        const fullCoverPath = path.join(extensionPath, coverPath);
        
        if (fs.existsSync(fullCoverPath)) {
          const ext = path.extname(fullCoverPath);
          const coverFileName = `cover${ext}`;
          const destPath = path.join(stagingDir, coverFileName);
          fs.copyFileSync(fullCoverPath, destPath);
          coverFound = true;
          console.log(`  ✓ Cover image found`);
          
          if (config.githubRepo) {
            metaData.cover = `${config.githubRepo}/bundled-extensions/${extName}/${coverFileName}`;
          } else {
            metaData.cover = `./${extName}/${coverFileName}`;
          }
        }
      }

      const iconMatch = extContent.match(/icon:\s*['"]([^'"]+)['"]/);
      if (iconMatch) {
        const iconPath = iconMatch[1];
        const fullIconPath = path.join(extensionPath, iconPath);
        
        if (fs.existsSync(fullIconPath)) {
          const ext = path.extname(fullIconPath);
          const iconFileName = `icon${ext}`;
          const destPath = path.join(stagingDir, iconFileName);
          fs.copyFileSync(fullIconPath, destPath);
          iconFound = true;
          console.log(`  ✓ Icon image found`);
          
          if (config.githubRepo) {
            metaData.icon = `${config.githubRepo}/bundled-extensions/${extName}/${iconFileName}`;
          } else {
            metaData.icon = `./${extName}/${iconFileName}`;
          }
        }
      }
    } catch (error) {}

    if (!coverFound) {
      console.warn(`  ⚠ No cover image found`);
    }

    if (!iconFound) {
      console.warn(`  ⚠ No icon image found`);
    }

    const assetsDir = path.join(extensionPath, 'assets');
    if (fs.existsSync(assetsDir)) {
      const assetDestDir = path.join(stagingDir, 'assets');
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
      
      if (config.githubRepo) {
        metaData.assets = `${config.githubRepo}/bundled-extensions/${extName}/assets`;
      } else {
        metaData.assets = `./${extName}/assets`;
      }
    }

    const metaFile = path.join(stagingDir, 'extension.json');
    fs.writeFileSync(metaFile, JSON.stringify(metaData, null, 2));

    console.log(`  ✓ ${extName} staged successfully`);
    
    return {
      name: extName,
      stagingDir: stagingDir,
      metaData: metaData,
      coverUrl: metaData.cover || null,
      iconUrl: metaData.icon || null
    };

  } catch (error) {
    console.error(`  ✗ Failed to build ${extName}:`, error.message);
    return null;
  }
}

async function bundleAllExtensions(config) {
  const extensionsDir = path.resolve(config.extensionsDir);
  const outputDir = path.resolve(config.outputDir);
  const minFiles = config.minFiles;
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
    
    const validExtensions = [];

    for (const dir of directories) {
      const extPath = path.join(extensionsDir, dir);
      const cargoToml = path.join(extPath, 'Cargo.toml');
      const srcDir = path.join(extPath, 'src');
      const mainRs = path.join(srcDir, 'main.rs');
      
      if (!fs.existsSync(cargoToml)) {
        console.log(`Skipping ${dir}: Cargo.toml not found`);
        continue;
      }
      
      if (!fs.existsSync(mainRs)) {
        console.log(`Skipping ${dir}: src/main.rs not found`);
        continue;
      }

      console.log(`Processing ${dir}`);
      validExtensions.push(extPath);
    }

    if (validExtensions.length === 0) {
      console.log('No valid extensions to bundle');
      return;
    }

    console.log(`\nBuilding ${validExtensions.length} extension(s) with concurrency: ${concurrency}\n`);

    let successCount = 0;
    let failCount = 0;
    const results = [];

    for (let i = 0; i < validExtensions.length; i += concurrency) {
      const batch = validExtensions.slice(i, i + concurrency);
      const batchPromises = batch.map(async (extPath) => {
        const result = await buildExtension(extPath, config);
        if (result) {
          successCount++;
          results.push({
            name: result.name,
            stagingDir: result.stagingDir,
            metaData: result.metaData,
            coverUrl: result.coverUrl,
            iconUrl: result.iconUrl
          });
        } else {
          failCount++;
        }
      });
      await Promise.all(batchPromises);
      
      console.log(`\nBatch ${Math.floor(i / concurrency) + 1} completed (${Math.min(i + concurrency, validExtensions.length)}/${validExtensions.length})`);
    }

    console.log(`\nBuild complete: ${successCount} succeeded, ${failCount} failed`);

    if (successCount === 0) {
      console.log('No extensions were built successfully');
      return;
    }

    if (!fs.existsSync(outputDir)) {
      fs.mkdirSync(outputDir, { recursive: true });
    }

    for (const ext of results) {
      const extOutputDir = path.join(outputDir, ext.name);
      
      if (fs.existsSync(extOutputDir)) {
        fs.rmSync(extOutputDir, { recursive: true, force: true });
      }
      fs.mkdirSync(extOutputDir, { recursive: true });
      
      const items = fs.readdirSync(ext.stagingDir);
      for (const item of items) {
        const srcPath = path.join(ext.stagingDir, item);
        const destPath = path.join(extOutputDir, item);
        const stat = fs.statSync(srcPath);
        if (stat.isDirectory()) {
          fs.cpSync(srcPath, destPath, { recursive: true });
        } else {
          fs.copyFileSync(srcPath, destPath);
        }
      }
    }

    const manifestExts = results.map(ext => {
      const entry = {
        name: ext.metaData.name,
        version: ext.metaData.version,
        description: ext.metaData.description,
        author: ext.metaData.author,
        scriptPath: ext.metaData.scriptPath,
        executable: ext.metaData.executable,
        platform: ext.metaData.platform
      };
      
      if (ext.coverUrl) {
        entry.cover = ext.coverUrl;
      }
      
      if (ext.iconUrl) {
        entry.icon = ext.iconUrl;
      }
      
      return entry;
    });

    await createExtensionManifest(outputDir, manifestExts, config);

    for (const ext of results) {
      try {
        fs.rmSync(ext.stagingDir, { recursive: true, force: true });
      } catch (error) {}
    }

    console.log('\nAll extensions bundled successfully!');
    console.log(`Manifest created: ${path.join(outputDir, 'wzread.mf.json')}`);
  } catch (error) {
    console.error('Error bundling extensions:', error);
  }
}

async function createExtensionManifest(outputDir, extensions, config) {
  const manifest = {
    extensions: extensions.map(ext => {
      const entry = {
        name: ext.name,
        version: ext.version,
        description: ext.description,
        author: ext.author,
        scriptPath: ext.scriptPath,
        executable: ext.executable,
        platform: ext.platform
      };
      
      if (ext.cover) {
        entry.cover = ext.cover;
      }
      
      if (ext.icon) {
        entry.icon = ext.icon;
      }
      
      return entry;
    })
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
console.log(`  Platform: ${config.platform}`);
if (config.githubRepo) {
  console.log(`  GitHub Repo: ${config.githubRepo}`);
}
console.log('');

bundleAllExtensions(config).catch(console.error);