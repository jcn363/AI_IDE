#!/usr/bin/env node

/**
 * Performance Report Generator
 * Aggregates and generates comprehensive performance reports
 */

const fs = require('fs');
const path = require('path');

const REPORT_DIR = path.join(__dirname, '../performance-reports');
const BASELINE_DIR = path.join(__dirname, '../performance-baselines');

class PerformanceReportGenerator {
    constructor() {
        this.ensureDirectories();
    }

    ensureDirectories() {
        if (!fs.existsSync(REPORT_DIR)) {
            fs.mkdirSync(REPORT_DIR, { recursive: true });
        }
        if (!fs.existsSync(BASELINE_DIR)) {
            fs.mkdirSync(BASELINE_DIR, { recursive: true });
        }
    }

    collectAllReports() {
        const reports = {};

        if (!fs.existsSync(REPORT_DIR)) {
            return reports;
        }

        const files = fs.readdirSync(REPORT_DIR);

        files.forEach(file => {
            if (path.extname(file) === '.json') {
                try {
                    const filePath = path.join(REPORT_DIR, file);
                    const content = JSON.parse(fs.readFileSync(filePath, 'utf8'));

                    // Categorize reports
                    if (file.includes('bundle-size-report')) {
                        if (!reports.bundleSize) reports.bundleSize = [];
                        reports.bundleSize.push(content);
                    } else if (file.includes('load-time-report')) {
                        if (!reports.loadTime) reports.loadTime = [];
                        reports.loadTime.push(content);
                    } else if (file.includes('memory-report')) {
                        if (!reports.memory) reports.memory = [];
                        reports.memory.push(content);
                    } else if (file.includes('performance-test-report')) {
                        if (!reports.performanceTests) reports.performanceTests = [];
                        reports.performanceTests.push(content);
                    } else if (file.includes('benchmark-report')) {
                        if (!reports.benchmarks) reports.benchmarks = [];
                        reports.benchmarks.push(content);
                    } else if (file.includes('memory-leak-report')) {
                        if (!reports.memoryLeaks) reports.memoryLeaks = [];
                        reports.memoryLeaks.push(content);
                    } else if (file.includes('continuous-monitor')) {
                        if (!reports.continuous) reports.continuous = [];
                        reports.continuous.push(content);
                    }
                } catch (error) {
                    console.warn(`Could not parse report file ${file}:`, error.message);
                }
            }
        });

        return reports;
    }

    generateComprehensiveReport(reports) {
        const report = {
            generatedAt: new Date().toISOString(),
            period: this.calculateReportPeriod(reports),
            summary: this.generateSummary(reports),
            trends: this.analyzeTrends(reports),
            recommendations: this.generateRecommendations(reports),
            details: reports
        };

        return report;
    }

    calculateReportPeriod(reports) {
        let earliest = null;
        let latest = null;

        Object.values(reports).forEach(category => {
            category.forEach(item => {
                const timestamp = item.timestamp || item.generatedAt;
                if (timestamp) {
                    const date = new Date(timestamp);
                    if (!earliest || date < earliest) earliest = date;
                    if (!latest || date > latest) latest = date;
                }
            });
        });

        return {
            start: earliest ? earliest.toISOString() : null,
            end: latest ? latest.toISOString() : null,
            duration: earliest && latest ? (latest - earliest) / (1000 * 60 * 60) : null // hours
        };
    }

    generateSummary(reports) {
        const summary = {
            totalReports: 0,
            categories: {},
            alerts: [],
            performance: {}
        };

        Object.entries(reports).forEach(([category, items]) => {
            summary.totalReports += items.length;
            summary.categories[category] = {
                count: items.length,
                latest: items.length > 0 ? items[items.length - 1] : null
            };

            // Extract alerts and performance metrics
            items.forEach(item => {
                if (item.summary) {
                    if (item.summary.warnings && item.summary.warnings > 0) {
                        summary.alerts.push({
                            category,
                            type: 'warning',
                            count: item.summary.warnings,
                            timestamp: item.timestamp
                        });
                    }
                    if (item.summary.errors && item.summary.errors > 0) {
                        summary.alerts.push({
                            category,
                            type: 'error',
                            count: item.summary.errors,
                            timestamp: item.timestamp
                        });
                    }
                }

                // Performance metrics
                if (item.analysis && item.analysis.overall) {
                    if (!summary.performance.overallScore) {
                        summary.performance.overallScore = [];
                    }
                    summary.performance.overallScore.push({
                        score: item.analysis.overall.score,
                        grade: item.analysis.overall.grade,
                        timestamp: item.timestamp
                    });
                }
            });
        });

        return summary;
    }

