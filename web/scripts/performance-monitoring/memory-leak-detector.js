#!/usr/bin/env node

/**
 * Memory Leak Detection Script
 * Advanced memory leak detection using heap analysis
 */

const puppeteer = require('puppeteer');
const fs = require('fs');
const path = require('path');

class MemoryLeakDetector {
    async detectLeaks(url, options = {}) {
        const {
            duration = 300000, // 5 minutes
            interval = 30000,   // 30 seconds
            actions = [],
            headless = true
        } = options;

        console.log(`Starting memory leak detection for ${duration / 1000 / 60} minutes...`);

        const browser = await puppeteer.launch({
            headless,
            args: [
                '--no-sandbox',
                '--disable-setuid-sandbox',
                '--disable-dev-shm-usage',
                '--max_old_space_size=4096',
                '--js-flags=--expose-gc'
            ]
        });

        try {
            const page = await browser.newPage();
            await page.setViewport({ width: 1280, height: 720 });

            // Navigate to page
            await page.goto(url, {
                waitUntil: 'networkidle0',
                timeout: 60000
            });

            console.log('Page loaded, starting leak detection...');

            const leakData = {
                timeline: [],
                heapSnapshots: [],
                eventListeners: [],
                domNodes: []
            };

            const startTime = Date.now();

            // Perform initial actions
            if (actions.length > 0) {
                console.log('Performing initial user actions...');
                await this.performActions(page, actions);
            }

            // Monitor memory over time
            let iteration = 0;
            while (Date.now() - startTime < duration) {
                const timestamp = Date.now();
                const elapsed = timestamp - startTime;

                console.log(`Leak detection iteration ${iteration + 1} (${Math.round(elapsed / 1000)}s elapsed)...`);

                // Collect memory metrics
                const memoryMetrics = await this.collectMemoryMetrics(page);

                leakData.timeline.push({
                    iteration,
                    timestamp,
                    elapsed,
                    ...memoryMetrics
                });

                // Perform actions to stress memory
                if (actions.length > 0 && iteration % 3 === 0) { // Every 3 iterations
                    console.log('  Performing stress actions...');
                    await this.performActions(page, actions);
                }

                // Force garbage collection
                await page.evaluate(() => {
                    if (window.gc) {
                        window.gc();
                    }
                });

                iteration++;
                await page.waitForTimeout(interval);
            }

            // Analyze collected data for leaks
            const analysis = this.analyzeLeakData(leakData);

            return {
                success: true,
                duration,
                iterations: iteration,
                analysis,
                leakDetected: analysis.leakDetected,
                leakData
            };

        } finally {
            await browser.close();
        }
    }

    async collectMemoryMetrics(page) {
        return await page.evaluate(() => {
            const perf = window.performance;
            const memory = perf.memory || {};

            // Count DOM nodes
            const domNodes = document.getElementsByTagName('*').length;

            // Count event listeners (simplified)
            const eventListeners = window._eventListenersCount || 0;

            return {
                usedJSHeapSize: memory.usedJSHeapSize || 0,
                totalJSHeapSize: memory.totalJSHeapSize || 0,
                jsHeapSizeLimit: memory.jsHeapSizeLimit || 0,
                domNodes,
                eventListeners,
                timestamp: Date.now()
            };
        });
    }

    async performActions(page, actions) {
        for (const action of actions) {
            try {
                switch (action.type) {
                    case 'click':
                        if (action.selector) {
                            await page.waitForSelector(action.selector, { timeout: 5000 });
                            await page.click(action.selector);
                        }
                        break;
                    case 'type':
                        if (action.selector && action.text) {
                            await page.waitForSelector(action.selector, { timeout: 5000 });
                            await page.clear(action.selector);
                            await page.type(action.selector, action.text);
                        }
                        break;
                    case 'scroll':
                        await page.evaluate(() => {
                            window.scrollTo(0, document.body.scrollHeight);
                        });
                        break;
                    case 'navigate':
                        if (action.url) {
                            await page.goto(action.url, { waitUntil: 'networkidle0' });
                        }
                        break;
                    case 'wait':
                        await page.waitForTimeout(action.duration || 1000);
                        break;
                }

                if (action.waitAfter) {
                    await page.waitForTimeout(action.waitAfter);
                }
            } catch (error) {
                console.warn(`Failed to perform action ${action.type}:`, error.message);
            }
        }
    }

