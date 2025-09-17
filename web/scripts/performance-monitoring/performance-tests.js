#!/usr/bin/env node

/**
 * Automated Performance Tests
 * Runs performance test suites and benchmarks
 */

const { execSync, spawn } = require('child_process');
const fs = require('fs');
const path = require('path');

const BASELINE_FILE = path.join(__dirname, '../performance-baselines/performance-test-results.json');
const REPORT_DIR = path.join(__dirname, '../performance-reports');
const TEST_RESULTS_DIR = path.join(__dirname, '../../test-results');

class PerformanceTestRunner {
    constructor() {
        this.ensureDirectories();
        this.baseline = this.loadBaseline();
    }

    ensureDirectories() {
        if (!fs.existsSync(REPORT_DIR)) {
            fs.mkdirSync(REPORT_DIR, { recursive: true });
        }
        const baselineDir = path.dirname(BASELINE_FILE);
        if (!fs.existsSync(baselineDir)) {
            fs.mkdirSync(baselineDir, { recursive: true });
        }
        if (!fs.existsSync(TEST_RESULTS_DIR)) {
            fs.mkdirSync(TEST_RESULTS_DIR, { recursive: true });
        }
    }

    loadBaseline() {
        try {
            if (fs.existsSync(BASELINE_FILE)) {
                return JSON.parse(fs.readFileSync(BASELINE_FILE, 'utf8'));
            }
        } catch (error) {
            console.warn('Could not load performance test baseline:', error.message);
        }
        return {};
    }

    saveBaseline(data) {
        fs.writeFileSync(BASELINE_FILE, JSON.stringify(data, null, 2));
    }

    async runUnitTests() {
        console.log('Running unit performance tests...');

        try {
            // Run vitest with performance profiling
            const result = execSync('cd ../.. && npm run test', {
                cwd: path.join(__dirname, '../../'),
                encoding: 'utf8',
                stdio: 'pipe'
            });

            // Parse test results (simplified - would need to be adapted to actual test output format)
            const testResults = this.parseTestOutput(result);
            console.log(`Unit tests completed: ${testResults.passed}/${testResults.total} passed`);

            return testResults;
        } catch (error) {
            console.error('Unit tests failed:', error.message);
            return { passed: 0, total: 0, failed: 0, duration: 0, error: error.message };
        }
    }

    async runComponentBenchmarks() {
        console.log('Running component performance benchmarks...');

        const benchmarks = [
            { name: 'React Component Render', test: 'component-render.perf.test.js' },
            { name: 'Large List Virtualization', test: 'virtual-list.perf.test.js' },
            { name: 'Code Editor Performance', test: 'editor.perf.test.js' },
            { name: 'AI Model Loading', test: 'ai-model.perf.test.js' }
        ];

        const results = {};

        for (const benchmark of benchmarks) {
            console.log(`Running benchmark: ${benchmark.name}`);

            try {
                const startTime = Date.now();
                const result = execSync(`cd ../.. && npm run test -- ${benchmark.test}`, {
                    cwd: path.join(__dirname, '../../'),
                    encoding: 'utf8',
                    stdio: 'pipe',
                    timeout: 30000 // 30 second timeout per benchmark
                });
                const duration = Date.now() - startTime;

                results[benchmark.name] = {
                    duration,
                    success: true,
                    output: result.substring(0, 500) // First 500 chars of output
                };

                console.log(`  ✓ ${benchmark.name}: ${duration}ms`);
            } catch (error) {
                results[benchmark.name] = {
                    duration: 0,
                    success: false,
                    error: error.message
                };
                console.log(`  ✗ ${benchmark.name}: Failed`);
            }
        }

        return results;
    }

