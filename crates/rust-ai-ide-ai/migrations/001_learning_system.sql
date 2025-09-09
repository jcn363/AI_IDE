-- Learning System Database Schema Migration
-- Version: 001
-- Description: Initial schema for AI learning system to store error patterns, fixes, and user preferences

-- Error Patterns Table
-- Stores unique error patterns with their characteristics and metadata
CREATE TABLE error_patterns (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    pattern_hash TEXT NOT NULL UNIQUE,
    error_code TEXT,
    error_message TEXT NOT NULL,
    error_type TEXT NOT NULL,
    file_extension TEXT,
    context_before TEXT,
    context_after TEXT,
    ast_pattern TEXT,
    severity TEXT NOT NULL DEFAULT 'medium',
    category TEXT NOT NULL,
    first_seen DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    last_seen DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    occurrence_count INTEGER NOT NULL DEFAULT 1,
    metadata TEXT, -- JSON blob for additional pattern data
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Successful Fixes Table
-- Stores applied fixes with success/failure tracking and confidence scoring
CREATE TABLE successful_fixes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    pattern_id INTEGER NOT NULL,
    fix_description TEXT NOT NULL,
    fix_code TEXT NOT NULL,
    fix_type TEXT NOT NULL, -- 'automatic', 'suggested', 'manual'
    application_method TEXT NOT NULL, -- 'replace', 'insert', 'delete', 'refactor'
    success_count INTEGER NOT NULL DEFAULT 0,
    failure_count INTEGER NOT NULL DEFAULT 0,
    confidence_score REAL NOT NULL DEFAULT 0.5,
    user_rating INTEGER, -- 1-5 star rating from user
    time_to_apply INTEGER, -- milliseconds to apply fix
    lines_changed INTEGER,
    project_context TEXT, -- JSON blob for project-specific context
    compiler_version TEXT,
    rust_edition TEXT,
    applied_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (pattern_id) REFERENCES error_patterns(id) ON DELETE CASCADE
);

