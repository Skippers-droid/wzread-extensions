// tester.js
const { exec, spawn } = require('child_process');
const fs = require('fs');
const path = require('path');
const readline = require('readline');

const rl = readline.createInterface({
    input: process.stdin,
    output: process.stdout
});

const EXTENSIONS_DIR = path.join(__dirname, 'bundled-extensions');
const MANIFEST_PATH = path.join(EXTENSIONS_DIR, 'wzread.mf.json');

let selectedExtension = null;
let extensionExe = null;
let methods = [];

function log(message, type = 'info') {
    const colors = {
        info: '\x1b[36m',
        success: '\x1b[32m',
        error: '\x1b[31m',
        warning: '\x1b[33m',
        debug: '\x1b[90m',
        reset: '\x1b[0m'
    };
    console.log(`${colors[type]}${message}${colors.reset}`);
}

function question(query) {
    return new Promise((resolve) => {
        rl.question(query, resolve);
    });
}

async function loadExtensions() {
    try {
        const manifestData = fs.readFileSync(MANIFEST_PATH, 'utf8');
        const manifest = JSON.parse(manifestData);
        return manifest.extensions;
    } catch (error) {
        log(`Failed to load manifest: ${error.message}`, 'error');
        return [];
    }
}

async function selectExtension(extensions) {
    log('\n📚 Available Extensions:', 'info');
    extensions.forEach((ext, index) => {
        const exePath = path.join(EXTENSIONS_DIR, ext.id, `${ext.id}.exe`);
        const status = fs.existsSync(exePath) ? '✅' : '❌';
        log(`  ${index + 1}. ${status} ${ext.name} (${ext.id})`, 'info');
    });

    const choice = await question('\nSelect extension (number): ');
    const index = parseInt(choice) - 1;
    
    if (index >= 0 && index < extensions.length) {
        selectedExtension = extensions[index];
        const exePath = path.join(EXTENSIONS_DIR, selectedExtension.id, `${selectedExtension.id}.exe`);
        
        if (fs.existsSync(exePath)) {
            extensionExe = exePath;
            methods = ['search', 'getPopular', 'getLatest', 'getFiltered', 'manga_info', 'get_chapter_images', 'extension_info'];
            log(`\n✅ Selected: ${selectedExtension.name}`, 'success');
            return true;
        } else {
            log(`\n❌ Executable not found: ${exePath}`, 'error');
            log('Please build the extension first using bundle-extension.cjs', 'warning');
            return false;
        }
    }
    
    log('Invalid selection', 'error');
    return false;
}

function executeMethod(method, args = []) {
    return new Promise((resolve, reject) => {
        const startTime = performance.now();
        
        // For RPC mode, we need to start the extension with --rpc flag first
        const isRpc = true;
        
        log(`\n🚀 Executing: ${method}`, 'info');
        log(`📝 Command: "${extensionExe}" --rpc`, 'debug');
        
        const child = spawn(extensionExe, ['--rpc'], {
            env: { 
                ...process.env, 
                USER_AGENT: 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36',
                RUST_LOG: 'debug'
            },
            stdio: ['pipe', 'pipe', 'pipe']
        });

        let stdout = '';
        let stderr = '';
        let port = 0;
        let logs = [];
        let resolved = false;

        child.stdout.on('data', (data) => {
            const output = data.toString();
            stdout += output;
            
            // Check for RPC_PORT
            const portMatch = output.match(/RPC_PORT=(\d+)/);
            if (portMatch) {
                port = parseInt(portMatch[1]);
                log(`🔌 RPC server started on port ${port}`, 'success');
                
                // Now make RPC call
                makeRpcCall(port, method, args).then(result => {
                    if (!resolved) {
                        resolved = true;
                        child.kill();
                        resolve(result);
                    }
                }).catch(err => {
                    if (!resolved) {
                        resolved = true;
                        child.kill();
                        reject(err);
                    }
                });
            }
            
            // Display any JSON output
            try {
                const parsed = JSON.parse(output);
                log(`📤 JSON Output:`, 'success');
                console.log(JSON.stringify(parsed, null, 2));
            } catch {
                if (output.trim() && !output.includes('RPC_PORT')) {
                    log(`📤 Output:`, 'info');
                    console.log(output);
                }
            }
        });

        child.stderr.on('data', (data) => {
            const output = data.toString();
            stderr += output;
            
            const lines = output.split('\n');
            for (const line of lines) {
                if (line.trim()) {
                    if (line.includes('[extension]')) {
                        log(`🔍 ${line.trim()}`, 'debug');
                        logs.push(line.trim());
                    } else if (line.includes('Error') || line.includes('error')) {
                        log(`❌ ${line.trim()}`, 'error');
                    } else if (line.includes('Warning') || line.includes('warning')) {
                        log(`⚠️  ${line.trim()}`, 'warning');
                    } else if (line.trim()) {
                        log(`📝 ${line.trim()}`, 'debug');
                    }
                }
            }
        });

        child.on('close', (code) => {
            const endTime = performance.now();
            const duration = (endTime - startTime).toFixed(2);
            
            if (!resolved) {
                resolved = true;
                log(`\n⏱️  Execution time: ${duration}ms`, 'info');
                log(`📊 Exit code: ${code}`, code === 0 ? 'success' : 'error');
                resolve({ code, stdout, stderr, duration, logs });
            }
        });

        child.on('error', (err) => {
            if (!resolved) {
                resolved = true;
                reject(err);
            }
        });

        // Timeout after 30 seconds
        setTimeout(() => {
            if (!resolved) {
                resolved = true;
                child.kill();
                reject(new Error('RPC call timeout'));
            }
        }, 30000);
    });
}