    async runBuildPerformanceTest() {
        console.log('Running build performance test...');

        try {
            const startTime = Date.now();
            execSync('cd ../.. && npm run build', {
                cwd: path.join(__dirname, '../../'),
                encoding: 'utf8',
                stdio: 'pipe'
            });
            const buildTime = Date.now() - startTime;

            console.log(`Build completed in: ${buildTime}ms`);

            return {
                buildTime,
                success: true
            };
        } catch (error) {
            console.error('Build performance test failed:', error.message);
            return {
                buildTime: 0,
                success: false,
                error: error.message
            };
        }
    }

    async runLighthouseAudit(url) {
        console.log('Running Lighthouse performance audit...');

        if (!url) {
            console.log('No URL provided for Lighthouse audit, skipping...');
            return null;
        }

        try {
            // Using puppeteer for a simple performance check (Lighthouse would be ideal but more complex)
            const puppeteer = require('puppeteer');
            const browser = await puppeteer.launch({ headless: true });
            const page = await browser.newPage();

            const startTime = Date.now();
            await page.goto(url, { waitUntil: 'networkidle0', timeout: 60000 });
            const loadTime = Date.now() - startTime;

            // Get basic performance metrics
            const metrics = await page.evaluate(() => {
                const perf = performance.getEntriesByType('navigation')[0];
                return {
                    domContentLoaded: perf.domContentLoadedEventEnd - perf.domContentLoadedEventStart,
                    loadComplete: perf.loadEventEnd - perf.loadEventStart,
                    resources: performance.getEntriesByType('resource').length
                };
            });

            await browser.close();

            const result = {
                loadTime,
                ...metrics,
                success: true,
                timestamp: new Date().toISOString()
            };

            console.log(`Lighthouse-style audit completed: ${loadTime}ms load time`);

            return result;
        } catch (error) {
            console.error('Lighthouse audit failed:', error.message);
            return {
                success: false,
                error: error.message,
                timestamp: new Date().toISOString()
            };
        }
    }

    parseTestOutput(output) {
        // Simple parsing - would need to be adapted to actual test output format
        const passed = (output.match(/✓/g) || []).length;
        const failed = (output.match(/✗/g) || []).length;
        const total = passed + failed;

        // Extract duration if available
        const durationMatch = output.match(/(\d+)ms/);
        const duration = durationMatch ? parseInt(durationMatch[1]) : 0;

        return {
            passed,
            failed,
            total,
            duration,
            output: output.substring(0, 1000) // First 1000 chars
        };
    }

    analyzeResults(results) {
        const analysis = {
            overall: {
                score: 0,
                grade: 'F'
            },
            categories: {}
        };

        let totalScore = 0;
        let categoriesCount = 0;

        // Analyze unit tests
        if (results.unitTests) {
            const unitScore = results.unitTests.total > 0 ? (results.unitTests.passed / results.unitTests.total) * 100 : 0;
            analysis.categories.unitTests = {
                score: unitScore,
                grade: this.scoreToGrade(unitScore),
                details: results.unitTests
            };
            totalScore += unitScore;
            categoriesCount++;
        }

        // Analyze component benchmarks
        if (results.componentBenchmarks) {
            const benchmarkResults = Object.values(results.componentBenchmarks);
            const successfulBenchmarks = benchmarkResults.filter(b => b.success).length;
            const benchmarkScore = (successfulBenchmarks / benchmarkResults.length) * 100;

            analysis.categories.componentBenchmarks = {
                score: benchmarkScore,
                grade: this.scoreToGrade(benchmarkScore),
                successful: successfulBenchmarks,
                total: benchmarkResults.length,
                details: results.componentBenchmarks
            };
            totalScore += benchmarkScore;
            categoriesCount++;
        }

        // Analyze build performance
        if (results.buildPerformance) {
            const buildScore = results.buildPerformance.success ? Math.max(0, 100 - (results.buildPerformance.buildTime / 1000)) : 0;
            analysis.categories.buildPerformance = {
                score: Math.max(0, buildScore),
                grade: this.scoreToGrade(buildScore),
                buildTime: results.buildPerformance.buildTime,
                success: results.buildPerformance.success
            };
            totalScore += buildScore;
            categoriesCount++;
        }

        // Analyze Lighthouse audit
        if (results.lighthouseAudit && results.lighthouseAudit.success) {
            // Score based on load time (faster is better)
            const loadTimeScore = Math.max(0, 100 - (results.lighthouseAudit.loadTime / 100));
            analysis.categories.lighthouseAudit = {
                score: loadTimeScore,
                grade: this.scoreToGrade(loadTimeScore),
                loadTime: results.lighthouseAudit.loadTime,
                details: results.lighthouseAudit
            };
            totalScore += loadTimeScore;
            categoriesCount++;
        }

        // Calculate overall score
        analysis.overall.score = categoriesCount > 0 ? totalScore / categoriesCount : 0;
        analysis.overall.grade = this.scoreToGrade(analysis.overall.score);

        return analysis;
    }