-- User Preferences Table
-- Stores user learning preferences and privacy settings
CREATE TABLE user_preferences (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id TEXT NOT NULL UNIQUE, -- Anonymous user identifier
    learning_enabled BOOLEAN NOT NULL DEFAULT TRUE,
    data_collection_enabled BOOLEAN NOT NULL DEFAULT TRUE,
    share_anonymous_data BOOLEAN NOT NULL DEFAULT FALSE,
    auto_apply_high_confidence BOOLEAN NOT NULL DEFAULT FALSE,
    min_confidence_threshold REAL NOT NULL DEFAULT 0.7,
    preferred_fix_types TEXT, -- JSON array of preferred fix types
    excluded_error_types TEXT, -- JSON array of error types to exclude from learning
    max_suggestions_per_error INTEGER NOT NULL DEFAULT 3,
    enable_real_time_learning BOOLEAN NOT NULL DEFAULT TRUE,
    privacy_level TEXT NOT NULL DEFAULT 'standard', -- 'minimal', 'standard', 'full'
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Pattern Similarity Table
-- Stores pattern similarity scores for efficient matching
CREATE TABLE pattern_similarity (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    pattern_id_1 INTEGER NOT NULL,
    pattern_id_2 INTEGER NOT NULL,
    similarity_score REAL NOT NULL,
    similarity_type TEXT NOT NULL, -- 'lexical', 'semantic', 'structural', 'contextual'
    calculation_method TEXT NOT NULL,
    calculated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (pattern_id_1) REFERENCES error_patterns(id) ON DELETE CASCADE,
    FOREIGN KEY (pattern_id_2) REFERENCES error_patterns(id) ON DELETE CASCADE,
    UNIQUE(pattern_id_1, pattern_id_2, similarity_type)
);

-- Analytics Table
-- Stores usage analytics for learning system effectiveness
CREATE TABLE analytics (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    event_type TEXT NOT NULL, -- 'fix_applied', 'fix_rejected', 'pattern_learned', 'suggestion_shown'
    pattern_id INTEGER,
    fix_id INTEGER,
    user_id TEXT,
    success BOOLEAN,
    confidence_score REAL,
    time_taken INTEGER, -- milliseconds
    error_category TEXT,
    fix_category TEXT,
    session_id TEXT,
    project_hash TEXT, -- Anonymous project identifier
    rust_version TEXT,
    ide_version TEXT,
    additional_data TEXT, -- JSON blob for extra analytics data
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (pattern_id) REFERENCES error_patterns(id) ON DELETE SET NULL,
    FOREIGN KEY (fix_id) REFERENCES successful_fixes(id) ON DELETE SET NULL
);

-- Indexes for efficient querying

-- Error Patterns indexes
CREATE INDEX idx_error_patterns_hash ON error_patterns(pattern_hash);
CREATE INDEX idx_error_patterns_code ON error_patterns(error_code);
CREATE INDEX idx_error_patterns_type ON error_patterns(error_type);
CREATE INDEX idx_error_patterns_category ON error_patterns(category);
CREATE INDEX idx_error_patterns_severity ON error_patterns(severity);
CREATE INDEX idx_error_patterns_last_seen ON error_patterns(last_seen);
CREATE INDEX idx_error_patterns_occurrence ON error_patterns(occurrence_count DESC);

-- Successful Fixes indexes
CREATE INDEX idx_successful_fixes_pattern ON successful_fixes(pattern_id);
CREATE INDEX idx_successful_fixes_confidence ON successful_fixes(confidence_score DESC);
CREATE INDEX idx_successful_fixes_success_rate ON successful_fixes(success_count, failure_count);
CREATE INDEX idx_successful_fixes_type ON successful_fixes(fix_type);
CREATE INDEX idx_successful_fixes_applied_at ON successful_fixes(applied_at);
CREATE INDEX idx_successful_fixes_rating ON successful_fixes(user_rating DESC);

-- User Preferences indexes
CREATE INDEX idx_user_preferences_user_id ON user_preferences(user_id);
CREATE INDEX idx_user_preferences_learning ON user_preferences(learning_enabled);

-- Pattern Similarity indexes
CREATE INDEX idx_pattern_similarity_pattern1 ON pattern_similarity(pattern_id_1);
CREATE INDEX idx_pattern_similarity_pattern2 ON pattern_similarity(pattern_id_2);
CREATE INDEX idx_pattern_similarity_score ON pattern_similarity(similarity_score DESC);
CREATE INDEX idx_pattern_similarity_type ON pattern_similarity(similarity_type);

-- Analytics indexes
CREATE INDEX idx_analytics_event_type ON analytics(event_type);
CREATE INDEX idx_analytics_pattern_id ON analytics(pattern_id);
CREATE INDEX idx_analytics_fix_id ON analytics(fix_id);
CREATE INDEX idx_analytics_user_id ON analytics(user_id);
CREATE INDEX idx_analytics_created_at ON analytics(created_at);
CREATE INDEX idx_analytics_success ON analytics(success);
CREATE INDEX idx_analytics_session ON analytics(session_id);

-- Composite indexes for common queries
CREATE INDEX idx_error_patterns_type_severity ON error_patterns(error_type, severity);
CREATE INDEX idx_successful_fixes_pattern_confidence ON successful_fixes(pattern_id, confidence_score DESC);
CREATE INDEX idx_analytics_event_success ON analytics(event_type, success);
CREATE INDEX idx_pattern_similarity_patterns ON pattern_similarity(pattern_id_1, pattern_id_2);

-- Triggers for maintaining data integrity and timestamps

-- Update timestamp trigger for error_patterns
CREATE TRIGGER update_error_patterns_timestamp 
    AFTER UPDATE ON error_patterns
    FOR EACH ROW
BEGIN
    UPDATE error_patterns 
    SET updated_at = CURRENT_TIMESTAMP,
        last_seen = CURRENT_TIMESTAMP,
        occurrence_count = occurrence_count + 1
    WHERE id = NEW.id;
END;

-- Update timestamp trigger for successful_fixes
CREATE TRIGGER update_successful_fixes_timestamp 
    AFTER UPDATE ON successful_fixes
    FOR EACH ROW
BEGIN
    UPDATE successful_fixes 
    SET updated_at = CURRENT_TIMESTAMP
    WHERE id = NEW.id;
END;

-- Update timestamp trigger for user_preferences
CREATE TRIGGER update_user_preferences_timestamp 
    AFTER UPDATE ON user_preferences
    FOR EACH ROW
BEGIN
    UPDATE user_preferences 
    SET updated_at = CURRENT_TIMESTAMP
    WHERE id = NEW.id;
END;

-- Trigger to update confidence score based on success/failure ratio
CREATE TRIGGER update_fix_confidence_score
    AFTER UPDATE OF success_count, failure_count ON successful_fixes
    FOR EACH ROW
BEGIN
    UPDATE successful_fixes
    SET confidence_score = CASE 
        WHEN (NEW.success_count + NEW.failure_count) = 0 THEN 0.5
        ELSE CAST(NEW.success_count AS REAL) / (NEW.success_count + NEW.failure_count)
    END
    WHERE id = NEW.id;
END;

-- Views for common queries

-- High confidence fixes view
CREATE VIEW high_confidence_fixes AS
SELECT 
    ep.pattern_hash,
    ep.error_code,
    ep.error_message,
    sf.fix_description,
    sf.fix_code,
    sf.confidence_score,
    sf.success_count,
    sf.failure_count,
    sf.user_rating
FROM error_patterns ep
JOIN successful_fixes sf ON ep.id = sf.pattern_id
WHERE sf.confidence_score >= 0.8
ORDER BY sf.confidence_score DESC, sf.success_count DESC;

-- Learning effectiveness view
CREATE VIEW learning_effectiveness AS
SELECT 
    ep.error_type,
    ep.category,
    COUNT(sf.id) as total_fixes,
    AVG(sf.confidence_score) as avg_confidence,
    SUM(sf.success_count) as total_successes,
    SUM(sf.failure_count) as total_failures,
    CAST(SUM(sf.success_count) AS REAL) / NULLIF(SUM(sf.success_count + sf.failure_count), 0) as success_rate
FROM error_patterns ep
LEFT JOIN successful_fixes sf ON ep.id = sf.pattern_id
GROUP BY ep.error_type, ep.category
ORDER BY success_rate DESC, avg_confidence DESC;

-- User activity summary view
CREATE VIEW user_activity_summary AS
SELECT 
    a.user_id,
    COUNT(CASE WHEN a.event_type = 'fix_applied' THEN 1 END) as fixes_applied,
    COUNT(CASE WHEN a.event_type = 'fix_rejected' THEN 1 END) as fixes_rejected,
    COUNT(CASE WHEN a.event_type = 'suggestion_shown' THEN 1 END) as suggestions_shown,
    AVG(CASE WHEN a.event_type = 'fix_applied' THEN a.confidence_score END) as avg_applied_confidence,
    MIN(a.created_at) as first_activity,
    MAX(a.created_at) as last_activity
FROM analytics a
WHERE a.user_id IS NOT NULL
GROUP BY a.user_id;

-- Data retention and cleanup procedures (as comments for manual execution)

-- Clean up old analytics data (older than 1 year)
-- DELETE FROM analytics WHERE created_at < datetime('now', '-1 year');

-- Clean up unused error patterns (no successful fixes and not seen in 6 months)
-- DELETE FROM error_patterns 
-- WHERE id NOT IN (SELECT DISTINCT pattern_id FROM successful_fixes WHERE pattern_id IS NOT NULL)
-- AND last_seen < datetime('now', '-6 months');

-- Clean up low-confidence fixes with no recent success
-- DELETE FROM successful_fixes 
-- WHERE confidence_score < 0.3 
-- AND success_count = 0 
-- AND created_at < datetime('now', '-3 months');