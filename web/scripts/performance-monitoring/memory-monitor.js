#!/usr/bin/env node

/**
 * Memory Usage Monitor
 * Monitors memory usage patterns and detects potential memory leaks
 */

const puppeteer = require('puppeteer');
const fs = require('fs');
const path = require('path');

const BASELINE_FILE = path.join(__dirname, '../performance-baselines/memory-usage.json');
const REPORT_DIR = path.join(__dirname, '../performance-reports');

class MemoryMonitor {
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
            console.warn('Could not load baseline memory usage:', error.message);
        }
        return {};
    }

    saveBaseline(data) {
        fs.writeFileSync(BASELINE_FILE, JSON.stringify(data, null, 2));
    }

    async monitorMemoryUsage(url, options = {}) {
        const {
            duration = 60000, // 60 seconds
            interval = 5000,   // 5 seconds
            actions = [],
            headless = true,
            viewport = { width: 1280, height: 720 }
        } = options;

        console.log(`Starting memory monitoring for ${duration / 1000} seconds...`);

        const browser = await puppeteer.launch({
            headless,
            args: [
                '--no-sandbox',
                '--disable-setuid-sandbox',
                '--disable-dev-shm-usage',
                '--max_old_space_size=4096'
            ]
        });

        try {
            const page = await browser.newPage();
            await page.setViewport(viewport);

            // Enable performance monitoring
            await page.evaluateOnNewDocument(() => {
                // Track heap usage over time
                window.memorySnapshots = [];
                window.initialMemory = performance.memory ? {
                    usedJSHeapSize: performance.memory.usedJSHeapSize,
                    totalJSHeapSize: performance.memory.totalJSHeapSize,
                    jsHeapSizeLimit: performance.memory.jsHeapSizeLimit
                } : null;

                // Monitor for leaks by tracking object counts
                if (window.gc) {
                    window.gc();
                }
            });

            // Navigate to the page
            await page.goto(url, {
                waitUntil: 'networkidle0',
                timeout: 60000
            });

            console.log('Page loaded, starting memory monitoring...');

            const memoryReadings = [];
            const startTime = Date.now();

            // Perform user actions if specified
            if (actions && actions.length > 0) {
                console.log('Performing user actions...');
                for (const action of actions) {
                    try {
                        if (action.type === 'click' && action.selector) {
                            await page.waitForSelector(action.selector, { timeout: 5000 });
                            await page.click(action.selector);
                        } else if (action.type === 'type' && action.selector && action.text) {
                            await page.waitForSelector(action.selector, { timeout: 5000 });
                            await page.type(action.selector, action.text);
                        } else if (action.type === 'wait') {
                            await page.waitForTimeout(action.duration || 1000);
                        } else if (action.type === 'scroll') {
                            await page.evaluate(() => window.scrollTo(0, document.body.scrollHeight));
                        }
                        
                        if (action.waitAfter) {
                            await page.waitForTimeout(action.waitAfter);
                        }
                    } catch (error) {
                        console.warn(`Failed to perform action ${action.type}:`, error.message);
                    }
                }
            }

            // Monitor memory at regular intervals
            while (Date.now() - startTime < duration) {
                const timestamp = Date.now();

                const memoryData = await page.evaluate(() => {
                    const perf = window.performance;
                    const memory = perf.memory || {};

                    // Force garbage collection if available
                    if (window.gc) {
                        window.gc();
                    }

                    return {
                        timestamp: Date.now(),
                        usedJSHeapSize: memory.usedJSHeapSize || 0,
                        totalJSHeapSize: memory.totalJSHeapSize || 0,
                        jsHeapSizeLimit: memory.jsHeapSizeLimit || 0,
                        // Additional metrics if available
                        domNodes: document.getElementsByTagName('*').length,
                        jsEventListeners: window._eventListenersCount || 0
                    };
                });

                memoryReadings.push({
                    elapsed: timestamp - startTime,
                    ...memoryData
                });

                console.log(`Memory check: ${memoryData.usedJSHeapSize / 1024 / 1024} MB used`);

                await page.waitForTimeout(interval);
            }

            // Final memory snapshot
            const finalMemory = await page.evaluate(() => {
                if (window.gc) {
                    window.gc();
                }
                const memory = window.performance.memory || {};
                return {
                    usedJSHeapSize: memory.usedJSHeapSize || 0,
                    totalJSHeapSize: memory.totalJSHeapSize || 0,
                    jsHeapSizeLimit: memory.jsHeapSizeLimit || 0
                };
            });

            return {
                startTime,
                endTime: Date.now(),
                duration,
                memoryReadings,
                finalMemory,
                actionsPerformed: actions.length,
                initialMemory: await page.evaluate(() => window.initialMemory)
            };

        } finally {
            await browser.close();
        }
    }

    analyzeMemoryPatterns(memoryReadings) {
        if (memoryReadings.length < 2) {
            return { trend: 'insufficient-data', analysis: {} };
        }

        const readings = memoryReadings.map(r => r.usedJSHeapSize);
        const firstReading = readings[0];
        const lastReading = readings[readings.length - 1];
        const maxReading = Math.max(...readings);
        const minReading = Math.min(...readings);

        // Calculate memory growth trend
        const totalGrowth = lastReading - firstReading;
        const growthRate = totalGrowth / (readings.length - 1); // bytes per interval

        // Check for memory leaks (sustained growth)
        const growthThreshold = 1024 * 1024 * 10; // 10MB sustained growth
        const leakThreshold = 1024 * 1024 * 50;   // 50MB total growth

        let trend = 'stable';
        let leakDetected = false;

        if (totalGrowth > leakThreshold) {
            trend = 'significant-growth';
            leakDetected = true;
        } else if (growthRate > 1024 * 100) { // 100KB per interval growth rate
            trend = 'gradual-growth';
            if (totalGrowth > growthThreshold) {
                leakDetected = true;
            }
        } else if (growthRate < -1024 * 50) { // Significant decrease
            trend = 'decreasing';
        }

        // Calculate memory statistics
        const average = readings.reduce((sum, val) => sum + val, 0) / readings.length;
        const variance = readings.reduce((sum, val) => sum + Math.pow(val - average, 2), 0) / readings.length;
        const stdDev = Math.sqrt(variance);

        return {
            trend,
            leakDetected,
            analysis: {
                initialMemory: firstReading,
                finalMemory: lastReading,
                maxMemory: maxReading,
                minMemory: minReading,
                averageMemory: average,
                totalGrowth,
                growthRate,
                standardDeviation: stdDev,
                memoryRange: maxReading - minReading,
                readingsCount: readings.length
            }
        };
    }

    compareWithBaseline(currentAnalysis) {
        const changes = {};
        const warnings = [];
        const errors = [];

        if (!this.baseline.averageMemory) {
            console.log('No baseline memory usage available. This run will establish the baseline.');
            return { changes: {}, warnings: [], errors: [] };
        }

        // Compare memory statistics
        const metrics = ['averageMemory', 'maxMemory', 'totalGrowth'];
        metrics.forEach(metric => {
            const current = currentAnalysis.analysis[metric];
            const baseline = this.baseline[metric];

            if (baseline) {
                const diff = current - baseline;
                const percentChange = ((diff / baseline) * 100);

                changes[metric] = {
                    current,
                    baseline,
                    difference: diff,
                    percentChange: percentChange
                };

                // Alert thresholds
                if (Math.abs(percentChange) > 25) {
                    const direction = diff > 0 ? 'increased' : 'decreased';
                    warnings.push(`${metric} ${direction} by ${Math.abs(percentChange).toFixed(2)}% (${(diff / 1024 / 1024).toFixed(2)} MB)`);
                }
                if (Math.abs(percentChange) > 50) {
                    errors.push(`${metric} CRITICALLY ${diff > 0 ? 'increased' : 'decreased'} by ${Math.abs(percentChange).toFixed(2)}% (${(diff / 1024 / 1024).toFixed(2)} MB)`);
                }
            }
        });

        // Check for new memory leaks
        if (currentAnalysis.leakDetected && !this.baseline.leakDetected) {
            errors.push('Memory leak detected in current run (not present in baseline)');
        }

        return { changes, warnings, errors };
    }

    generateReport(results, analysis, comparison) {
        const timestamp = new Date().toISOString().replace(/[:.]/g, '-');
        const reportPath = path.join(REPORT_DIR, `memory-report-${timestamp}.json`);

        const report = {
            timestamp: new Date().toISOString(),
            buildInfo: {
                nodeVersion: process.version,
                platform: process.platform,
                arch: process.arch
            },
            monitoringResults: results,
            memoryAnalysis: analysis,
            comparison: comparison.changes,
            summary: {
                trend: analysis.trend,
                leakDetected: analysis.leakDetected,
                warnings: comparison.warnings.length,
                errors: comparison.errors.length,
                warningsList: comparison.warnings,
                errorsList: comparison.errors
            }
        };

        fs.writeFileSync(reportPath, JSON.stringify(report, null, 2));
        console.log(`Memory report saved to: ${reportPath}`);

        return report;
    }

    generateConsoleReport(analysis, comparison) {
        console.log('\n=== Memory Usage Analysis Report ===');
        console.log(`Timestamp: ${new Date().toISOString()}`);
        console.log(`Monitoring Trend: ${analysis.trend}`);
        console.log(`Memory Leak Detected: ${analysis.leakDetected ? 'YES' : 'NO'}`);

        console.log('\n--- Memory Statistics ---');
        const stats = analysis.analysis;
        console.log(`Initial Memory: ${(stats.initialMemory / 1024 / 1024).toFixed(2)} MB`);
        console.log(`Final Memory: ${(stats.finalMemory / 1024 / 1024).toFixed(2)} MB`);
        console.log(`Peak Memory: ${(stats.maxMemory / 1024 / 1024).toFixed(2)} MB`);
        console.log(`Average Memory: ${(stats.averageMemory / 1024 / 1024).toFixed(2)} MB`);
        console.log(`Total Growth: ${(stats.totalGrowth / 1024 / 1024).toFixed(2)} MB`);
        console.log(`Growth Rate: ${(stats.growthRate / 1024).toFixed(2)} KB per interval`);
        console.log(`Memory Range: ${(stats.memoryRange / 1024 / 1024).toFixed(2)} MB`);

        if (Object.keys(comparison.changes).length > 0) {
            console.log('\n--- Memory Changes vs Baseline ---');
            Object.entries(comparison.changes).forEach(([metric, data]) => {
                const changeSymbol = data.difference > 0 ? '🔺' : '🟢';
                const changeColor = data.difference > 0 ? '\x1b[31m' : '\x1b[32m';
                const resetColor = '\x1b[0m';
                console.log(`${changeSymbol} ${metric}: ${(data.current / 1024 / 1024).toFixed(2)} MB vs ${(data.baseline / 1024 / 1024).toFixed(2)} MB (${changeColor}${data.difference > 0 ? '+' : ''}${(data.difference / 1024 / 1024).toFixed(2)} MB, ${data.percentChange.toFixed(2)}%)${resetColor}`);
            });
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
                duration = 60000,
                interval = 5000,
                actions = [],
                updateBaseline = true
            } = options;

            if (!url) {
                throw new Error('URL is required. Use --url parameter or set MEMORY_MONITOR_URL environment variable.');
            }

            console.log('Starting memory monitoring...');
            console.log(`URL: ${url}`);
            console.log(`Duration: ${duration / 1000} seconds`);
            console.log(`Interval: ${interval / 1000} seconds`);

            const results = await this.monitorMemoryUsage(url, { duration, interval, actions });
            const analysis = this.analyzeMemoryPatterns(results.memoryReadings);
            const comparison = this.compareWithBaseline(analysis);

            this.generateConsoleReport(analysis, comparison);
            const report = this.generateReport(results, analysis, comparison);

            // Update baseline if requested and no critical issues
            if (updateBaseline && comparison.errors.length === 0) {
                const baselineData = {
                    timestamp: results.endTime,
                    averageMemory: analysis.analysis.averageMemory,
                    maxMemory: analysis.analysis.maxMemory,
                    totalGrowth: analysis.analysis.totalGrowth,
                    leakDetected: analysis.leakDetected,
                    trend: analysis.trend
                };

                this.saveBaseline(baselineData);
                console.log('Baseline updated with current memory usage.');
            } else if (comparison.errors.length > 0) {
                console.warn('Baseline NOT updated due to critical memory issues.');
            }

            // Exit with error code if there are critical issues
            if (comparison.errors.length > 0 || analysis.leakDetected) {
                console.error(`Memory monitoring failed with critical issues.`);
                process.exit(1);
            }

            return report;

        } catch (error) {
            console.error('Memory monitoring failed:', error.message);
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
            case '--duration':
                options.duration = parseInt(args[++i]) * 1000; // Convert to milliseconds
                break;
            case '--interval':
                options.interval = parseInt(args[++i]) * 1000; // Convert to milliseconds
                break;
            case '--no-baseline-update':
                options.updateBaseline = false;
                break;
            case '--help':
                console.log('Usage: node memory-monitor.js [options]');
                console.log('');
                console.log('Options:');
                console.log('  --url <url>              URL to test (required)');
                console.log('  --duration <seconds>     Monitoring duration in seconds (default: 60)');
                console.log('  --interval <seconds>     Monitoring interval in seconds (default: 5)');
                console.log('  --no-baseline-update     Don\'t update baseline with results');
                console.log('  --help                   Show this help');
                process.exit(0);
        }
    }

    // Check for environment variable if no URL provided
    if (!options.url) {
        options.url = process.env.MEMORY_MONITOR_URL;
    }

    const monitor = new MemoryMonitor();
    monitor.run(options);
}

module.exports = MemoryMonitor;
