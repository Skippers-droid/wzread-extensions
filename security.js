/* 
Basic security/forbidden system using acorn-walk, might be able to catch malcicous plugin or so, this isnt a fully gurantee result, so dont use random extensions other than trusted
*/
const fs = require('fs');
const path = require('path');
const acorn = require('acorn');
const walk = require('acorn-walk');

const FORBIDDEN_MODULES = [
  'child_process', 'fs', 'path', 'os', 'net', 'http', 'https', 
  'dns', 'vm', 'worker_threads', 'crypto', 'tls', 'cluster',
  'readline', 'repl', 'stream', 'tty', 'zlib', 'perf_hooks',
  'electron', 'better-sqlite3'
];

const FORBIDDEN_BUILTINS = [
  'eval', 'Function', 'setImmediate', 'queueMicrotask'
];

const FORBIDDEN_GLOBALS = [
  'process', 'global', 'globalThis', '__dirname', '__filename',
  'Buffer', 'module', 'exports'
];

function validateExtensionCode(code) {
  const violations = [];
  
  try {
    const ast = acorn.parse(code, {
      ecmaVersion: 2022,
      sourceType: 'module',
      allowAwaitOutsideFunction: true,
      allowReturnOutsideFunction: true
    });

    walk.simple(ast, {
      ImportDeclaration(node) {
        const source = node.source.value;
        if (!source.startsWith('.')) {
          if (FORBIDDEN_MODULES.includes(source)) {
            violations.push(`Import from '${source}' is forbidden`);
          }
        }
      },
      
      CallExpression(node) {
        if (node.callee.type === 'Identifier' && node.callee.name === 'require') {
          if (node.arguments.length > 0 && node.arguments[0].type === 'Literal') {
            const moduleName = node.arguments[0].value;
            if (FORBIDDEN_MODULES.includes(moduleName)) {
              violations.push(`require('${moduleName}') is forbidden`);
            }
          }
        }
        
        if (node.callee.type === 'Identifier') {
          if (node.callee.name === 'eval') {
            violations.push('eval() is forbidden');
          }
          if (node.callee.name === 'Function' && node.type === 'NewExpression') {
            violations.push('new Function() is forbidden');
          }
          if (node.callee.name === 'setTimeout' || node.callee.name === 'setInterval') {
            if (node.arguments.length > 0 && node.arguments[0].type === 'Literal') {
              violations.push(`${node.callee.name}() with string code is forbidden`);
            }
          }
          if (node.callee.name === 'setImmediate') {
            violations.push('setImmediate() is forbidden');
          }
          if (node.callee.name === 'queueMicrotask') {
            violations.push('queueMicrotask() is forbidden');
          }
        }
        
        if (node.callee.type === 'MemberExpression') {
          if (node.callee.object.type === 'Identifier' && node.callee.object.name === 'Buffer') {
            violations.push('Buffer() is forbidden');
          }
        }
      },
      
      MemberExpression(node) {
        if (node.object.type === 'Identifier') {
          if (FORBIDDEN_GLOBALS.includes(node.object.name)) {
            violations.push(`'${node.object.name}' access is forbidden`);
          }
          if (FORBIDDEN_MODULES.includes(node.object.name)) {
            violations.push(`'${node.object.name}' module access is forbidden`);
          }
        }
        
        if (node.property.type === 'Identifier') {
          if (node.property.name === '__proto__') {
            violations.push('__proto__ manipulation is forbidden');
          }
          if (node.property.name === 'constructor' && node.object.type === 'MemberExpression') {
            violations.push('constructor prototype manipulation is forbidden');
          }
        }
        
        if (node.object.type === 'MemberExpression') {
          if (node.object.property && node.object.property.name === 'prototype') {
            if (node.property.name === 'constructor') {
              violations.push('prototype constructor manipulation is forbidden');
            }
          }
        }
      },
      
      Identifier(node) {
        if (FORBIDDEN_GLOBALS.includes(node.name) && (!node.parent || node.parent.type !== 'MemberExpression')) {
          let isTopLevel = true;
          let parent = node.parent;
          while (parent) {
            if (parent.type === 'FunctionDeclaration' || parent.type === 'FunctionExpression' || 
                parent.type === 'ArrowFunctionExpression' || parent.type === 'BlockStatement') {
              isTopLevel = false;
              break;
            }
            parent = parent.parent;
          }
          if (isTopLevel) {
            violations.push(`'${node.name}' global access is forbidden`);
          }
        }
      },
      
      NewExpression(node) {
        if (node.callee.type === 'Identifier' && node.callee.name === 'Function') {
          violations.push('new Function() is forbidden');
        }
        if (node.callee.type === 'Identifier' && node.callee.name === 'Buffer') {
          violations.push('new Buffer() is forbidden');
        }
      },
      
      BinaryExpression(node) {
        if (node.operator === 'in') {
          if (node.right.type === 'Identifier' && node.right.name === 'global') {
            violations.push('Accessing global via "in" operator is forbidden');
          }
        }
      }
    });
    
  } catch (error) {
    violations.push(`Failed to parse code: ${error.message}`);
  }
  
  return violations;
}

function scanDirectoryForMaliciousFiles(dir, scriptType) {
  const suspiciousFiles = [];
  const extensions = scriptType === 'all' ? ['.js', '.mjs', '.cjs', '.ts', '.tsx'] : 
                     scriptType === 'ts' ? ['.ts', '.tsx'] : 
                     ['.js', '.mjs', '.cjs'];
  
  function scanFiles(currentDir) {
    const items = fs.readdirSync(currentDir);
    
    for (const item of items) {
      const fullPath = path.join(currentDir, item);
      const stat = fs.statSync(fullPath);
      
      if (stat.isDirectory()) {
        if (item === 'node_modules' || item.startsWith('.')) {
          continue;
        }
        scanFiles(fullPath);
      } else if (stat.isFile()) {
        const ext = path.extname(item);
        if (extensions.includes(ext)) {
          try {
            const content = fs.readFileSync(fullPath, 'utf-8');
            const violations = validateExtensionCode(content);
            if (violations.length > 0) {
              suspiciousFiles.push({
                file: fullPath,
                violations: violations
              });
            }
          } catch (error) {
            // Could not read file, skip
          }
        }
      }
    }
  }
  
  scanFiles(dir);
  return suspiciousFiles;
}

module.exports = {
  validateExtensionCode,
  scanDirectoryForMaliciousFiles
};