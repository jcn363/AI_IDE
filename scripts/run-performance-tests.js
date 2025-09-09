#!/usr/bin/env node

//! Performance Testing Script (Consolidated)
//!
//! This script consolidates performance testing functionality from:
//! - test-performance-analyzer/
//! - test-performance-project/
//! - Existing performance test scripts
//!
//! Usage: node scripts/run-performance-tests.js [options]

const fs = require('fs').promises;
const path = require('path');
const { execSync, spawn } = require('child_process');

class PerformanceTestRunner {
    constructor(options = {}) {
        this.options = {
            projectPath: options.projectPath || 'test-performance-project',
            iterations: options.iterations || 10000,
            profile: options.profile || 'debug',
            enableIncremental: options.enableIncremental !== false,
            outputDir: options.outputDir || 'performance-results',
            outputFormat: options.outputFormat || 'json', // json, markdown, console
            compareProfiles: options.compareProfiles || false,
            enableProfiling: options.enableProfiling || false,
            ...options
        };

        this.results = [];
        this.startTime = Date.now();
    }

    async run() {
        console.log('🚀 Starting Performance Test Suite');
        console.log('==================================');

        try {
            // Validate project exists
            await this.validateProject();

            // Run build performance tests
            await this.runBuildTests();

            // Run workload performance tests
            await this.runWorkloadTests();

            // Generate reports
            await this.generateReports();

            console.log('✅ Performance tests completed successfully');
            return this.results;

        } catch (error) {
            console.error('❌ Performance tests failed:', error.message);
            throw error;
        }
    }

    async validateProject() {
        console.log('📂 Validating project structure...');

        const projectPath = this.options.projectPath;
        const absolutePath = path.resolve(projectPath);

        if (!await this.exists(absolutePath)) {
            console.log(`⚠️  Project not found at ${absolutePath}. Creating minimal test project...`);
            await this.createTestProject(absolutePath);
        }

        const cargoToml = path.join(absolutePath, 'Cargo.toml');
        const srcDir = path.join(absolutePath, 'src');
        const libRs = path.join(srcDir, 'lib.rs');

        if (!await this.exists(cargoToml)) {
            throw new Error(`Cargo.toml not found in ${absolutePath}`);
        }

        if (!await this.exists(srcDir) || !await this.exists(libRs)) {
            throw new Error(`src/lib.rs not found in ${absolutePath}`);
        }

        console.log(`✅ Project validation passed for: ${absolutePath}`);
    }

    async createTestProject(projectPath) {
        const srcDir = path.join(projectPath, 'src');

        // Create directories
        await fs.mkdir(srcDir, { recursive: true });

        // Create Cargo.toml
        const cargoToml = `[package]
name = "performance-test-project"
version = "0.1.0"
edition = "2021"

[dependencies]
tokio = { version = "1", features = ["full"] }`;

        await fs.writeFile(path.join(projectPath, 'Cargo.toml'), cargoToml);

        // Create src/lib.rs
        const libRs = `//! Performance testing library

/// Simulate CPU-bound work
pub fn run_sync_workload(iterations: u32) -> u64 {
    let mut result = 0u64;

    for i in 0..iterations {
        let x = i as u64 * 2654435761 % (1 << 31);
        result = result.wrapping_add(x);

        let mut vec = Vec::with_capacity(1000);
        for j in 0..1000 {
            vec.push(j as u64);
        }

        result = result.wrapping_add(vec[vec.len() - 1]);
    }

    result
}

/// Simulate I/O-bound work
pub async fn run_async_workload(iterations: u32) -> u64 {
    let mut result = 0u64;

    for i in 0..iterations {
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        let x = i as u64 * 11400714819323198549u64;
        result = result.wrapping_add(x);
    }

    result
}`;

        await fs.writeFile(path.join(srcDir, 'lib.rs'), libRs);

        console.log(`✅ Created minimal test project at: ${projectPath}`);
    }

    async runBuildTests() {
        console.log('🔨 Running build performance tests...');

        const projectPath = this.options.projectPath;

        // Clean first
        console.log('   📦 Cleaning project...');
        execSync('cargo clean', {
            cwd: projectPath,
            stdio: this.options.verbose ? 'inherit' : 'pipe'
        });

        // Test debug build
        console.log('   📦 Building debug profile...');
        const debugStart = Date.now();
        execSync(`cargo build ${this.options.enableIncremental ? '--incremental' : ''}`, {
            cwd: projectPath,
            stdio: this.options.verbose ? 'inherit' : 'pipe'
        });
        const debugTime = Date.now() - debugStart;

        // Test incremental if enabled
        let incrementalTime = null;
        if (this.options.enableIncremental) {
            console.log('   📦 Building incrementally...');
            const incStart = Date.now();
            execSync('cargo build --incremental', {
                cwd: projectPath,
                stdio: 'pipe'
            });
            incrementalTime = Date.now() - incStart;
        }

        // Test release build if requested
        let releaseTime = null;
        if (this.options.compareProfiles || this.options.profile === 'release') {
            console.log('   📦 Building release profile...');
            execSync('cargo clean', {
                cwd: projectPath,
                stdio: 'pipe'
            });

            const releaseStart = Date.now();
            execSync('cargo build --release', {
                cwd: projectPath,
                stdio: 'pipe'
            });
            releaseTime = Date.now() - releaseStart;
        }

        const buildResult = {
            test: 'build_performance',
            debugBuildTimeMs: debugTime,
            incrementalBuildTimeMs: incrementalTime,
            releaseBuildTimeMs: releaseTime,
            profile: this.options.profile,
            incrementalEnabled: this.options.enableIncremental,
            timestamp: new Date().toISOString()
        };

        this.results.push(buildResult);
        this.logBuildResults(buildResult);
    }

