#!/usr/bin/env node

const fs = require('fs');
const path = require('path');
const { exec } = require('child_process');
const { promisify } = require('util');
const execAsync = promisify(exec);

class AccessibilityMonitoring {
  constructor() {
    this.reportsDir = path.join(__dirname, '../../accessibility-reports');
    this.monitoringFile = path.join(this.reportsDir, 'accessibility-monitoring.json');
    this.schedules = {
      hourly: '0 * * * *',    // Every hour
      daily: '0 6 * * *',     // Daily at 6 AM
      weekly: '0 6 * * 1'     // Weekly on Monday at 6 AM
    };
    this.ensureDirectories();
  }

  ensureDirectories() {
    if (!fs.existsSync(this.reportsDir)) {
      fs.mkdirSync(this.reportsDir, { recursive: true });
    }
  }

  async runFullAccessibilityTest() {
    console.log('🔄 Running full accessibility test suite...');

    try {
      const { stdout: buildOutput, stderr: buildError } = await execAsync('cd web && npm run build');
      console.log('✅ Build completed');

      // Run accessibility tests
      const { stdout: axeOutput, stderr: axeError } = await execAsync('cd web && npm run accessibility:test:axe');
      console.log('✅ Axe tests completed');

      const { stdout: lighthouseOutput, stderr: lighthouseError } = await execAsync('cd web && npm run accessibility:test:lighthouse');
      console.log('✅ Lighthouse tests completed');

      const { stdout: pa11yOutput, stderr: pa11yError } = await execAsync('cd web && npm run accessibility:test:pa11y');
      console.log('✅ Pa11y tests completed');

      // Generate report
      const { stdout: reportOutput, stderr: reportError } = await execAsync('cd web && npm run accessibility:report');
      console.log('✅ Report generated');

      // Process alerts
      const { stdout: alertOutput, stderr: alertError } = await execAsync('cd web && npm run accessibility:alert');
      console.log('✅ Alerts processed');

      return {
        success: true,
        timestamp: new Date().toISOString(),
        output: {
          build: buildOutput,
          axe: axeOutput,
          lighthouse: lighthouseOutput,
          pa11y: pa11yOutput,
          report: reportOutput,
          alert: alertOutput
        }
      };

    } catch (error) {
      console.error('❌ Accessibility test failed:', error.message);
      return {
        success: false,
        timestamp: new Date().toISOString(),
        error: error.message,
        output: error.stdout || ''
      };
    }
  }

  async checkComplianceStatus() {
    console.log('📊 Checking accessibility compliance status...');

    const reportFiles = this.findLatestReports();
    if (reportFiles.length === 0) {
      return {
        status: 'no_reports',
        message: 'No accessibility reports found'
      };
    }

    const latestReport = reportFiles[0];
    let reportData;

    try {
      const reportContent = fs.readFileSync(latestReport, 'utf8');
      reportData = JSON.parse(reportContent);
    } catch (error) {
      return {
        status: 'error',
        message: `Failed to read report: ${error.message}`
      };
    }

    const compliance = this.evaluateCompliance(reportData);

    return {
      status: compliance.passed ? 'passed' : 'failed',
      score: reportData.summary.overallScore,
      criticalIssues: reportData.summary.criticalIssues,
      compliance: compliance,
      reportFile: latestReport,
      timestamp: new Date().toISOString()
    };
  }

  evaluateCompliance(reportData) {
    const score = reportData.summary.overallScore;
    const criticalIssues = reportData.summary.criticalIssues;

    const passed = score >= 85 && criticalIssues === 0;

    return {
      passed,
      score,
      criticalIssues,
      level: score >= 95 ? 'AAA' : score >= 85 ? 'AA' : 'A',
      recommendations: this.generateComplianceRecommendations(reportData)
    };
  }

  generateComplianceRecommendations(reportData) {
    const recommendations = [];

    if (reportData.summary.overallScore < 85) {
      recommendations.push('Improve overall accessibility score to meet WCAG AA standards (85%+)');
    }

    if (reportData.summary.criticalIssues > 0) {
      recommendations.push('Address all critical accessibility violations immediately');
    }

    Object.entries(reportData.tools).forEach(([tool, data]) => {
      if (data.status !== 'completed') {
        recommendations.push(`Ensure ${tool} accessibility tests are running successfully`);
      }
    });

    return recommendations;
  }

  findLatestReports() {
    try {
      const generatedDir = path.join(this.reportsDir, 'generated');
      if (!fs.existsSync(generatedDir)) {
        return [];
      }

      const files = fs.readdirSync(generatedDir)
        .filter(file => file.startsWith('accessibility-report-'))
        .map(file => ({
          name: file,
          path: path.join(generatedDir, file),
          mtime: fs.statSync(path.join(generatedDir, file)).mtime
        }))
        .sort((a, b) => b.mtime - a.mtime);

      return files.map(f => f.path);
    } catch (error) {
      console.warn('Warning: Could not find reports:', error.message);
      return [];
    }
  }

  updateMonitoringData(testResult, complianceStatus) {
    let monitoringData = this.loadMonitoringData();

    const entry = {
      timestamp: new Date().toISOString(),
      testResult,
      complianceStatus
    };

    monitoringData.entries.push(entry);

    // Keep only last 100 entries
    if (monitoringData.entries.length > 100) {
      monitoringData.entries = monitoringData.entries.slice(-100);
    }

    monitoringData.lastUpdated = new Date().toISOString();
    monitoringData.summary = this.generateMonitoringSummary(monitoringData.entries);

    this.saveMonitoringData(monitoringData);
    return monitoringData;
  }

