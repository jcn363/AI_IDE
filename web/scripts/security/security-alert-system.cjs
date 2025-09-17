#!/usr/bin/env node

/**
 * Security Alert System for Rust AI IDE
 * Handles notifications and alerts for security issues
 */

const fs = require('fs');
const path = require('path');

class SecurityAlertSystem {
    constructor() {
        this.config = this.loadConfig();
        this.outputDir = path.join(process.cwd(), 'security-reports');
    }

    loadConfig() {
        // Default configuration - can be overridden by environment variables
        return {
            webhookUrl: process.env.SECURITY_WEBHOOK_URL,
            emailTo: process.env.SECURITY_EMAIL_TO || 'security@company.com',
            slackChannel: process.env.SECURITY_SLACK_CHANNEL || '#security-alerts',
            criticalThreshold: parseInt(process.env.SECURITY_CRITICAL_THRESHOLD) || 1,
            highThreshold: parseInt(process.env.SECURITY_HIGH_THRESHOLD) || 5
        };
    }

    async sendAlerts() {
        console.log('🚨 Checking for security alerts...');

        try {
            // Read the latest security report
            const reportPath = path.join(this.outputDir, 'security-comprehensive-report.json');
            
            if (!fs.existsSync(reportPath)) {
                console.log('No security report found. Run security scans first.');
                return;
            }

            const report = JSON.parse(fs.readFileSync(reportPath, 'utf8'));
            
            const alerts = this.analyzeReportForAlerts(report);
            
            if (alerts.length > 0) {
                console.log(`Found ${alerts.length} security alerts to send`);
                
                for (const alert of alerts) {
                    await this.sendAlert(alert);
                }
            } else {
                console.log('✅ No security alerts to send');
            }

        } catch (error) {
            console.error('❌ Error sending security alerts:', error.message);
        }
    }

    analyzeReportForAlerts(report) {
        const alerts = [];
        
        const { summary } = report;
        
        // Critical alerts
        if (summary.criticalIssues >= this.config.criticalThreshold) {
            alerts.push({
                level: 'CRITICAL',
                title: 'Critical Security Vulnerabilities Detected',
                message: `${summary.criticalIssues} critical security issues found`,
                details: summary,
                action: 'IMMEDIATE_ATTENTION_REQUIRED'
            });
        }

        // High priority alerts
        if (summary.highIssues >= this.config.highThreshold) {
            alerts.push({
                level: 'HIGH',
                title: 'High Priority Security Issues',
                message: `${summary.highIssues} high-priority security issues found`,
                details: summary,
                action: 'REVIEW_WITHIN_24H'
            });
        }

        // License compliance alerts
        if (summary.licenseIssues > 0) {
            alerts.push({
                level: 'MEDIUM',
                title: 'License Compliance Issues',
                message: `${summary.licenseIssues} license compliance issues detected`,
                details: summary,
                action: 'REVIEW_LICENSES'
            });
        }

        return alerts;
    }

    async sendAlert(alert) {
        console.log(`📤 Sending ${alert.level} alert: ${alert.title}`);

        // Send webhook alert
        if (this.config.webhookUrl) {
            await this.sendWebhookAlert(alert);
        }

        // Send email alert (would integrate with email service)
        if (this.config.emailTo) {
            await this.sendEmailAlert(alert);
        }

        // Log alert
        this.logAlert(alert);
    }

    async sendWebhookAlert(alert) {
        try {
            const payload = {
                timestamp: new Date().toISOString(),
                alert: alert,
                source: 'Rust AI IDE Security Scanner'
            };

            console.log(`🌐 Sending webhook alert to ${this.config.webhookUrl}`);
            // In a real implementation, this would make an HTTP request
            console.log('Webhook payload:', JSON.stringify(payload, null, 2));
            
        } catch (error) {
            console.error('Failed to send webhook alert:', error.message);
        }
    }

    async sendEmailAlert(alert) {
        try {
            const subject = `[${alert.level}] ${alert.title}`;
            const body = `
Security Alert from Rust AI IDE

Level: ${alert.level}
Title: ${alert.title}
Message: ${alert.message}
Action Required: ${alert.action}

Details: ${JSON.stringify(alert.details, null, 2)}

Generated: ${new Date().toISOString()}
            `.trim();

            console.log(`📧 Sending email alert to ${this.config.emailTo}`);
            console.log('Subject:', subject);
            console.log('Body:', body);
            
        } catch (error) {
            console.error('Failed to send email alert:', error.message);
        }
    }

    logAlert(alert) {
        const logEntry = {
            timestamp: new Date().toISOString(),
            level: alert.level,
            title: alert.title,
            message: alert.message,
            action: alert.action
        };

        const logPath = path.join(this.outputDir, 'security-alerts.log');
        const logLine = JSON.stringify(logEntry) + '\n';
        
        fs.appendFileSync(logPath, logLine);
        console.log(`📝 Alert logged to ${logPath}`);
    }
}

if (require.main === module) {
    const alertSystem = new SecurityAlertSystem();
    alertSystem.sendAlerts();
}

module.exports = SecurityAlertSystem;