    scoreToGrade(score) {
        if (score >= 90) return 'A';
        if (score >= 80) return 'B';
        if (score >= 70) return 'C';
        if (score >= 60) return 'D';
        return 'F';
    }

    compareWithBaseline(currentAnalysis) {
        const changes = {};
        const warnings = [];
        const errors = [];

        if (!this.baseline.overallScore) {
            console.log('No performance test baseline available. This run will establish the baseline.');
            return { changes: {}, warnings: [], errors: [] };
        }

        // Compare overall score
        const currentScore = currentAnalysis.overall.score;
        const baselineScore = this.baseline.overallScore;
        const scoreDiff = currentScore - baselineScore;

        changes.overallScore = {
            current: currentScore,
            baseline: baselineScore,
            difference: scoreDiff,
            percentChange: ((scoreDiff / baselineScore) * 100)
        };

        // Alert thresholds
        if (scoreDiff < -10) {
            warnings.push(`Performance score decreased by ${Math.abs(scoreDiff).toFixed(1)} points (${Math.abs(changes.overallScore.percentChange).toFixed(1)}%)`);
        }
        if (scoreDiff < -20) {
            errors.push(`Performance score CRITICALLY decreased by ${Math.abs(scoreDiff).toFixed(1)} points (${Math.abs(changes.overallScore.percentChange).toFixed(1)}%)`);
        }

        // Check individual categories for regressions
        ['unitTests', 'componentBenchmarks', 'buildPerformance', 'lighthouseAudit'].forEach(category => {
            if (currentAnalysis.categories[category] && this.baseline.categories && this.baseline.categories[category]) {
                const current = currentAnalysis.categories[category].score;
                const baseline = this.baseline.categories[category].score;
                const categoryDiff = current - baseline;

                if (categoryDiff < -15) {
                    warnings.push(`${category} performance decreased by ${Math.abs(categoryDiff).toFixed(1)} points`);
                }
            }
        });

        return { changes, warnings, errors };
    }

    generateReport(results, analysis, comparison) {
        const timestamp = new Date().toISOString().replace(/[:.]/g, '-');
        const reportPath = path.join(REPORT_DIR, `performance-test-report-${timestamp}.json`);

        const report = {
            timestamp: new Date().toISOString(),
            buildInfo: {
                nodeVersion: process.version,
                platform: process.platform,
                arch: process.arch
            },
            results,
            analysis,
            comparison: comparison.changes,
            summary: {
                overallScore: analysis.overall.score,
                overallGrade: analysis.overall.grade,
                warnings: comparison.warnings.length,
                errors: comparison.errors.length,
                warningsList: comparison.warnings,
                errorsList: comparison.errors
            }
        };

        fs.writeFileSync(reportPath, JSON.stringify(report, null, 2));
        console.log(`Performance test report saved to: ${reportPath}`);

        return report;
    }