    analyzeTrends(reports) {
        const trends = {
            bundleSize: this.analyzeNumericTrend(reports.bundleSize, 'bundleSizes', 'size'),
            loadTime: this.analyzeNumericTrend(reports.loadTime, 'averages', 'totalLoadTime'),
            memoryUsage: this.analyzeNumericTrend(reports.memory, 'averages', 'usedJSHeapSize'),
            performanceScore: this.analyzeNumericTrend(reports.performanceTests, 'analysis.overall', 'score')
        };

        return trends;
    }

    analyzeNumericTrend(reportCategory, dataPath, metric) {
        if (!reportCategory || reportCategory.length === 0) {
            return { available: false };
        }

        const sorted = reportCategory.sort((a, b) => new Date(a.timestamp) - new Date(b.timestamp));
        const values = sorted.map(item => {
            const pathParts = dataPath.split('.');
            let value = item;
            for (const part of pathParts) {
                value = value ? value[part] : null;
            }
            return value ? value[metric] : null;
        }).filter(v => v !== null);

        if (values.length < 2) {
            return { available: true, insufficientData: true };
        }

        const first = values[0];
        const last = values[values.length - 1];
        const change = last - first;
        const percentChange = ((change / first) * 100);

        // Trend direction
        let direction = 'stable';
        if (percentChange > 5) direction = 'increasing';
        else if (percentChange < -5) direction = 'decreasing';

        // Calculate simple linear trend
        const slope = this.calculateSlope(values);

        return {
            available: true,
            direction,
            change,
            percentChange,
            slope,
            first,
            last,
            dataPoints: values.length
        };
    }

    calculateSlope(values) {
        // Simple linear regression slope calculation
        const n = values.length;
        const sumX = (n * (n - 1)) / 2;
        const sumY = values.reduce((a, b) => a + b, 0);
        const sumXY = values.reduce((sum, y, x) => sum + (x * y), 0);
        const sumXX = (n * (n - 1) * (2 * n - 1)) / 6;

        return (n * sumXY - sumX * sumY) / (n * sumXX - sumX * sumX);
    }

    generateRecommendations(reports) {
        const recommendations = [];

        // Bundle size recommendations
        if (reports.bundleSize && reports.bundleSize.length > 0) {
            const latest = reports.bundleSize[reports.bundleSize.length - 1];
            if (latest.summary && latest.summary.errors > 0) {
                recommendations.push({
                    priority: 'high',
                    category: 'bundle-size',
                    message: 'Critical bundle size increases detected. Consider code splitting or tree shaking.',
                    details: `Found ${latest.summary.errors} critical size issues.`
                });
            }
        }

        // Performance score recommendations
        if (reports.performanceTests && reports.performanceTests.length > 0) {
            const latest = reports.performanceTests[reports.performanceTests.length - 1];
            if (latest.analysis && latest.analysis.overall && latest.analysis.overall.score < 70) {
                recommendations.push({
                    priority: 'high',
                    category: 'performance',
                    message: 'Overall performance score is low. Focus on optimization.',
                    details: `Current score: ${latest.analysis.overall.score.toFixed(1)} (${latest.analysis.overall.grade})`
                });
            }
        }

        // Memory leak recommendations
        if (reports.memoryLeaks && reports.memoryLeaks.length > 0) {
            const latest = reports.memoryLeaks[reports.memoryLeaks.length - 1];
            if (latest.leakDetected) {
                recommendations.push({
                    priority: 'critical',
                    category: 'memory-leaks',
                    message: 'Memory leaks detected. Immediate investigation required.',
                    details: `Leak type: ${latest.analysis.leakType}, Confidence: ${latest.analysis.confidence}`
                });
            }
        }

        // Trend-based recommendations
        const trends = this.analyzeTrends(reports);
        if (trends.bundleSize.available && trends.bundleSize.direction === 'increasing' && trends.bundleSize.percentChange > 10) {
            recommendations.push({
                priority: 'medium',
                category: 'bundle-size-trend',
                message: 'Bundle size is trending upward. Monitor for optimization opportunities.',
                details: `${trends.bundleSize.percentChange.toFixed(1)}% increase over time.`
            });
        }

        return recommendations.sort((a, b) => {
            const priorityOrder = { critical: 0, high: 1, medium: 2, low: 3 };
            return priorityOrder[a.priority] - priorityOrder[b.priority];
        });
    }

