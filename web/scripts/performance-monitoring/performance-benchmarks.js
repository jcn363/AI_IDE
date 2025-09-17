#!/usr/bin/env node

/**
 * Performance Benchmarks
 * Configures and runs detailed performance benchmarks
 */

const fs = require('fs');
const path = require('path');

class PerformanceBenchmarks {
    constructor() {
        this.benchmarks = [];
        this.configPath = path.join(__dirname, '../performance-configs/benchmarks.json');
        this.resultsPath = path.join(__dirname, '../performance-reports');
        this.loadConfig();
    }

    loadConfig() {
        try {
            if (fs.existsSync(this.configPath)) {
                const config = JSON.parse(fs.readFileSync(this.configPath, 'utf8'));
                this.benchmarks = config.benchmarks || [];
            }
        } catch (error) {
            console.warn('Could not load benchmark configuration:', error.message);
        }
    }

    saveConfig() {
        const configDir = path.dirname(this.configPath);
        if (!fs.existsSync(configDir)) {
            fs.mkdirSync(configDir, { recursive: true });
        }

        const config = {
            benchmarks: this.benchmarks,
            timestamp: new Date().toISOString()
        };

        fs.writeFileSync(this.configPath, JSON.stringify(config, null, 2));
    }

    addBenchmark(name, config) {
        const benchmark = {
            id: Date.now().toString(),
            name,
            type: config.type || 'custom',
            description: config.description || '',
            config,
            enabled: true,
            createdAt: new Date().toISOString()
        };

        this.benchmarks.push(benchmark);
        this.saveConfig();
        return benchmark;
    }

    removeBenchmark(id) {
        const index = this.benchmarks.findIndex(b => b.id === id);
        if (index !== -1) {
            this.benchmarks.splice(index, 1);
            this.saveConfig();
            return true;
        }
        return false;
    }

    async runBenchmark(benchmark) {
        const startTime = Date.now();

        try {
            switch (benchmark.type) {
                case 'render':
                    return await this.runRenderBenchmark(benchmark);
                case 'memory':
                    return await this.runMemoryBenchmark(benchmark);
                case 'interaction':
                    return await this.runInteractionBenchmark(benchmark);
                case 'custom':
                    return await this.runCustomBenchmark(benchmark);
                default:
                    throw new Error(`Unknown benchmark type: ${benchmark.type}`);
            }
        } catch (error) {
            return {
                success: false,
                error: error.message,
                duration: Date.now() - startTime
            };
        }
    }

    async runRenderBenchmark(benchmark) {
        // Enhanced render benchmark implementation
        const puppeteer = require('puppeteer');

        const browser = await puppeteer.launch({ 
            headless: true,
            args: ['--no-sandbox', '--disable-setuid-sandbox']
        });
        const page = await browser.newPage();

        // Navigate to the application
        const url = benchmark.config.url || 'http://localhost:8080';
        await page.goto(url, { waitUntil: 'networkidle0' });

        const results = [];
        const iterations = benchmark.config.iterations || 10;

        // Run multiple render cycles
        for (let i = 0; i < iterations; i++) {
            const startTime = performance.now();

            // Trigger component re-render based on configuration
            await page.evaluate((config) => {
                if (config.selector) {
                    const element = document.querySelector(config.selector);
                    if (element) {
                        // Force re-render by toggling visibility or updating content
                        element.style.display = 'none';
                        element.offsetHeight; // Force reflow
                        element.style.display = '';
                    }
                }
                
                // Trigger React re-render if possible
                if (window.React && config.reactComponent) {
                    // Force update React components
                    const event = new CustomEvent('forceUpdate');
                    document.dispatchEvent(event);
                }

                return true;
            }, benchmark.config);

            // Wait for render to complete
            await page.waitForTimeout(100);

            const renderTime = performance.now() - startTime;
            results.push(renderTime);
        }

        await browser.close();

        return {
            success: true,
            type: 'render',
            results,
            average: results.reduce((a, b) => a + b, 0) / results.length,
            min: Math.min(...results),
            max: Math.max(...results),
            median: results.sort((a, b) => a - b)[Math.floor(results.length / 2)]
        };
    }

