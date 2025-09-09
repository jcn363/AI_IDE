const fs = require('fs');
const path = require('path');

/**
 * Sanitizes a field value for safe insertion into a Markdown table.
 * Removes pipes, newlines, and backticks, and defaults to empty string.
 * @param {any} value - The value to sanitize
 * @returns {string} - The sanitized string
 */
function sanitizeField(value) {
    return (value || '').toString().replace(/[|\n`]/g, '').trim();
}

/**
 * Reads and parses a JSON file containing crates data.
 * @param {string} filePath - Path to the JSON file
 * @returns {object} - Parsed JSON data
 */
function readJson(filePath) {
    try {
        const fileContent = fs.readFileSync(filePath, 'utf8');
        return JSON.parse(fileContent);
    } catch (error) {
        console.error(`Error reading or parsing JSON file '${filePath}': ${error.message}`);
        throw error; // Re-throw to let caller handle
    }
}

/**
 * Generates a Markdown table from crates data using array accumulation for efficiency.
 * @param {array} crates - Array of crate objects
 * @returns {string} - Markdown table string
 */
function generateTable(crates) {
    const rows = [];
    rows.push('| Crate                  | Description                             | Status        |');
    rows.push('| ---------------------- | --------------------------------------- | ------------- |');

    crates.forEach(crate => {
        rows.push(`| \`${sanitizeField(crate.name)}\` | ${sanitizeField(crate.description)} | ${sanitizeField(crate.status)} |\n`);
    });

    return rows.join('\n');
}

/**
 * Updates the Markdown file with the generated crates table using atomic writes.
 * @param {string} inputFile - Path to the crates JSON file
 * @param {string} outputFile - Path to the Markdown file to update
 */
function updateMarkdown(inputFile, outputFile) {
    try {
        const data = readJson(inputFile);

        // Validate data.crates is an array
        if (!data || !Array.isArray(data.crates)) {
            console.error('Invalid data: data.crates must be an array');
            console.error('Please check your JSON file format');
            process.exit(1);
        }

        const table = generateTable(data.crates);

        const startMarker = '<!-- AUTO-GENERATED: DO NOT EDIT DIRECTLY -->';
        const endMarker = '<!-- END AUTO-GENERATED -->';

        let content;
        if (fs.existsSync(outputFile)) {
            content = fs.readFileSync(outputFile, 'utf8');
        } else {
            console.log(`Output file '${outputFile}' does not exist. Creating it with auto-generated markers.`);
            content = startMarker + '\n' + endMarker + '\n';
            // Write the initial file
            fs.writeFileSync(outputFile, content);
            content = fs.readFileSync(outputFile, 'utf8');
        }

        const startIdx = content.indexOf(startMarker);
        const endIdx = content.indexOf(endMarker, startIdx);

        if (startIdx === -1 || endIdx === -1 || startIdx >= endIdx) {
            console.error('Auto-generated markers not found or invalid in markdown file');
            console.error('Check that the file contains the required comments for auto-generation');
            process.exit(1);
        }

        const beforeStart = content.substring(0, startIdx);
        const afterEnd = content.substring(endIdx + endMarker.length);

        const newContent = beforeStart + startMarker + '\n' + table + '\n' + endMarker + afterEnd;

        // Atomic write: write to temp file then rename
        const tempFile = outputFile + '.tmp';
        fs.writeFileSync(tempFile, newContent);
        fs.renameSync(tempFile, outputFile);

        console.log('Crates table updated successfully!');
    } catch (error) {
        console.error(`Error updating markdown file: ${error.message}`);
        process.exit(1);
    }
}

if (require.main === module) {
    const args = process.argv.slice(2);

    if (args.length === 1 && args[0] === '--help') {
        console.log('Usage: node generate_crates_table.js [input.json] [output.md]');
        console.log('Defaults: crates.json RUST_AI_IDE_PLAN.md');
        process.exit(0);
    }

    const inputFile = args[0] || 'crates.json';
    const outputFile = args[1] || 'RUST_AI_IDE_PLAN.md';

    updateMarkdown(inputFile, outputFile);
}