async function makeRpcCall(port, method, args) {
    const fetch = await import('node-fetch').then(m => m.default || m);
    
    // Map method names to RPC method names
    const methodMap = {
        'search': 'search',
        'getPopular': 'getPopular',
        'getLatest': 'getLatest',
        'getFiltered': 'getFiltered',
        'manga_info': 'manga_info',
        'get_chapter_images': 'get_chapter_images',
        'extension_info': 'extension_info'
    };
    
    const rpcMethod = methodMap[method] || method;
    const params = args.map(arg => {
        // Convert string numbers to actual numbers
        const num = parseFloat(arg);
        return isNaN(num) ? arg : num;
    });
    
    const response = await fetch(`http://127.0.0.1:${port}`, {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json',
        },
        body: JSON.stringify({
            jsonrpc: '2.0',
            method: rpcMethod,
            params: params,
            id: 1
        })
    });
    
    const result = await response.json();
    
    if (result.error) {
        throw new Error(`RPC error: ${result.error.message || JSON.stringify(result.error)}`);
    }
    
    log(`\n✅ Final Result:`, 'success');
    console.log(JSON.stringify(result.result, null, 2));
    
    return result.result;
}

async function showMethods() {
    log('\n📋 Available Methods:', 'info');
    methods.forEach((method, index) => {
        const descriptions = {
            'search': '🔍 Search for comics',
            'getPopular': '📈 Get popular comics',
            'getLatest': '🆕 Get latest updates',
            'getFiltered': '🔎 Get filtered results',
            'manga_info': '📖 Get manga details',
            'get_chapter_images': '🖼️ Get chapter images',
            'extension_info': 'ℹ️ Get extension info'
        };
        log(`  ${index + 1}. ${method} - ${descriptions[method] || ''}`, 'info');
    });
    log(`  ${methods.length + 1}. 🚀 Run all methods`, 'info');
    log(`  ${methods.length + 2}. 🧹 Clear logs`, 'info');
    log(`  ${methods.length + 3}. ❌ Exit`, 'info');
}

async function getMethodArgs(method) {
    switch(method) {
        case 'search':
            return [await question('🔍 Enter search query: ')];
        case 'manga_info':
            return [await question('📖 Enter manga identifier (slug): ')];
        case 'get_chapter_images':
            const bookId = await question('📚 Enter book ID: ');
            const chapter = await question('📄 Enter chapter number: ');
            const page = await question('📑 Enter page (default 1): ') || '1';
            const perPage = await question('📊 Enter per page (default 5): ') || '5';
            return [bookId, chapter, page, perPage];
        case 'getLatest':
            const pageNum = await question('📑 Enter page number (default 1): ') || '1';
            return [pageNum];
        case 'getPopular':
            const popPage = await question('📑 Enter page number (default 1): ') || '1';
            return [popPage];
        case 'getFiltered':
            log('📋 Filter options:', 'info');
            log('  Example: status=ongoing&type=manga&order=desc', 'debug');
            const filter = await question('🔎 Enter filter params: ');
            const filterPage = await question('📑 Enter page (default 1): ') || '1';
            return [filter, filterPage];
        case 'extension_info':
            return [];
        default:
            return [];
    }
}

async function runMethod(method) {
    log(`\n${'━'.repeat(50)}`, 'info');
    const args = await getMethodArgs(method);
    try {
        await executeMethod(method, args);
    } catch (error) {
        log(`❌ Error: ${error.message}`, 'error');
    }
    log(`${'━'.repeat(50)}\n`, 'info');
}

async function runAllMethods() {
    log(`\n🚀 Running all methods sequentially...`, 'info');
    for (const method of methods) {
        await runMethod(method);
    }
}

async function main() {
    log('\n╔═══════════════════════════════════════════════════╗', 'info');
    log('║     WZREAD Extension Tester v2.0 (RPC)           ║', 'info');
    log('╚═══════════════════════════════════════════════════╝\n', 'info');

    const extensions = await loadExtensions();
    
    if (extensions.length === 0) {
        log('❌ No extensions found in manifest', 'error');
        rl.close();
        return;
    }

    log(`📦 Found ${extensions.length} extension(s)`, 'success');
    
    const selected = await selectExtension(extensions);
    if (!selected) {
        rl.close();
        return;
    }

    let clearLogs = false;

    while (true) {
        if (clearLogs) {
            console.clear();
            clearLogs = false;
            log('\n🧹 Logs cleared!', 'success');
            log('\n╔═══════════════════════════════════════════════════╗', 'info');
            log('║     WZREAD Extension Tester v2.0 (RPC)           ║', 'info');
            log('╚═══════════════════════════════════════════════════╝\n', 'info');
            log(`📦 Selected: ${selectedExtension.name} (${selectedExtension.id})`, 'success');
        }

        await showMethods();
        
        const choice = await question('\n📌 Select option (number): ');
        const choiceNum = parseInt(choice);
        
        if (choiceNum === methods.length + 3) {
            log('\n👋 Goodbye!', 'info');
            rl.close();
            break;
        }
        
        if (choiceNum === methods.length + 2) {
            clearLogs = true;
            continue;
        }
        
        if (choiceNum === methods.length + 1) {
            await runAllMethods();
            continue;
        }
        
        if (choiceNum >= 1 && choiceNum <= methods.length) {
            const method = methods[choiceNum - 1];
            await runMethod(method);
        } else {
            log('❌ Invalid selection', 'error');
        }
    }
}

rl.on('SIGINT', () => {
    log('\n\n👋 Goodbye!', 'info');
    rl.close();
    process.exit(0);
});

main().catch((error) => {
    log(`\n❌ Error: ${error.message}`, 'error');
    console.error(error.stack);
    rl.close();
});