    async runMemoryBenchmark(benchmark) {
        // Enhanced memory benchmark implementation
        const puppeteer = require('puppeteer');

        const browser = await puppeteer.launch({ 
            headless: true,
            args: ['--no-sandbox', '--disable-setuid-sandbox', '--enable-precise-memory-info']
        });
        const page = await browser.newPage();

        const url = benchmark.config.url || 'http://localhost:8080';
        await page.goto(url, { waitUntil: 'networkidle0' });

        const memoryReadings = [];
        const duration = benchmark.config.duration || 30;

        // Monitor memory over time
        for (let i = 0; i < duration; i++) {
            const memoryInfo = await page.evaluate(() => {
                if (window.performance.memory) {
                    return {
                        used: window.performance.memory.usedJSHeapSize,
                        total: window.performance.memory.totalJSHeapSize,
                        limit: window.performance.memory.jsHeapSizeLimit
                    };
                }
                return { used: 0, total: 0, limit: 0 };
            });

            memoryReadings.push({
                timestamp: Date.now(),
                ...memoryInfo
            });

            // Perform memory-intensive operations if configured
            if (benchmark.config.stressTest) {
                await page.evaluate(() => {
                    // Create and destroy objects to stress memory
                    const objects = [];
                    for (let j = 0; j < 1000; j++) {
                        objects.push({ data: new Array(1000).fill(Math.random()) });
                    }
                    // Let GC clean up
                    objects.length = 0;
                });
            }

            await new Promise(resolve => setTimeout(resolve, 1000));
        }

        await browser.close();

        const usedMemory = memoryReadings.map(r => r.used);
        return {
            success: true,
            type: 'memory',
            readings: memoryReadings,
            average: usedMemory.reduce((a, b) => a + b, 0) / usedMemory.length,
            growth: usedMemory[usedMemory.length - 1] - usedMemory[0],
            peak: Math.max(...usedMemory),
            efficiency: memoryReadings[memoryReadings.length - 1].used / memoryReadings[memoryReadings.length - 1].total
        };
    }

    async runInteractionBenchmark(benchmark) {
        // Enhanced interaction benchmark implementation
        const puppeteer = require('puppeteer');

        const browser = await puppeteer.launch({ 
            headless: true,
            args: ['--no-sandbox', '--disable-setuid-sandbox']
        });
        const page = await browser.newPage();

        const url = benchmark.config.url || 'http://localhost:8080';
        await page.goto(url, { waitUntil: 'networkidle0' });

        const interactionTimes = [];
        const interactions = benchmark.config.interactions || [
            { type: 'click', selector: 'button' },
            { type: 'type', selector: 'input', text: 'test input' },
            { type: 'scroll', distance: 500 }
        ];

        // Perform interaction sequences
        for (const interaction of interactions) {
            const startTime = performance.now();

            try {
                switch (interaction.type) {
                    case 'click':
                        if (interaction.selector) {
                            await page.click(interaction.selector);
                        }
                        break;
                    case 'type':
                        if (interaction.selector) {
                            await page.type(interaction.selector, interaction.text || 'test');
                        }
                        break;
                    case 'scroll':
                        await page.evaluate((distance) => {
                            window.scrollTo(0, distance || 500);
                        }, interaction.distance);
                        break;
                    case 'hover':
                        if (interaction.selector) {
                            await page.hover(interaction.selector);
                        }
                        break;
                }

                // Wait for interaction to complete
                await page.waitForTimeout(100);

                const interactionTime = performance.now() - startTime;
                interactionTimes.push({
                    type: interaction.type,
                    time: interactionTime,
                    selector: interaction.selector
                });
            } catch (error) {
                interactionTimes.push({
                    type: interaction.type,
                    time: -1,
                    error: error.message
                });
            }
        }

        await browser.close();

        const validTimes = interactionTimes.filter(t => t.time > 0).map(t => t.time);
        return {
            success: true,
            type: 'interaction',
            interactions: interactionTimes,
            average: validTimes.length > 0 ? validTimes.reduce((a, b) => a + b, 0) / validTimes.length : 0,
            fastest: validTimes.length > 0 ? Math.min(...validTimes) : 0,
            slowest: validTimes.length > 0 ? Math.max(...validTimes) : 0
        };
    }

