#!/usr/bin/env node

/**
 * Performance Optimization Check Script
 * Analyzes build output for optimization opportunities
 */

const fs = require('fs');
const path = require('path');

const OPTIMIZATION_THRESHOLDS = {
  totalBundleSize: 5 * 1024 * 1024, // 5MB
  largestChunkSize: 1024 * 1024, // 1MB
  chunkCount: 20,
  vendorChunkRatio: 0.3 // 30% vendor code
};

function analyzeBuild() {
  const distPath = path.join(__dirname, '..', 'dist');

  if (!fs.existsSync(distPath)) {
    console.error('❌ Dist directory not found. Run build first.');
    return;
  }

  console.log('🔍 Analyzing build optimization...\n');

  // Check bundle sizes
  const assetsPath = path.join(distPath, 'assets');
  if (!fs.existsSync(assetsPath)) {
    console.error('❌ Assets directory not found');
    return;
  }

  const files = fs.readdirSync(assetsPath);
  const jsFiles = files.filter(f => f.endsWith('.js'));
  const cssFiles = files.filter(f => f.endsWith('.css'));

  let totalSize = 0;
  let largestChunk = 0;
  let vendorSize = 0;

  console.log('📦 JavaScript Chunks:');
  jsFiles.forEach(file => {
    const filePath = path.join(assetsPath, file);
    const stats = fs.statSync(filePath);
    const sizeKB = Math.round(stats.size / 1024);

    totalSize += stats.size;
    largestChunk = Math.max(largestChunk, stats.size);

    if (file.includes('vendor') || file.includes('react') || file.includes('mui')) {
      vendorSize += stats.size;
    }

    const status = stats.size > OPTIMIZATION_THRESHOLDS.largestChunkSize ? '⚠️' : '✅';
    console.log(`  ${status} ${file}: ${sizeKB}KB`);
  });

  console.log('\n📄 CSS Files:');
  cssFiles.forEach(file => {
    const filePath = path.join(assetsPath, file);
    const stats = fs.statSync(filePath);
    const sizeKB = Math.round(stats.size / 1024);
    console.log(`  ✅ ${file}: ${sizeKB}KB`);
  });

  // Performance analysis
  const totalSizeMB = Math.round(totalSize / (1024 * 1024) * 100) / 100;
  const vendorRatio = Math.round((vendorSize / totalSize) * 100);

  console.log('\n📊 Performance Analysis:');
  console.log(`  Total bundle size: ${totalSizeMB}MB ${totalSize > OPTIMIZATION_THRESHOLDS.totalBundleSize ? '⚠️' : '✅'}`);
  console.log(`  Largest chunk: ${Math.round(largestChunk / 1024)}KB ${largestChunk > OPTIMIZATION_THRESHOLDS.largestChunkSize ? '⚠️' : '✅'}`);
  console.log(`  Vendor code ratio: ${vendorRatio}% ${vendorRatio > OPTIMIZATION_THRESHOLDS.vendorChunkRatio * 100 ? '⚠️' : '✅'}`);
  console.log(`  Number of chunks: ${jsFiles.length} ${jsFiles.length > OPTIMIZATION_THRESHOLDS.chunkCount ? '⚠️' : '✅'}`);

  // Recommendations
  console.log('\n💡 Optimization Recommendations:');

  if (totalSize > OPTIMIZATION_THRESHOLDS.totalBundleSize) {
    console.log('  • Consider implementing more aggressive code splitting');
    console.log('  • Review and remove unused dependencies');
  }

  if (largestChunk > OPTIMIZATION_THRESHOLDS.largestChunkSize) {
    console.log('  • Split large chunks into smaller modules');
  }

  if (vendorRatio > OPTIMIZATION_THRESHOLDS.vendorChunkRatio * 100) {
    console.log('  • Consider lazy loading vendor libraries');
  }

  if (jsFiles.length > OPTIMIZATION_THRESHOLDS.chunkCount) {
    console.log('  • Too many chunks - consider consolidating small chunks');
  }

  console.log('\n✨ Optimization check complete!');
}

if (require.main === module) {
  analyzeBuild();
}

module.exports = { analyzeBuild, OPTIMIZATION_THRESHOLDS };