    generateConsoleReport(analysis, comparison) {
        console.log('\n=== Performance Test Results ===');
        console.log(`Timestamp: ${new Date().toISOString()}`);
        console.log(`Overall Score: ${analysis.overall.score.toFixed(1)} (${analysis.overall.grade})`);

        console.log('\n--- Category Scores ---');
        Object.entries(analysis.categories).forEach(([category, data]) => {
            console.log(`${category}: ${data.score.toFixed(1)} (${data.grade})`);
        });

        if (comparison.changes.overallScore) {
            const change = comparison.changes.overallScore;
            console.log('\n--- Score Changes ---');
            const changeSymbol = change.difference > 0 ? '🟢' : '🔴';
            const changeColor = change.difference > 0 ? '\x1b[32m' : '\x1b[31m';
            const resetColor = '\x1b[0m';
            console.log(`${changeSymbol} Overall Score: ${change.current.toFixed(1)} vs ${change.baseline.toFixed(1)} (${changeColor}${change.difference > 0 ? '+' : ''}${change.difference.toFixed(1)} points, ${change.percentChange.toFixed(1)}%)${resetColor}`);
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
                skipUnitTests = false,
                skipBenchmarks = false,
                skipBuildTest = false,
                skipLighthouse = false,
                updateBaseline = true
            } = options;

            console.log('Starting automated performance tests...');

            const results = {};

            // Run unit tests
            if (!skipUnitTests) {
                results.unitTests = await this.runUnitTests();
            }

            // Run component benchmarks
            if (!skipBenchmarks) {
                results.componentBenchmarks = await this.runComponentBenchmarks();
            }

            // Run build performance test
            if (!skipBuildTest) {
                results.buildPerformance = await this.runBuildPerformanceTest();
            }

            // Run Lighthouse-style audit
            if (!skipLighthouse && url) {
                results.lighthouseAudit = await this.runLighthouseAudit(url);
            }

            // Analyze results
            const analysis = this.analyzeResults(results);
            const comparison = this.compareWithBaseline(analysis);

            this.generateConsoleReport(analysis, comparison);
            const report = this.generateReport(results, analysis, comparison);

            // Update baseline if requested and no critical issues
            if (updateBaseline && comparison.errors.length === 0) {
                const baselineData = {
                    timestamp: new Date().toISOString(),
                    overallScore: analysis.overall.score,
                    overallGrade: analysis.overall.grade,
                    categories: analysis.categories
                };

                this.saveBaseline(baselineData);
                console.log('Performance test baseline updated.');
            } else if (comparison.errors.length > 0) {
                console.warn('Baseline NOT updated due to critical performance regressions.');
            }

            // Exit with error code if there are critical issues
            if (comparison.errors.length > 0) {
                console.error(`Performance tests failed with ${comparison.errors.length} critical issues.`);
                process.exit(1);
            }

            return report;

        } catch (error) {
            console.error('Performance tests failed:', error.message);
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
            case '--skip-unit-tests':
                options.skipUnitTests = true;
                break;
            case '--skip-benchmarks':
                options.skipBenchmarks = true;
                break;
            case '--skip-build-test':
                options.skipBuildTest = true;
                break;
            case '--skip-lighthouse':
                options.skipLighthouse = true;
                break;
            case '--no-baseline-update':
                options.updateBaseline = false;
                break;
            case '--help':
                console.log('Usage: node performance-tests.js [options]');
                console.log('');
                console.log('Options:');
                console.log('  --url <url>              URL for Lighthouse audit');
                console.log('  --skip-unit-tests        Skip unit test performance');
                console.log('  --skip-benchmarks        Skip component benchmarks');
                console.log('  --skip-build-test        Skip build performance test');
                console.log('  --skip-lighthouse        Skip Lighthouse audit');
                console.log('  --no-baseline-update     Don\'t update baseline with results');
                console.log('  --help                   Show this help');
                process.exit(0);
        }
    }

    const testRunner = new PerformanceTestRunner();
    testRunner.run(options);
}

module.exports = PerformanceTestRunner;
