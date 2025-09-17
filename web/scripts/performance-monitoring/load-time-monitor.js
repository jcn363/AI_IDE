#!/usr/bin/env node

/**
 * Load Time Monitor
 * Measures page load times and performance metrics using Puppeteer
 */

const puppeteer = require('puppeteer');
const fs = require('fs');
const path = require('path');

const BASELINE_FILE = path.join(__dirname, '../performance-baselines/load-times.json');
const REPORT_DIR = path.join(__dirname, '../performance-reports');
const DIST_DIR = path.join(__dirname, '../../dist');

class LoadTimeMonitor {
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
            console.warn('Could not load baseline load times:', error.message);
        }
        return {};
    }

    saveBaseline(data) {
        fs.writeFileSync(BASELINE_FILE, JSON.stringify(data, null, 2));
    }

    async measureLoadTimes(url, options = {}) {
        const {
            runs = 3,
            headless = true,
            viewport = { width: 1280, height: 720 }
        } = options;

        const results = [];
        
        for (let i = 0; i < runs; i++) {
            console.log(`Running load time test ${i + 1}/${runs}...`);
            
            const browser = await puppeteer.launch({ 
                headless,
                args: ['--no-sandbox', '--disable-setuid-sandbox']
            });
            
            try {
                const page = await browser.newPage();
                await page.setViewport(viewport);
                
                // Enable performance monitoring
                await page.evaluateOnNewDocument(() => {
                    window.performance.mark('start');
                });
                
                const startTime = Date.now();
                
                // Navigate and wait for load
                const response = await page.goto(url, { 
                    waitUntil: 'networkidle0',
                    timeout: 60000 
                });
                
                const loadTime = Date.now() - startTime;
                
                // Get performance metrics
                const performanceMetrics = await page.evaluate(() => {
                    const perf = window.performance;
                    
                    // Navigation timing
                    const navigation = perf.getEntriesByType('navigation')[0];
                    
                    // Resource timing
                    const resources = perf.getEntriesByType('resource');
                    
                    // Paint timing
                    const paints = perf.getEntriesByType('paint');
                    
                    return {
                        domContentLoaded: navigation.domContentLoadedEventEnd - navigation.domContentLoadedEventStart,
                        loadComplete: navigation.loadEventEnd - navigation.loadEventStart,
                        firstPaint: paints.find(p => p.name === 'first-paint')?.startTime || 0,
                        firstContentfulPaint: paints.find(p => p.name === 'first-contentful-paint')?.startTime || 0,
                        resources: resources.length,
                        totalResourceSize: resources.reduce((sum, r) => sum + (r.transferSize || 0), 0)
                    };
                });
                
                const result = {
                    run: i + 1,
                    timestamp: new Date().toISOString(),
                    url,
                    totalLoadTime: loadTime,
                    statusCode: response.status(),
                    ...performanceMetrics
                };
                
                results.push(result);
                console.log(`  Load time: ${loadTime}ms, Resources: ${performanceMetrics.resources}`);
                
            } finally {
                await browser.close();
            }
            
            // Small delay between runs
            if (i < runs - 1) {
                await new Promise(resolve => setTimeout(resolve, 1000));
            }
        }
        
        return this.aggregateResults(results);
    }

    aggregateResults(results) {
        if (results.length === 0) return null;
        
        const metrics = ['totalLoadTime', 'domContentLoaded', 'loadComplete', 'firstPaint', 'firstContentfulPaint', 'resources', 'totalResourceSize'];
        const aggregated = {
            runs: results.length,
            timestamp: new Date().toISOString(),
            url: results[0].url,
            averages: {},
            medians: {},
            min: {},
            max: {},
            individualRuns: results
        };
        
        metrics.forEach(metric => {
            const values = results.map(r => r[metric]).sort((a, b) => a - b);
            aggregated.averages[metric] = values.reduce((sum, val) => sum + val, 0) / values.length;
            aggregated.medians[metric] = values.length % 2 === 0 
                ? (values[values.length / 2 - 1] + values[values.length / 2]) / 2
                : values[Math.floor(values.length / 2)];
            aggregated.min[metric] = Math.min(...values);
            aggregated.max[metric] = Math.max(...values);
        });
        
        return aggregated;
    }

    compareWithBaseline(currentResults) {
        const changes = {};
        const warnings = [];
        const errors = [];
        
        if (!this.baseline.averageLoadTime) {
            console.log('No baseline load times available. This run will establish the baseline.');
            return { changes: {}, warnings: [], errors: [] };
        }
        
        const currentAvg = currentResults.averages.totalLoadTime;
        const baselineAvg = this.baseline.averageLoadTime;
        const diff = currentAvg - baselineAvg;
        const percentChange = ((diff / baselineAvg) * 100);
        
        changes.loadTime = {
            current: currentAvg,
            baseline: baselineAvg,
            difference: diff,
            percentChange: percentChange,
            timestamp: currentResults.timestamp
        };
        
        // Alert thresholds for load time degradation
        if (percentChange > 15) {
            warnings.push(`Load time increased by ${percentChange.toFixed(2)}% (${diff.toFixed(2)}ms)`);
        }
        if (percentChange > 30) {
            errors.push(`Load time CRITICALLY increased by ${percentChange.toFixed(2)}% (${diff.toFixed(2)}ms)`);
        }
        
        // Check other metrics
        ['domContentLoaded', 'firstContentfulPaint'].forEach(metric => {
            const current = currentResults.averages[metric];
            const baseline = this.baseline[metric];
            
            if (baseline && current) {
                const metricDiff = current - baseline;
                const metricPercentChange = ((metricDiff / baseline) * 100);
                
                if (Math.abs(metricPercentChange) > 20) {
                    const direction = metricDiff > 0 ? 'increased' : 'decreased';
                    warnings.push(`${metric} ${direction} by ${Math.abs(metricPercentChange).toFixed(2)}% (${Math.abs(metricDiff).toFixed(2)}ms)`);
                }
            }
        });
        
        return { changes, warnings, errors };
    }

    generateReport(results, comparison) {
        const timestamp = new Date().toISOString().replace(/[:.]/g, '-');
        const reportPath = path.join(REPORT_DIR, `load-time-report-${timestamp}.json`);
        
        const report = {
            timestamp: new Date().toISOString(),
            buildInfo: {
                nodeVersion: process.version,
                platform: process.platform,
                arch: process.arch
            },
            loadTimeResults: results,
            comparison: comparison.changes,
            summary: {
                warnings: comparison.warnings.length,
                errors: comparison.errors.length,
                warningsList: comparison.warnings,
                errorsList: comparison.errors
            }
        };
        
        fs.writeFileSync(reportPath, JSON.stringify(report, null, 2));
        console.log(`Load time report saved to: ${reportPath}`);
        
        return report;
    }

    generateConsoleReport(results, comparison) {
        console.log('\n=== Load Time Analysis Report ===');
        console.log(`Timestamp: ${new Date().toISOString()}`);
        console.log(`URL: ${results.url}`);
        console.log(`Runs: ${results.runs}`);
        
        console.log('\n--- Average Metrics ---');
        console.log(`Total Load Time: ${results.averages.totalLoadTime.toFixed(2)}ms`);
        console.log(`DOM Content Loaded: ${results.averages.domContentLoaded.toFixed(2)}ms`);
        console.log(`Load Complete: ${results.averages.loadComplete.toFixed(2)}ms`);
        console.log(`First Paint: ${results.averages.firstPaint.toFixed(2)}ms`);
        console.log(`First Contentful Paint: ${results.averages.firstContentfulPaint.toFixed(2)}ms`);
        console.log(`Resources: ${results.averages.resources.toFixed(0)}`);
        console.log(`Total Resource Size: ${(results.averages.totalResourceSize / 1024).toFixed(2)} KB`);
        
        if (comparison.changes.loadTime) {
            const change = comparison.changes.loadTime;
            console.log('\n--- Load Time Changes ---');
            const changeSymbol = change.difference > 0 ? '🔺' : '🟢';
            const changeColor = change.difference > 0 ? '\x1b[31m' : '\x1b[32m';
            const resetColor = '\x1b[0m';
            console.log(`${changeSymbol} Load Time: ${change.current.toFixed(2)}ms vs ${change.baseline.toFixed(2)}ms (${changeColor}${change.difference > 0 ? '+' : ''}${change.difference.toFixed(2)}ms, ${change.percentChange.toFixed(2)}%)${resetColor}`);
        }
        
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

    async run(options = {}) {
        try {
            const {
                url,
                runs = 3,
                updateBaseline = true
            } = options;
            
            if (!url) {
                throw new Error('URL is required. Use --url parameter or set LOAD_TIME_URL environment variable.');
            }
            
            console.log('Measuring load times...');
            console.log(`URL: ${url}`);
            console.log(`Runs: ${runs}`);
            
            const results = await this.measureLoadTimes(url, { runs });
            const comparison = this.compareWithBaseline(results);
            
            this.generateConsoleReport(results, comparison);
            const report = this.generateReport(results, comparison);
            
            // Update baseline if requested and no critical issues
            if (updateBaseline && comparison.errors.length === 0) {
                const baselineData = {
                    timestamp: results.timestamp,
                    url: results.url,
                    averageLoadTime: results.averages.totalLoadTime,
                    domContentLoaded: results.averages.domContentLoaded,
                    loadComplete: results.averages.loadComplete,
                    firstPaint: results.averages.firstPaint,
                    firstContentfulPaint: results.averages.firstContentfulPaint,
                    resources: results.averages.resources,
                    totalResourceSize: results.averages.totalResourceSize
                };
                
                this.saveBaseline(baselineData);
                console.log('Baseline updated with current load times.');
            } else if (comparison.errors.length > 0) {
                console.warn('Baseline NOT updated due to critical load time increases.');
            }
            
            // Exit with error code if there are critical issues
            if (comparison.errors.length > 0) {
                console.error(`Load time monitoring failed with ${comparison.errors.length} critical issues.`);
                process.exit(1);
            }
            
            return report;
            
        } catch (error) {
            console.error('Load time monitoring failed:', error.message);
            process.exit(1);
        }
    }
}

// CLI interface
if (require.main === module) {
    const args = process.argv.slice(2);
    const options = {};
    
    // Parse command line arguments
    for (let i = 0; i < args.length; i++) {
        switch (args[i]) {
            case '--url':
                options.url = args[++i];
                break;
            case '--runs':
                options.runs = parseInt(args[++i]);
                break;
            case '--no-baseline-update':
                options.updateBaseline = false;
                break;
            case '--help':
                console.log('Usage: node load-time-monitor.js [options]');
                console.log('');
                console.log('Options:');
                console.log('  --url <url>              URL to test (required)');
                console.log('  --runs <number>          Number of test runs (default: 3)');
                console.log('  --no-baseline-update     Don\'t update baseline with results');
                console.log('  --help                   Show this help');
                process.exit(0);
        }
    }
    
    // Check for environment variable if no URL provided
    if (!options.url) {
        options.url = process.env.LOAD_TIME_URL;
    }
    
    const monitor = new LoadTimeMonitor();
    monitor.run(options);
}

module.exports = LoadTimeMonitor;