    generateConsoleReport(report) {
        console.log('\n==================================================');
        console.log('           PERFORMANCE REPORT SUMMARY');
        console.log('==================================================');
        console.log(`Generated: ${new Date(report.generatedAt).toLocaleString()}`);
        console.log(`Period: ${report.period.start ? new Date(report.period.start).toLocaleDateString() : 'N/A'} - ${report.period.end ? new Date(report.period.end).toLocaleDateString() : 'N/A'}`);
        console.log('');

        // Summary
        console.log('📊 SUMMARY');
        console.log(`Total Reports: ${report.summary.totalReports}`);
        console.log(`Alert Count: ${report.summary.alerts.length}`);

        Object.entries(report.summary.categories).forEach(([category, data]) => {
            console.log(`  ${category}: ${data.count} reports`);
        });
        console.log('');

        // Trends
        console.log('📈 TRENDS');
        Object.entries(report.trends).forEach(([category, trend]) => {
            if (trend.available && !trend.insufficientData) {
                const direction = trend.direction === 'increasing' ? '📈' :
                                trend.direction === 'decreasing' ? '📉' : '➡️';
                console.log(`  ${category}: ${direction} ${trend.percentChange.toFixed(1)}% (${trend.direction})`);
            }
        });
        console.log('');

        // Recommendations
        if (report.recommendations.length > 0) {
            console.log('💡 RECOMMENDATIONS');
            report.recommendations.forEach((rec, index) => {
                const priority = rec.priority.toUpperCase();
                const icon = rec.priority === 'critical' ? '🚨' :
                            rec.priority === 'high' ? '⚠️' : 'ℹ️';
                console.log(`  ${index + 1}. ${icon} [${priority}] ${rec.message}`);
            });
        }

        console.log('\n==================================================\n');
    }

    saveReport(report, filename) {
        const timestamp = new Date().toISOString().replace(/[:.]/g, '-');
        const reportPath = path.join(REPORT_DIR, filename || `comprehensive-report-${timestamp}.json`);

        fs.writeFileSync(reportPath, JSON.stringify(report, null, 2));
        console.log(`Comprehensive performance report saved to: ${reportPath}`);

        return reportPath;
    }

    async run(options = {}) {
        const {
            save = true,
            filename,
            console = true
        } = options;

        console.log('Generating comprehensive performance report...');

        const reports = this.collectAllReports();
        const comprehensiveReport = this.generateComprehensiveReport(reports);

        if (console) {
            this.generateConsoleReport(comprehensiveReport);
        }

        if (save) {
            const reportPath = this.saveReport(comprehensiveReport, filename);
            return { report: comprehensiveReport, path: reportPath };
        }

        return { report: comprehensiveReport };
    }
}

// CLI interface
if (require.main === module) {
    const args = process.argv.slice(2);
    const options = {};

    for (let i = 0; i < args.length; i++) {
        switch (args[i]) {
            case '--no-save':
                options.save = false;
                break;
            case '--filename':
                options.filename = args[++i];
                break;
            case '--no-console':
                options.console = false;
                break;
            case '--help':
                console.log('Usage: node performance-report-generator.js [options]');
                console.log('');
                console.log('Options:');
                console.log('  --no-save         Don\'t save report to file');
                console.log('  --filename <file> Custom filename for report');
                console.log('  --no-console      Don\'t output to console');
                console.log('  --help            Show this help');
                process.exit(0);
        }
    }

    const generator = new PerformanceReportGenerator();
    generator.run(options);
}

module.exports = PerformanceReportGenerator;
