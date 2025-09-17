#!/usr/bin/env node

/**
 * Continuous Performance Monitoring
 * Automates regular performance monitoring and alerting
 */

const fs = require('fs');
const path = require('path');
const { execSync } = require('child_process');

const CONFIG_FILE = path.join(__dirname, '../performance-configs/continuous-monitoring.json');
const REPORT_DIR = path.join(__dirname, '../performance-reports');

class ContinuousMonitor {
    constructor() {
        this.config = this.loadConfig();
        this.ensureDirectories();
    }

    loadConfig() {
        try {
            if (fs.existsSync(CONFIG_FILE)) {
                return JSON.parse(fs.readFileSync(CONFIG_FILE, 'utf8'));
            }
        } catch (error) {
            console.warn('Could not load continuous monitoring config:', error.message);
        }

        // Default configuration
        return {
            enabled: false,
            schedule: '0 */6 * * *', // Every 6 hours
            url: process.env.PERFORMANCE_MONITOR_URL || 'http://localhost:3000',
            checks: {
                bundleSize: true,
                loadTime: true,
                memoryUsage: true,
                performanceTests: false, // Too expensive for continuous
                memoryLeaks: false // Too expensive for continuous
            },
            thresholds: {
                bundleSizeIncrease: 10, // 10% increase
                loadTimeIncrease: 15, // 15% increase
                memoryLeakThreshold: 50 // MB growth
            },
            notifications: {
                enabled: false,
                webhook: null,
                email: null
            }
        };
    }

    saveConfig() {
        const configDir = path.dirname(CONFIG_FILE);
        if (!fs.existsSync(configDir)) {
            fs.mkdirSync(configDir, { recursive: true });
        }

        fs.writeFileSync(CONFIG_FILE, JSON.stringify(this.config, null, 2));
    }

    ensureDirectories() {
        if (!fs.existsSync(REPORT_DIR)) {
            fs.mkdirSync(REPORT_DIR, { recursive: true });
        }
    }

    updateConfig(updates) {
        this.config = { ...this.config, ...updates };
        this.saveConfig();
    }

    async runScheduledChecks() {
        console.log('Running scheduled performance checks...');

        const results = {
            timestamp: new Date().toISOString(),
            checks: {},
            alerts: []
        };

        // Bundle size check
        if (this.config.checks.bundleSize) {
            console.log('Running bundle size check...');
            try {
                const bundleResult = execSync('node bundle-size-monitor.js', {
                    cwd: __dirname,
                    encoding: 'utf8',
                    stdio: 'pipe'
                });

                results.checks.bundleSize = {
                    success: true,
                    output: bundleResult
                };
            } catch (error) {
                results.checks.bundleSize = {
                    success: false,
                    error: error.message
                };
                results.alerts.push({
                    type: 'bundle-size-failure',
                    message: 'Bundle size monitoring failed',
                    details: error.message
                });
            }
        }

        // Load time check
        if (this.config.checks.loadTime) {
            console.log('Running load time check...');
            try {
                const loadTimeResult = execSync(`node load-time-monitor.js --url "${this.config.url}" --no-baseline-update`, {
                    cwd: __dirname,
                    encoding: 'utf8',
                    stdio: 'pipe'
                });

                results.checks.loadTime = {
                    success: true,
                    output: loadTimeResult
                };
            } catch (error) {
                results.checks.loadTime = {
                    success: false,
                    error: error.message
                };
                results.alerts.push({
                    type: 'load-time-failure',
                    message: 'Load time monitoring failed',
                    details: error.message
                });
            }
        }

        // Memory usage check
        if (this.config.checks.memoryUsage) {
            console.log('Running memory usage check...');
            try {
                const memoryResult = execSync(`node memory-monitor.js --url "${this.config.url}" --duration 2 --interval 10 --no-baseline-update`, {
                    cwd: __dirname,
                    encoding: 'utf8',
                    stdio: 'pipe'
                });

                results.checks.memoryUsage = {
                    success: true,
                    output: memoryResult
                };
            } catch (error) {
                results.checks.memoryUsage = {
                    success: false,
                    error: error.message
                };
                results.alerts.push({
                    type: 'memory-usage-failure',
                    message: 'Memory usage monitoring failed',
                    details: error.message
                });
            }
        }

        // Generate summary report
        this.generateSummaryReport(results);

        // Send notifications if enabled
        if (this.config.notifications.enabled && results.alerts.length > 0) {
            await this.sendNotifications(results.alerts);
        }

        console.log(`Scheduled checks completed. ${results.alerts.length} alerts generated.`);
        return results;
    }