  loadMonitoringData() {
    try {
      if (fs.existsSync(this.monitoringFile)) {
        return JSON.parse(fs.readFileSync(this.monitoringFile, 'utf8'));
      }
    } catch (error) {
      console.warn('Warning: Could not load monitoring data:', error.message);
    }

    return {
      entries: [],
      summary: {
        totalTests: 0,
        passedTests: 0,
        failedTests: 0,
        averageScore: 0,
        complianceRate: 0
      },
      lastUpdated: new Date().toISOString()
    };
  }

  saveMonitoringData(data) {
    fs.writeFileSync(this.monitoringFile, JSON.stringify(data, null, 2));
    console.log(`💾 Monitoring data saved to ${this.monitoringFile}`);
  }

  generateMonitoringSummary(entries) {
    if (entries.length === 0) {
      return {
        totalTests: 0,
        passedTests: 0,
        failedTests: 0,
        averageScore: 0,
        complianceRate: 0
      };
    }

    const totalTests = entries.length;
    const passedTests = entries.filter(e => e.complianceStatus.status === 'passed').length;
    const failedTests = totalTests - passedTests;

    const scores = entries
      .filter(e => e.complianceStatus.score !== undefined)
      .map(e => e.complianceStatus.score);

    const averageScore = scores.length > 0
      ? Math.round(scores.reduce((a, b) => a + b, 0) / scores.length)
      : 0;

    const complianceRate = Math.round((passedTests / totalTests) * 100);

    return {
      totalTests,
      passedTests,
      failedTests,
      averageScore,
      complianceRate
    };
  }

  async sendMonitoringAlert(complianceStatus) {
    if (complianceStatus.status === 'failed') {
      console.log('🚨 ACCESSIBILITY COMPLIANCE ALERT 🚨');
      console.log(`Score: ${complianceStatus.score}/100`);
      console.log(`Critical Issues: ${complianceStatus.criticalIssues}`);
      console.log('Recommendations:');
      complianceStatus.compliance.recommendations.forEach(rec => {
        console.log(`  • ${rec}`);
      });

      // In a real implementation, this would send emails, Slack notifications, etc.
      // For now, we'll just log to a file
      this.logAlert(complianceStatus);
    }
  }

  logAlert(complianceStatus) {
    const alertLog = path.join(this.reportsDir, 'accessibility-monitoring-alerts.log');
    const timestamp = new Date().toISOString();
    const logEntry = `
=== ACCESSIBILITY MONITORING ALERT - ${timestamp} ===
Status: ${complianceStatus.status.toUpperCase()}
Score: ${complianceStatus.score}/100
Critical Issues: ${complianceStatus.criticalIssues}
Compliance Level: ${complianceStatus.compliance.level}
Recommendations:
${complianceStatus.compliance.recommendations.map(r => `  • ${r}`).join('\n')}
===================================================
`;

    fs.appendFileSync(alertLog, logEntry);
    console.log(`📝 Alert logged to ${alertLog}`);
  }

  // Main monitoring execution
  async runMonitoring() {
    console.log('🔍 Starting accessibility monitoring...');

    // Run full test suite
    const testResult = await this.runFullAccessibilityTest();

    // Check compliance status
    const complianceStatus = await this.checkComplianceStatus();

    // Update monitoring data
    const monitoringData = this.updateMonitoringData(testResult, complianceStatus);

    // Send alerts if needed
    await this.sendMonitoringAlert(complianceStatus);

    // Log summary
    console.log('\n📊 MONITORING SUMMARY:');
    console.log(`   Tests Run: ${monitoringData.summary.totalTests}`);
    console.log(`   Passed: ${monitoringData.summary.passedTests}`);
    console.log(`   Failed: ${monitoringData.summary.failedTests}`);
    console.log(`   Average Score: ${monitoringData.summary.averageScore}%`);
    console.log(`   Compliance Rate: ${monitoringData.summary.complianceRate}%`);

    return {
      testResult,
      complianceStatus,
      monitoringData
    };
  }

  // Setup cron job (would be called by external scheduler)
  setupScheduledMonitoring() {
    console.log('⏰ Setting up scheduled accessibility monitoring...');

    // This would typically integrate with cron or a job scheduler
    // For demonstration, we'll just log the schedules
    console.log('Scheduled monitoring configurations:');
    Object.entries(this.schedules).forEach(([frequency, schedule]) => {
      console.log(`  ${frequency}: ${schedule}`);
    });

    return this.schedules;
  }
}

// Command line interface
if (require.main === module) {
  const monitor = new AccessibilityMonitoring();

  const command = process.argv[2];

  switch (command) {
    case 'run':
      monitor.runMonitoring().catch(console.error);
      break;
    case 'status':
      monitor.checkComplianceStatus().then(status => {
        console.log(JSON.stringify(status, null, 2));
      }).catch(console.error);
      break;
    case 'setup':
      monitor.setupScheduledMonitoring();
      break;
    default:
      console.log('Usage: node accessibility-monitoring.js [run|status|setup]');
      console.log('  run   - Run full accessibility monitoring');
      console.log('  status- Check current compliance status');
      console.log('  setup - Setup scheduled monitoring');
  }
}

module.exports = AccessibilityMonitoring;