    async runWorkloadTests() {
        console.log('⚡ Running workload performance tests...');

        const projectPath = this.options.projectPath;
        const tempDir = path.join(projectPath, 'target', 'perf');

        await fs.mkdir(tempDir, { recursive: true });

        // Build executable if needed
        execSync(`cargo build ${this.options.profile === 'release' ? '--release' : ''}`, {
            cwd: projectPath,
            stdio: 'pipe'
        });

        const exeName = process.platform === 'win32' ? 'performance_test_project.exe' : 'performance_test_project';
        const exePath = path.join(projectPath, 'target', this.options.profile, exeName);

        if (!await this.exists(exePath)) {
            throw new Error(`Executable not found: ${exePath}`);
        }

        console.log('   📊 Running synchronous workload test...');
        const syncStart = Date.now();
        const syncOutput = await this.runExecutable(exePath, ['--sync', this.options.iterations.toString()]);
        const syncTime = Date.now() - syncStart;
        const syncOpsPerSec = this.options.iterations / (syncTime / 1000);

        console.log('   📊 Running asynchronous workload test...');
        const asyncStart = Date.now();
        const asyncOutput = await this.runExecutable(exePath, ['--async', (this.options.iterations / 10).toString()]);
        const asyncTime = Date.now() - asyncStart;
        const asyncOpsPerSec = (this.options.iterations / 10) / (asyncTime / 1000);

        const workloadResult = {
            test: 'workload_performance',
            syncIterations: this.options.iterations,
            syncTimeMs: syncTime,
            syncOpsPerSecond: syncOpsPerSec,
            asyncIterations: this.options.iterations / 10,
            asyncTimeMs: asyncTime,
            asyncOpsPerSecond: asyncOpsPerSec,
            profile: this.options.profile,
            timestamp: new Date().toISOString()
        };

        this.results.push(workloadResult);
        this.logWorkloadResults(workloadResult);
    }

    async runExecutable(executable, args = []) {
        return new Promise((resolve, reject) => {
            const child = spawn(executable, args, {
                stdio: this.options.verbose ? 'inherit' : 'pipe'
            });

            let stdout = '';
            let stderr = '';

            if (child.stdout) {
                child.stdout.on('data', (data) => stdout += data.toString());
            }

            if (child.stderr) {
                child.stderr.on('data', (data) => stderr += data.toString());
            }

            child.on('close', (code) => {
                if (code === 0) {
                    resolve(stdout);
                } else {
                    reject(new Error(`Process exited with code ${code}: ${stderr}`));
                }
            });

            child.on('error', reject);

            // Timeout after 5 minutes
            setTimeout(() => {
                child.kill();
                reject(new Error('Process timeout'));
            }, 5 * 60 * 1000);
        });
    }

    logBuildResults(result) {
        console.log(`   ⏱️  Debug build time: ${result.debugBuildTimeMs}ms`);

        if (result.incrementalBuildTimeMs !== null) {
            console.log(`   ⏱️  Incremental build time: ${result.incrementalBuildTimeMs}ms`);
            const improvement = ((result.debugBuildTimeMs - result.incrementalBuildTimeMs) / result.debugBuildTimeMs * 100);
            console.log(`   📈 Incremental improvement: ${improvement.toFixed(1)}%`);
        }

        if (result.releaseBuildTimeMs !== null) {
            console.log(`   ⏱️  Release build time: ${result.releaseBuildTimeMs}ms`);
        }
    }

    logWorkloadResults(result) {
        console.log(`   🚀 Sync performance: ${result.syncOpsPerSecond.toFixed(2)} ops/second`);
        console.log(`   🚀 Async performance: ${result.asyncOpsPerSecond.toFixed(2)} ops/second`);

        if (result.asyncOpsPerSecond > result.syncOpsPerSecond) {
            const improvement = ((result.asyncOpsPerSecond - result.syncOpsPerSecond) / result.syncOpsPerSecond * 100);
            console.log(`   📈 Async improvement: ${improvement.toFixed(1)}%`);
        }
    }