    generateSummaryReport(results) {
        const timestamp = new Date().toISOString().replace(/[:.]/g, '-');
        const reportPath = path.join(REPORT_DIR, `continuous-monitor-${timestamp}.json`);

        const summary = {
            timestamp: results.timestamp,
            config: this.config,
            results: results.checks,
            alerts: results.alerts,
            summary: {
                totalChecks: Object.keys(results.checks).length,
                successfulChecks: Object.values(results.checks).filter(c => c.success).length,
                failedChecks: Object.values(results.checks).filter(c => !c.success).length,
                alertsCount: results.alerts.length
            }
        };

        fs.writeFileSync(reportPath, JSON.stringify(summary, null, 2));
        console.log(`Continuous monitoring report saved to: ${reportPath}`);

        return summary;
    }

    async sendNotifications(alerts) {
        console.log(`Sending ${alerts.length} notifications...`);

        const message = {
            timestamp: new Date().toISOString(),
            alerts: alerts,
            summary: `${alerts.length} performance alerts detected`
        };

        // Webhook notification
        if (this.config.notifications.webhook) {
            try {
                const response = await fetch(this.config.notifications.webhook, {
                    method: 'POST',
                    headers: {
                        'Content-Type': 'application/json'
                    },
                    body: JSON.stringify(message)
                });

                if (!response.ok) {
                    console.error('Failed to send webhook notification');
                }
            } catch (error) {
                console.error('Webhook notification failed:', error.message);
            }
        }

        // Email notification (placeholder - would need email service)
        if (this.config.notifications.email) {
            console.log(`Would send email to: ${this.config.notifications.email}`);
            // Implement email sending logic here
        }
    }

    generateCronSchedule() {
        // Example cron schedule generation
        return this.config.schedule;
    }

    setupCronJob() {
        const cronSchedule = this.generateCronSchedule();
        const scriptPath = path.resolve(__dirname, 'continuous-monitor.js');

        console.log('To set up continuous monitoring, add this to your crontab:');
        console.log(`${cronSchedule} cd ${path.dirname(scriptPath)} && node ${scriptPath} run`);
        console.log('');
        console.log('Or use a process manager like PM2 for more reliable scheduling.');
    }

    showStatus() {
        console.log('\n=== Continuous Monitoring Status ===');
        console.log(`Enabled: ${this.config.enabled ? 'YES' : 'NO'}`);
        console.log(`Schedule: ${this.config.schedule}`);
        console.log(`URL: ${this.config.url}`);
        console.log('');

        console.log('Checks:');
        Object.entries(this.config.checks).forEach(([check, enabled]) => {
            console.log(`  ${check}: ${enabled ? 'ENABLED' : 'DISABLED'}`);
        });

        console.log('');
        console.log('Thresholds:');
        Object.entries(this.config.thresholds).forEach(([threshold, value]) => {
            console.log(`  ${threshold}: ${value}`);
        });

        console.log('');
        console.log('Notifications:');
        console.log(`  Enabled: ${this.config.notifications.enabled ? 'YES' : 'NO'}`);
        if (this.config.notifications.webhook) {
            console.log(`  Webhook: ${this.config.notifications.webhook}`);
        }
        if (this.config.notifications.email) {
            console.log(`  Email: ${this.config.notifications.email}`);
        }
    }

    async run(options = {}) {
        if (options.setup) {
            this.setupCronJob();
            return;
        }

        if (options.status) {
            this.showStatus();
            return;
        }

        if (options.configure) {
            // Interactive configuration (simplified)
            console.log('Continuous monitoring configuration:');
            console.log('Edit the config file at:', CONFIG_FILE);
            return;
        }

        // Run the checks
        await this.runScheduledChecks();
    }
}

// CLI interface
if (require.main === module) {
    const args = process.argv.slice(2);
    const options = {};

    for (let i = 0; i < args.length; i++) {
        switch (args[i]) {
            case 'run':
                // Default action
                break;
            case '--setup':
                options.setup = true;
                break;
            case '--status':
                options.status = true;
                break;
            case '--configure':
                options.configure = true;
                break;
            case '--help':
                console.log('Usage: node continuous-monitor.js [command] [options]');
                console.log('');
                console.log('Commands:');
                console.log('  run                    Run scheduled checks (default)');
                console.log('  --setup               Show setup instructions');
                console.log('  --status              Show current configuration');
                console.log('  --configure           Configure monitoring settings');
                console.log('  --help                Show this help');
                process.exit(0);
        }
    }

    const monitor = new ContinuousMonitor();
    monitor.run(options);
}

module.exports = ContinuousMonitor;
