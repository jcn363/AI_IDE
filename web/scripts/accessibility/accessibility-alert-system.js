#!/usr/bin/env node

const fs = require('fs');
const path = require('path');

class AccessibilityAlertSystem {
  constructor() {
    this.reportsDir = path.join(__dirname, '../../accessibility-reports');
    this.alertsFile = path.join(this.reportsDir, 'accessibility-alerts.json');
    this.thresholds = {
      critical: 0,
      warning: 5,
      compliance: 85
    };
  }

  checkForAlerts(report) {
    const alerts = [];

    // Check for critical accessibility issues
    if (report.summary.criticalIssues > this.thresholds.critical) {
      alerts.push({
        level: 'critical',
        type: 'critical_issues',
        message: `${report.summary.criticalIssues} critical accessibility violations found`,
        details: `Critical issues exceed threshold of ${this.thresholds.critical}`,
        timestamp: new Date().toISOString()
      });
    }

    // Check for low compliance score
    if (report.summary.overallScore < this.thresholds.compliance) {
      alerts.push({
        level: 'warning',
        type: 'compliance_failure',
        message: `Accessibility compliance score is ${report.summary.overallScore}% (below ${this.thresholds.compliance}% threshold)`,
        details: 'WCAG AA compliance not met',
        timestamp: new Date().toISOString()
      });
    }

    // Check for tool failures
    Object.entries(report.tools).forEach(([tool, data]) => {
      if (data.status !== 'completed') {
        alerts.push({
          level: 'warning',
          type: 'tool_failure',
          message: `${tool.toUpperCase()} accessibility test failed or was not completed`,
          details: `Status: ${data.status}`,
          timestamp: new Date().toISOString()
        });
      }
    });

    // Check for specific WCAG violations
    if (report.tools.axe && report.tools.axe.violations) {
      const colorContrastViolations = report.tools.axe.violations.filter(v => v.id === 'color-contrast');
      if (colorContrastViolations.length > 0) {
        alerts.push({
          level: 'warning',
          type: 'color_contrast',
          message: `${colorContrastViolations.length} color contrast violations found`,
          details: 'Color contrast does not meet WCAG AA standards',
          timestamp: new Date().toISOString()
        });
      }
    }

    return alerts;
  }

  saveAlerts(alerts) {
    let existingAlerts = [];
    try {
      if (fs.existsSync(this.alertsFile)) {
        existingAlerts = JSON.parse(fs.readFileSync(this.alertsFile, 'utf8'));
      }
    } catch (error) {
      console.warn('Warning: Could not read existing alerts file:', error.message);
    }

    const allAlerts = [...existingAlerts, ...alerts];

    // Keep only last 100 alerts
    const recentAlerts = allAlerts.slice(-100);

    fs.writeFileSync(this.alertsFile, JSON.stringify(recentAlerts, null, 2));
    console.log(`💾 Saved ${alerts.length} new alerts to ${this.alertsFile}`);
  }

  sendNotifications(alerts) {
    if (alerts.length === 0) {
      console.log('✅ No accessibility alerts to send');
      return;
    }

    console.log('🚨 Sending accessibility alerts...');

    // Group alerts by level
    const criticalAlerts = alerts.filter(a => a.level === 'critical');
    const warningAlerts = alerts.filter(a => a.level === 'warning');

    if (criticalAlerts.length > 0) {
      console.log(`🚨 CRITICAL ALERTS (${criticalAlerts.length}):`);
      criticalAlerts.forEach(alert => {
        console.log(`   • ${alert.message}`);
        console.log(`     ${alert.details}`);
      });
    }

    if (warningAlerts.length > 0) {
      console.log(`⚠️  WARNING ALERTS (${warningAlerts.length}):`);
      warningAlerts.forEach(alert => {
        console.log(`   • ${alert.message}`);
        console.log(`     ${alert.details}`);
      });
    }

    // In a real implementation, you would send emails, Slack notifications, etc.
    // For now, we'll just log to console and save to file
    this.logAlertsToFile(alerts);
  }