    analyzeLeakData(leakData) {
        const timeline = leakData.timeline;

        if (timeline.length < 3) {
            return { leakDetected: false, confidence: 'insufficient-data' };
        }

        // Analyze memory growth trends
        const memoryUsage = timeline.map(t => t.usedJSHeapSize);
        const domNodes = timeline.map(t => t.domNodes);
        const eventListeners = timeline.map(t => t.eventListeners);

        // Calculate growth rates
        const memoryGrowthRate = this.calculateGrowthRate(memoryUsage);
        const domGrowthRate = this.calculateGrowthRate(domNodes);
        const eventListenerGrowthRate = this.calculateGrowthRate(eventListeners);

        // Memory leak detection logic
        let leakDetected = false;
        let leakType = 'none';
        let confidence = 'low';

        // Sustained memory growth
        if (memoryGrowthRate > 1024 * 50) { // 50KB per iteration growth rate
            leakDetected = true;
            leakType = 'memory';
            confidence = memoryGrowthRate > 1024 * 200 ? 'high' : 'medium';
        }

        // DOM node accumulation
        if (domGrowthRate > 5) { // 5 nodes per iteration
            leakDetected = true;
            leakType = 'dom';
            confidence = domGrowthRate > 10 ? 'high' : 'medium';
        }

        // Event listener accumulation
        if (eventListenerGrowthRate > 2) { // 2 listeners per iteration
            leakDetected = true;
            leakType = 'event-listeners';
            confidence = eventListenerGrowthRate > 5 ? 'high' : 'medium';
        }

        return {
            leakDetected,
            leakType,
            confidence,
            metrics: {
                memoryGrowthRate,
                domGrowthRate,
                eventListenerGrowthRate,
                initialMemory: memoryUsage[0],
                finalMemory: memoryUsage[memoryUsage.length - 1],
                memoryIncrease: memoryUsage[memoryUsage.length - 1] - memoryUsage[0],
                initialDomNodes: domNodes[0],
                finalDomNodes: domNodes[domNodes.length - 1],
                domIncrease: domNodes[domNodes.length - 1] - domNodes[0]
            }
        };
    }

    calculateGrowthRate(values) {
        if (values.length < 2) return 0;

        const firstHalf = values.slice(0, Math.floor(values.length / 2));
        const secondHalf = values.slice(Math.floor(values.length / 2));

        const firstAvg = firstHalf.reduce((a, b) => a + b, 0) / firstHalf.length;
        const secondAvg = secondHalf.reduce((a, b) => a + b, 0) / secondHalf.length;

        return secondAvg - firstAvg;
    }

    generateReport(result) {
        const timestamp = new Date().toISOString().replace(/[:.]/g, '-');
        const reportPath = path.join(__dirname, '../performance-reports', `memory-leak-report-${timestamp}.json`);

        const report = {
            timestamp: new Date().toISOString(),
            duration: result.duration,
            iterations: result.iterations,
            leakDetected: result.leakDetected,
            analysis: result.analysis,
            summary: {
                leakDetected: result.leakDetected,
                leakType: result.analysis.leakType,
                confidence: result.analysis.confidence,
                memoryIncrease: result.analysis.metrics.memoryIncrease,
                domIncrease: result.analysis.metrics.domIncrease
            }
        };

        fs.writeFileSync(reportPath, JSON.stringify(report, null, 2));
        console.log(`Memory leak report saved to: ${reportPath}`);

        return report;
    }

    generateConsoleReport(result) {
        console.log('\n=== Memory Leak Detection Report ===');
        console.log(`Duration: ${result.duration / 1000 / 60} minutes`);
        console.log(`Iterations: ${result.iterations}`);
        console.log(`Leak Detected: ${result.leakDetected ? 'YES' : 'NO'}`);

        if (result.leakDetected) {
            const analysis = result.analysis;
            console.log(`Leak Type: ${analysis.leakType}`);
            console.log(`Confidence: ${analysis.confidence}`);
            console.log(`Memory Increase: ${(analysis.metrics.memoryIncrease / 1024 / 1024).toFixed(2)} MB`);
            console.log(`DOM Node Increase: ${analysis.metrics.domIncrease}`);
        }

        console.log('\n=== End Report ===\n');
    }

    async run(options = {}) {
        try {
            const {
                url,
                duration = 300000,
                interval = 30000,
                actions = []
            } = options;

            if (!url) {
                throw new Error('URL is required for memory leak detection');
            }

            console.log('Starting memory leak detection...');
            console.log(`URL: ${url}`);
            console.log(`Duration: ${duration / 1000 / 60} minutes`);

            const result = await this.detectLeaks(url, { duration, interval, actions });
            const report = this.generateReport(result);
            this.generateConsoleReport(result);

            // Exit with error code if leaks detected
            if (result.leakDetected) {
                console.error('Memory leaks detected!');
                process.exit(1);
            }

            return report;

        } catch (error) {
            console.error('Memory leak detection failed:', error.message);
            process.exit(1);
        }
    }
}

// CLI interface
if (require.main === module) {
    const args = process.argv.slice(2);
    const options = {};

    for (let i = 0; i < args.length; i++) {
        switch (args[i]) {
            case '--url':
                options.url = args[++i];
                break;
            case '--duration':
                options.duration = parseInt(args[++i]) * 1000 * 60; // Convert minutes to milliseconds
                break;
            case '--interval':
                options.interval = parseInt(args[++i]) * 1000; // Convert seconds to milliseconds
                break;
            case '--help':
                console.log('Usage: node memory-leak-detector.js [options]');
                console.log('');
                console.log('Options:');
                console.log('  --url <url>              URL to test (required)');
                console.log('  --duration <minutes>     Test duration in minutes (default: 5)');
                console.log('  --interval <seconds>     Monitoring interval in seconds (default: 30)');
                console.log('  --help                   Show this help');
                process.exit(0);
        }
    }

    const detector = new MemoryLeakDetector();
    detector.run(options);
}

module.exports = MemoryLeakDetector;
