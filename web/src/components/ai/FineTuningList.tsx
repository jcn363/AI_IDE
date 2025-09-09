import React, { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import type { FineTuneJobInfo as FineTuneJob } from '../../features/ai/types';

interface FineTuningListProps {
  jobs: FineTuneJob[];
  isLoading: boolean;
  selectedJob: FineTuneJob | null;
  onJobSelect: (job: FineTuneJob | null) => void;
  onCancelJob: (jobId: string) => void;
  onRefreshJobs: () => void;
}

export const FineTuningList: React.FC<FineTuningListProps> = ({
  jobs,
  isLoading,
  selectedJob,
  onJobSelect,
  onCancelJob,
  onRefreshJobs,
}) => {
  const [searchTerm, setSearchTerm] = useState('');
  const [sortBy, setSortBy] = useState<'created' | 'name' | 'status'>('created');

  const filteredJobs = jobs.filter(job =>
    job.name.toLowerCase().includes(searchTerm.toLowerCase())
  );

  const sortedJobs = [...filteredJobs].sort((a, b) => {
    switch (sortBy) {
      case 'name':
        return a.name.localeCompare(b.name);
      case 'status':
        return a.status.localeCompare(b.status);
      default:
        return new Date(a.createdAt).getTime() - new Date(b.createdAt).getTime();
    }
  });

  return (
    <div className="fine-tuning-list">
      <div className="list-header">
        <h3>Fine-Tuning Jobs ({jobs.length})</h3>
        <div className="controls">
          <input
            type="text"
            placeholder="Search jobs..."
            value={searchTerm}
            onChange={(e) => setSearchTerm(e.target.value)}
          />
          <select value={sortBy} onChange={(e) => setSortBy(e.target.value as typeof sortBy)}>
            <option value="created">Sort by Created</option>
            <option value="name">Sort by Name</option>
            <option value="status">Sort by Status</option>
          </select>
          <button onClick={onRefreshJobs} disabled={isLoading}>
            {isLoading ? 'Loading...' : 'Refresh'}
          </button>
        </div>
      </div>

      <div className="jobs-container">
        {sortedJobs.length === 0 ? (
          <div className="empty-state">
            <p>No jobs found</p>
          </div>
        ) : (
          sortedJobs.map((job) => (
            <div
              key={job.jobId}
              className={`job-item ${selectedJob?.jobId === job.jobId ? 'selected' : ''}`}
              onClick={() => onJobSelect(job)}
            >
              <div className="job-header">
                <h4>{job.name}</h4>
                <span className={`status status-${job.status.toLowerCase()}`}>
                  {job.status}
                </span>
              </div>

              <div className="job-details">
                <p><strong>Model:</strong> {job.modelType} - {job.baseModel}</p>
                <p><strong>Created:</strong> {new Date(job.createdAt).toLocaleString()}</p>

                {job.progress && (
                  <div className="progress">
                    <div className="progress-bar">
                      <div
                        className="progress-fill"
                        style={{
                          width: `${(job.progress.epoch / job.progress.totalEpochs) * 100}%`
                        }}
                      />
                    </div>
                    <span>
                      Epoch {job.progress.epoch}/{job.progress.totalEpochs}
                      {job.progress.loss && ` • Loss: ${job.progress.loss.toFixed(4)}`}
                    </span>
                  </div>
                )}
              </div>

              <div className="job-actions">
                <button onClick={(e) => { e.stopPropagation(); onJobSelect(job); }}>
                  View Details
                </button>
                {(job.status === 'Training' || job.status === 'Initializing') && (
                  <button
                    className="danger"
                    onClick={(e) => { e.stopPropagation(); onCancelJob(job.jobId); }}
                  >
                    Cancel
                  </button>
                )}
              </div>
            </div>
          ))
        )}
      </div>

      <style jsx>{`
        .fine-tuning-list {
          padding: 20px;
          border: 1px solid #e1e5e9;
          border-radius: 8px;
          background: #fff;
        }

        .list-header {
          display: flex;
          justify-content: space-between;
          align-items: center;
          margin-bottom: 20px;
          padding-bottom: 15px;
          border-bottom: 1px solid #e1e5e9;
        }

        .list-header h3 {
          margin: 0;
          color: #2d3748;
        }

        .controls {
          display: flex;
          gap: 10px;
          align-items: center;
        }

        .controls input,
        .controls select {
          padding: 6px 12px;
          border: 1px solid #e1e5e9;
          border-radius: 4px;
          font-size: 14px;
        }

        .controls button {
          padding: 6px 12px;
          background: #3182ce;
          color: white;
          border: none;
          border-radius: 4px;
          cursor: pointer;
        }

        .controls button:disabled {
          opacity: 0.6;
          cursor: not-allowed;
        }

        .jobs-container {
          max-height: 400px;
          overflow-y: auto;
        }

        .job-item {
          border: 1px solid #e1e5e9;
          border-radius: 6px;
          padding: 12px;
          margin-bottom: 10px;
          cursor: pointer;
          transition: border-color 0.2s;
        }

        .job-item:hover,
        .job-item.selected {
          border-color: #3182ce;
        }

        .job-header {
          display: flex;
          justify-content: space-between;
          align-items: flex-start;
          margin-bottom: 8px;
        }

        .job-header h4 {
          margin: 0;
          font-size: 16px;
          color: #2d3748;
        }

        .status {
          padding: 2px 8px;
          border-radius: 12px;
          font-size: 12px;
          text-transform: uppercase;
          font-weight: 500;
        }

        .status-completed,
        .status-training { background: #c6f6d5; color: #276749; }
        .status-failed { background: #fed7d7; color: #c53030; }
        .status-created,
        .status-initializing { background: #e2e8f0; color: #4a5568; }

        .job-details p {
          margin: 4px 0;
          font-size: 14px;
          color: #4a5568;
        }

        .progress {
          margin-top: 8px;
        }

        .progress-bar {
          height: 4px;
          background: #e1e5e9;
          border-radius: 2px;
          margin-bottom: 4px;
        }

        .progress-fill {
          height: 100%;
          background: #3182ce;
          border-radius: 2px;
          transition: width 0.3s;
        }

        .job-actions {
          display: flex;
          gap: 6px;
          margin-top: 10px;
        }

        .job-actions button {
          padding: 4px 8px;
          border: none;
          border-radius: 4px;
          cursor: pointer;
          font-size: 12px;
        }

        .job-actions button {
          background: #448aca;
          color: white;
        }

        .job-actions button.danger {
          background: #e53e3e;
        }

        .empty-state {
          text-align: center;
          padding: 40px;
          color: #718096;
        }

        .empty-state p {
          margin: 0;
          font-size: 16px;
        }
      `}</style>
    </div>
  );
};

export default FineTuningList;