    async generateReports() {
        console.log('📝 Generating performance reports...');

        await fs.mkdir(this.options.outputDir, { recursive: true });

        const timestamp = new Date().toISOString().replace(/[:.]/g, '-');
        const baseName = `perf-test-${this.options.profile}-${timestamp}`;

        // Generate JSON report
        const jsonFile = path.join(this.options.outputDir, `${baseName}.json`);
        await fs.writeFile(jsonFile, JSON.stringify(this.results, null, 2));
        console.log(`   📄 JSON report: ${jsonFile}`);

        // Generate Markdown report
        const mdReport = this.generateMarkdownReport();
        const mdFile = path.join(this.options.outputDir, `${baseName}.md`);
        await fs.writeFile(mdFile, mdReport);
        console.log(`   📄 Markdown report: ${mdFile}`);

        // Generate console report
        console.log('\n' + mdReport);
    }

    generateMarkdownReport() {
        let report = `# Performance Test Report\n\n`;
        report += `**Generated:** ${new Date().toISOString()}\n`;
        report += `**Profile:** ${this.options.profile}\n`;
        report += `**Iterations:** ${this.options.iterations}\n\n`;

        // Summary table
        report += `## Summary\n\n`;
        report += `| Test | Duration | Performance |\n`;
        report += `|------|----------|-------------|\n`;

        for (const result of this.results) {
            if (result.test === 'build_performance') {
                report += `| Debug Build | ${result.debugBuildTimeMs}ms | ${result.incrementalBuildTimeMs ? `Inc: +${result.incrementalBuildTimeMs}ms` : '-'} |\n`;
                if (result.releaseBuildTimeMs) {
                    report += `| Release Build | ${result.releaseBuildTimeMs}ms | - |\n`;
                }
            } else if (result.test === 'workload_performance') {
                report += `| Sync Workload | ${result.syncTimeMs}ms | ${result.syncOpsPerSecond.toFixed(2)} ops/s |\n`;
                report += `| Async Workload | ${result.asyncTimeMs}ms | ${result.asyncOpsPerSecond.toFixed(2)} ops/s |\n`;
            }
        }

        // Recommendations
        report += `\n## Recommendations\n\n`;

        const [buildResult] = this.results.filter(r => r.test === 'build_performance');
        const [workloadResult] = this.results.filter(r => r.test === 'workload_performance');

        if (buildResult && buildResult.incrementalBuildTimeMs) {
            const incImprovement = ((buildResult.debugBuildTimeMs - buildResult.incrementalBuildTimeMs) / buildResult.debugBuildTimeMs * 100);
            if (incImprovement > 10) {
                report += `- 🚀 **Incremental builds** show ${incImprovement.toFixed(1)}% improvement\n`;
                report += `- Consider keeping incremental compilation enabled for development\n`;
            }
        }

        if (workloadResult && workloadResult.asyncOpsPerSecond > workloadResult.syncOpsPerSecond) {
            const asyncImprovement = ((workloadResult.asyncOpsPerSecond - workloadResult.syncOpsPerSecond) / workloadResult.syncOpsPerSecond * 100);
            report += `- ⚡ **Async patterns** show ${asyncImprovement.toFixed(1)}% improvement for I/O-bound work\n`;
            report += `- Consider using async/await in performance-critical code paths\n`;
        }

        if (buildResult && buildResult.releaseBuildTimeMs > buildResult.debugBuildTimeMs * 2) {
            report += `- 📦 **Release builds** are significantly slower than debug builds\n`;
            report += `- Consider using debug builds during development with occasional release tests\n`;
        }

        return report;
    }

    async exists(filePath) {
        try {
            await fs.access(filePath);
            return true;
        } catch {
            return false;
        }
    }
}

// CLI interface
async function main() {
    const args = process.argv.slice(2);
    const options = {};

    // Parse command-line arguments
    for (let i = 0; i < args.length; i++) {
        switch (args[i]) {
            case '--project':
                options.projectPath = args[++i];
                break;
            case '--iterations':
                options.iterations = parseInt(args[++i]);
                break;
            case '--profile':
                options.profile = args[++i];
                break;
            case '--output-dir':
                options.outputDir = args[++i];
                break;
            case '--format':
                options.outputFormat = args[++i];
                break;
            case '--compare-profiles':
                options.compareProfiles = true;
                break;
            case '--verbose':
                options.verbose = true;
                break;
            case '--help':
                showHelp();
                return;
        }
    }

    const runner = new PerformanceTestRunner(options);
    await runner.run();
}

function showHelp() {
    console.log(`
Performance Testing Script

Usage: node scripts/run-performance-tests.js [options]

Options:
    --project <path>         Path to Rust project to test (default: test-performance-project)
    --iterations <number>    Number of iterations for workload tests (default: 10000)
    --profile <profile>      Cargo build profile: debug or release (default: debug)
    --output-dir <path>      Directory for test results (default: performance-results)
    --format <format>        Output format: json, markdown, console (default: json)
    --compare-profiles       Compare debug vs release build performance
    --verbose                Enable verbose output
    --help                   Show this help message

Examples:
    # Basic performance test
    node scripts/run-performance-tests.js

    # Test with release profile and detailed output
    node scripts/run-performance-tests.js --profile release --verbose

    # Compare profiles and save results
    node scripts/run-performance-tests.js --compare-profiles --output-dir ./results
`);
    process.exit(0);
}

// Export for CommonJS and ESM compatibility
module.exports = { PerformanceTestRunner };

if (require.main === module) {
    main().catch(console.error);
}