  logAlertsToFile(alerts) {
    const logFile = path.join(this.reportsDir, 'accessibility-alerts.log');
    const timestamp = new Date().toISOString();
    let logContent = `\n=== ACCESSIBILITY ALERTS - ${timestamp} ===\n`;

    alerts.forEach(alert => {
      logContent += `[${alert.level.toUpperCase()}] ${alert.message}\n`;
      logContent += `Details: ${alert.details}\n`;
      logContent += `Type: ${alert.type}\n`;
      logContent += `Time: ${alert.timestamp}\n\n`;
    });

    fs.appendFileSync(logFile, logContent);
    console.log(`📝 Alerts logged to ${logFile}`);
  }

  generateAlertSummary() {
    const summary = {
      totalAlerts: 0,
      criticalAlerts: 0,
      warningAlerts: 0,
      recentAlerts: [],
      lastUpdated: new Date().toISOString()
    };

    try {
      if (fs.existsSync(this.alertsFile)) {
        const alerts = JSON.parse(fs.readFileSync(this.alertsFile, 'utf8'));
        summary.totalAlerts = alerts.length;
        summary.criticalAlerts = alerts.filter(a => a.level === 'critical').length;
        summary.warningAlerts = alerts.filter(a => a.level === 'warning').length;

        // Get last 10 alerts
        summary.recentAlerts = alerts.slice(-10).reverse();
      }
    } catch (error) {
      console.warn('Warning: Could not generate alert summary:', error.message);
    }

    return summary;
  }

  processReport(reportPath) {
    console.log('🔍 Processing accessibility report for alerts...');

    let report;
    try {
      const reportContent = fs.readFileSync(reportPath, 'utf8');
      report = JSON.parse(reportContent);
    } catch (error) {
      console.error('❌ Error reading report file:', error.message);
      return false;
    }

    const alerts = this.checkForAlerts(report);

    if (alerts.length > 0) {
      this.saveAlerts(alerts);
      this.sendNotifications(alerts);
    } else {
      console.log('✅ No accessibility alerts generated');
    }

    return true;
  }

  // Main execution method
  run() {
    const reportPattern = path.join(this.reportsDir, 'generated', 'accessibility-report-*.json');
    const reportFiles = this.findLatestReport(reportPattern);

    if (reportFiles.length === 0) {
      console.log('❌ No accessibility reports found');
      return false;
    }

    const latestReport = reportFiles[0];
    console.log(`📋 Processing latest report: ${latestReport}`);

    return this.processReport(latestReport);
  }

  findLatestReport(pattern) {
    try {
      const dir = path.dirname(pattern);
      const basePattern = path.basename(pattern).replace('*', '');

      if (!fs.existsSync(dir)) {
        return [];
      }

      const files = fs.readdirSync(dir)
        .filter(file => file.startsWith(basePattern))
        .map(file => ({
          name: file,
          path: path.join(dir, file),
          mtime: fs.statSync(path.join(dir, file)).mtime
        }))
        .sort((a, b) => b.mtime - a.mtime);

      return files.map(f => f.path);
    } catch (error) {
      console.warn('Warning: Could not find reports:', error.message);
      return [];
    }
  }
}

// Command line interface
if (require.main === module) {
  const alertSystem = new AccessibilityAlertSystem();

  // Check if a specific report file was provided
  const reportPath = process.argv[2];
  if (reportPath) {
    alertSystem.processReport(reportPath);
  } else {
    alertSystem.run();
  }

  // Always generate summary
  const summary = alertSystem.generateAlertSummary();
  console.log('\n📊 ALERT SUMMARY:');
  console.log(`   Total Alerts: ${summary.totalAlerts}`);
  console.log(`   Critical: ${summary.criticalAlerts}`);
  console.log(`   Warnings: ${summary.warningAlerts}`);
}

module.exports = AccessibilityAlertSystem;