    async runCustomBenchmark(benchmark) {
        // Enhanced custom benchmark implementation
        const { spawn } = require('child_process');

        return new Promise((resolve) => {
            const startTime = Date.now();
            const command = benchmark.config.command;
            const args = benchmark.config.args || [];
            const timeout = benchmark.config.timeout || 30000;

            const process = spawn(command, args, {
                stdio: ['pipe', 'pipe', 'pipe'],
                timeout
            });

            let stdout = '';
            let stderr = '';

            process.stdout.on('data', (data) => {
                stdout += data.toString();
            });

            process.stderr.on('data', (data) => {
                stderr += data.toString();
            });

            process.on('close', (code) => {
                const duration = Date.now() - startTime;
                
                resolve({
                    success: code === 0,
                    type: 'custom',
                    exitCode: code,
                    duration,
                    stdout,
                    stderr,
                    parsed: this.parseCustomOutput(stdout, benchmark.config.parser)
                });
            });

            process.on('error', (error) => {
                resolve({
                    success: false,
                    type: 'custom',
                    error: error.message,
                    duration: Date.now() - startTime
                });
            });

            // Set timeout
            setTimeout(() => {
                process.kill('SIGTERM');
                resolve({
                    success: false,
                    type: 'custom',
                    error: 'Timeout exceeded',
                    duration: Date.now() - startTime
                });
            }, timeout);
        });
    }

    parseCustomOutput(output, parser) {
        if (!parser) return output;

        // Simple parsing based on parser config
        switch (parser.type) {
            case 'regex':
                const match = output.match(new RegExp(parser.pattern));
                return match ? match[1] : null;
            case 'json':
                try {
                    return JSON.parse(output);
                } catch {
                    return null;
                }
            default:
                return output;
        }
    }

    async runAllBenchmarks() {
        const results = {
            timestamp: new Date().toISOString(),
            benchmarks: []
        };

        console.log(`Running ${this.benchmarks.filter(b => b.enabled).length} benchmarks...`);

        for (const benchmark of this.benchmarks.filter(b => b.enabled)) {
            console.log(`Running benchmark: ${benchmark.name}`);

            const result = await this.runBenchmark(benchmark);

            results.benchmarks.push({
                benchmark: benchmark.name,
                id: benchmark.id,
                type: benchmark.type,
                result
            });

            const status = result.success ? '✓' : '✗';
            console.log(`  ${status} ${benchmark.name}: ${result.success ? 'Completed' : 'Failed'}`);
        }

        return results;
    }

    generateReport(results) {
        const timestamp = new Date().toISOString().replace(/[:.]/g, '-');
        const reportPath = path.join(this.resultsPath, `benchmark-report-${timestamp}.json`);

        const report = {
            timestamp: results.timestamp,
            summary: {
                totalBenchmarks: results.benchmarks.length,
                successfulBenchmarks: results.benchmarks.filter(b => b.result.success).length,
                failedBenchmarks: results.benchmarks.filter(b => !b.result.success).length
            },
            results: results.benchmarks
        };

        fs.writeFileSync(reportPath, JSON.stringify(report, null, 2));
        console.log(`Benchmark report saved to: ${reportPath}`);

        return report;
    }

    listBenchmarks() {
        console.log('\nConfigured Benchmarks:');
        console.log('======================');

        if (this.benchmarks.length === 0) {
            console.log('No benchmarks configured.');
            return;
        }

        this.benchmarks.forEach((benchmark, index) => {
            const status = benchmark.enabled ? '🟢' : '🔴';
            console.log(`${index + 1}. ${status} ${benchmark.name} (${benchmark.type})`);
            console.log(`   ${benchmark.description || 'No description'}`);
        });

        console.log('');
    }

    // CLI methods
    async run(options = {}) {
        if (options.list) {
            this.listBenchmarks();
            return;
        }

        const results = await this.runAllBenchmarks();
        const report = this.generateReport(results);

        console.log('\n=== Benchmark Summary ===');
        console.log(`Total: ${report.summary.totalBenchmarks}`);
        console.log(`Successful: ${report.summary.successfulBenchmarks}`);
        console.log(`Failed: ${report.summary.failedBenchmarks}`);

        return report;
    }
}

// CLI interface
if (require.main === module) {
    const args = process.argv.slice(2);
    const options = {};

    for (let i = 0; i < args.length; i++) {
        switch (args[i]) {
            case '--list':
                options.list = true;
                break;
            case '--help':
                console.log('Usage: node performance-benchmarks.js [options]');
                console.log('');
                console.log('Options:');
                console.log('  --list    List configured benchmarks');
                console.log('  --help    Show this help');
                process.exit(0);
        }
    }

    const benchmarks = new PerformanceBenchmarks();
    benchmarks.run(options);
}

module.exports = PerformanceBenchmarks;