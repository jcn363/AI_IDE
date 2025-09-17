#!/usr/bin/env node

/**
 * Bundle Size Monitor
 * Monitors bundle size changes and alerts on significant increases
 */

const fs = require('fs');
const path = require('path');

const BUILD_DIR = path.join(__dirname, '../../dist');
const BASELINE_FILE = path.join(__dirname, '../performance-baselines/bundle-sizes.json');
const REPORT_DIR = path.join(__dirname, '../performance-reports');

class BundleSizeMonitor {
    constructor() {
        this.baseline = this.loadBaseline();
        this.ensureDirectories();
    }

    ensureDirectories() {
        if (!fs.existsSync(REPORT_DIR)) {
            fs.mkdirSync(REPORT_DIR, { recursive: true });
        }
        const baselineDir = path.dirname(BASELINE_FILE);
        if (!fs.existsSync(baselineDir)) {
            fs.mkdirSync(baselineDir, { recursive: true });
        }
    }

    loadBaseline() {
        try {
            if (fs.existsSync(BASELINE_FILE)) {
                return JSON.parse(fs.readFileSync(BASELINE_FILE, 'utf8'));
            }
        } catch (error) {
            console.warn('Could not load baseline bundle sizes:', error.message);
        }
        return {};
    }

    saveBaseline(data) {
        fs.writeFileSync(BASELINE_FILE, JSON.stringify(data, null, 2));
    }

    analyzeBundleSizes() {
        if (!fs.existsSync(BUILD_DIR)) {
            throw new Error(`Build directory not found: ${BUILD_DIR}. Run 'npm run build' first.`);
        }

        const sizes = {};
        const assetsDir = path.join(BUILD_DIR, 'assets');
        
        if (!fs.existsSync(assetsDir)) {
            throw new Error(`Assets directory not found: ${assetsDir}`);
        }
        
        const assets = fs.readdirSync(assetsDir);

        assets.forEach(asset => {
            const assetPath = path.join(assetsDir, asset);
            const stat = fs.statSync(assetPath);

            if (stat.isFile()) {
                const sizeKB = (stat.size / 1024).toFixed(2);
                sizes[asset] = {
                    size: parseFloat(sizeKB),
                    sizeBytes: stat.size,
                    timestamp: new Date().toISOString()
                };
            }
        });

        return sizes;
    }

    compareWithBaseline(currentSizes) {
        const changes = {};
        const warnings = [];
        const errors = [];

        Object.keys(currentSizes).forEach(asset => {
            const current = currentSizes[asset];
            const baseline = this.baseline[asset];

            if (baseline) {
                const sizeDiff = current.size - baseline.size;
                const percentChange = ((sizeDiff / baseline.size) * 100).toFixed(2);

                changes[asset] = {
                    current: current.size,
                    baseline: baseline.size,
                    difference: sizeDiff,
                    percentChange: parseFloat(percentChange),
                    timestamp: current.timestamp
                };

                // Alert thresholds
                if (Math.abs(percentChange) > 10) {
                    warnings.push(`${asset}: ${percentChange}% change (${sizeDiff > 0 ? '+' : ''}${sizeDiff.toFixed(2)} KB)`);
                }
                if (Math.abs(percentChange) > 25) {
                    errors.push(`${asset}: ${percentChange}% change (${sizeDiff > 0 ? '+' : ''}${sizeDiff.toFixed(2)} KB) - CRITICAL INCREASE`);
                }
            } else {
                changes[asset] = {
                    current: current.size,
                    baseline: null,
                    difference: null,
                    percentChange: null,
                    timestamp: current.timestamp,
                    status: 'new'
                };
                console.log(`New asset detected: ${asset} (${current.size} KB)`);
            }
        });

        return { changes, warnings, errors };
    }

    generateReport(currentSizes, comparison) {
        const timestamp = new Date().toISOString().replace(/[:.]/g, '-');
        const reportPath = path.join(REPORT_DIR, `bundle-size-report-${timestamp}.json`);

        const report = {
            timestamp: new Date().toISOString(),
            buildInfo: {
                nodeVersion: process.version,
                platform: process.platform,
                arch: process.arch
            },
            bundleSizes: currentSizes,
            comparison: comparison.changes,
            summary: {
                totalAssets: Object.keys(currentSizes).length,
                warnings: comparison.warnings.length,
                errors: comparison.errors.length,
                warningsList: comparison.warnings,
                errorsList: comparison.errors
            }
        };

        fs.writeFileSync(reportPath, JSON.stringify(report, null, 2));
        console.log(`Bundle size report saved to: ${reportPath}`);

        return report;
    }

    generateConsoleReport(comparison) {
        console.log('\n=== Bundle Size Analysis Report ===');
        console.log(`Timestamp: ${new Date().toISOString()}`);

        if (Object.keys(comparison.changes).length === 0) {
            console.log('No bundle size data to compare.');
            return;
        }

        console.log('\n--- Size Changes ---');
        Object.entries(comparison.changes).forEach(([asset, data]) => {
            if (data.status === 'new') {
                console.log(`📦 ${asset}: ${data.current} KB (NEW)`);
            } else {
                const changeSymbol = data.difference > 0 ? '🔺' : '🟢';
                const changeColor = data.difference > 0 ? '\x1b[31m' : '\x1b[32m';
                const resetColor = '\x1b[0m';
                console.log(`${changeSymbol} ${asset}: ${data.current} KB (${changeColor}${data.difference > 0 ? '+' : ''}${data.difference.toFixed(2)} KB, ${data.percentChange}%)${resetColor}`);
            }
        });

        if (comparison.warnings.length > 0) {
            console.log('\n⚠️  Warnings:');
            comparison.warnings.forEach(warning => console.log(`   ${warning}`));
        }

        if (comparison.errors.length > 0) {
            console.log('\n❌ Critical Issues:');
            comparison.errors.forEach(error => console.log(`   ${error}`));
        }

        console.log('\n=== End Report ===\n');
    }

    run() {
        try {
            console.log('Analyzing bundle sizes...');

            const currentSizes = this.analyzeBundleSizes();
            const comparison = this.compareWithBaseline(currentSizes);

            this.generateConsoleReport(comparison);
            const report = this.generateReport(currentSizes, comparison);

            // Update baseline if this is a successful build
            if (comparison.errors.length === 0) {
                this.saveBaseline(currentSizes);
                console.log('Baseline updated with current bundle sizes.');
            } else {
                console.warn('Baseline NOT updated due to critical size increases. Review changes before updating baseline.');
            }

            // Exit with error code if there are critical issues
            if (comparison.errors.length > 0) {
                console.error(`Bundle size monitoring failed with ${comparison.errors.length} critical issues.`);
                process.exit(1);
            }

            return report;

        } catch (error) {
            console.error('Bundle size monitoring failed:', error.message);
            process.exit(1);
        }
    }
}

// CLI interface
if (require.main === module) {
    const monitor = new BundleSizeMonitor();
    monitor.run();
}

module.exports = BundleSizeMonitor;
