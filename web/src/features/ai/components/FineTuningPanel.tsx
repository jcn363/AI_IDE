import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import type {
  FineTuneJob,
  TrainingStatus,
  TrainingProgress,
  TrainingConfigInfo,
  FineTuningRequest,
  DatasetPreparationRequest,
} from '../types';

interface FineTuningPanelProps {
  className?: string;
}

export const FineTuningPanel: React.FC<FineTuningPanelProps> = ({ className }) => {
  const [jobs, setJobs] = useState<FineTuneJob[]>([]);
  const [selectedJob, setSelectedJob] = useState<FineTuneJob | null>(null);
  const [isCreateJobOpen, setIsCreateJobOpen] = useState(false);
  const [isLoading, setIsLoading] = useState(false);

  useEffect(() => {
    loadJobs();
  }, []);

  const loadJobs = async () => {
    try {
      setIsLoading(true);
      const jobsList = await invoke<FineTuneJob[]>('list_finetune_jobs');
      setJobs(jobsList);
    } catch (error) {
      console.error('Failed to load fine-tuning jobs:', error);
    } finally {
      setIsLoading(false);
    }
  };

  const handleCreateJob = async (request: FineTuningRequest) => {
    try {
      setIsLoading(true);
      const jobId = await invoke<string>('start_finetune_job', { request });
      setIsCreateJobOpen(false);
      await loadJobs(); // Refresh the list
    } catch (error) {
      console.error('Failed to create fine-tuning job:', error);
    } finally {
      setIsLoading(false);
    }
  };

  const handleCancelJob = async (jobId: string) => {
    try {
      await invoke<void>('cancel_finetune_job', { jobId });
      await loadJobs(); // Refresh the list
    } catch (error) {
      console.error('Failed to cancel job:', error);
    }
  };

  return (
    <div className={`fine-tuning-panel ${className || ''}`}>
      <div className="panel-header">
        <h3>Fine-Tuning Management</h3>
        <button
          onClick={() => setIsCreateJobOpen(true)}
          className="primary-button"
          disabled={isLoading}
        >
          New Training Job
        </button>
      </div>

      <div className="jobs-list">
        {isLoading ? (
          <div className="loading">Loading training jobs...</div>
        ) : jobs.length === 0 ? (
          <div className="empty-state">
            <p>No fine-tuning jobs found</p>
            <button onClick={() => setIsCreateJobOpen(true)} className="secondary-button">
              Start Your First Training Job
            </button>
          </div>
        ) : (
          <div className="jobs-grid">
            {jobs.map((job) => (
              <div key={job.id} className="job-card">
                <div className="job-header">
                  <h4>{job.name}</h4>
                  <span className={`status-badge status-${job.status.toLowerCase()}`}>
                    {job.status}
                  </span>
                </div>

                <div className="job-info">
                  <div className="info-row">
                    <span className="label">Model:</span>
                    <span className="value">
                      {job.model_type} - {job.base_model}
                    </span>
                  </div>

                  {job.progress && (
                    <div className="progress-section">
                      <div className="progress-bar">
                        <div
                          className="progress-fill"
                          style={{
                            width: `${(job.progress.epoch / job.progress.total_epochs) * 100}%`,
                          }}
                        />
                      </div>
                      <div className="progress-text">
                        Epoch {job.progress.epoch}/{job.progress.total_epochs}
                        {job.progress.loss && <span> • Loss: {job.progress.loss.toFixed(4)}</span>}
                      </div>
                    </div>
                  )}

                  <div className="job-actions">
                    <button className="action-button" onClick={() => setSelectedJob(job)}>
                      View Details
                    </button>

                    {(job.status === 'Training' || job.status === 'Initializing') && (
                      <button className="cancel-button" onClick={() => handleCancelJob(job.id)}>
                        Cancel
                      </button>
                    )}
                  </div>
                </div>
              </div>
            ))}
          </div>
        )}
      </div>

      {/* Job Detail Modal */}
      {selectedJob && (
        <div className="modal-overlay" onClick={() => setSelectedJob(null)}>
          <div className="modal-content" onClick={(e) => e.stopPropagation()}>
            <div className="modal-header">
              <h3>{selectedJob.name}</h3>
              <button className="close-button" onClick={() => setSelectedJob(null)}>
                ×
              </button>
            </div>

            <div className="modal-body">
              <div className="detail-grid">
                <div className="detail-item">
                  <span className="label">Description:</span>
                  <span className="value">{selectedJob.description || 'No description'}</span>
                </div>

                <div className="detail-item">
                  <span className="label">Model Type:</span>
                  <span className="value">
                    {selectedJob.model_type} - {selectedJob.base_model}
                  </span>
                </div>

                <div className="detail-item">
                  <span className="label">Status:</span>
                  <span className="value">{selectedJob.status}</span>
                </div>

                <div className="detail-item">
                  <span className="label">Created:</span>
                  <span className="value">{new Date(selectedJob.created_at).toLocaleString()}</span>
                </div>

                {selectedJob.progress && (
                  <>
                    <div className="detail-item">
                      <span className="label">Progress:</span>
                      <span className="value">
                        Epoch {selectedJob.progress.epoch}/{selectedJob.progress.total_epochs}
                      </span>
                    </div>

                    {selectedJob.progress.loss && (
                      <div className="detail-item">
                        <span className="label">Current Loss:</span>
                        <span className="value">{selectedJob.progress.loss.toFixed(4)}</span>
                      </div>
                    )}

                    {selectedJob.progress.estimated_time_remaining && (
                      <div className="detail-item">
                        <span className="label">ETA:</span>
                        <span className="value">
                          {selectedJob.progress.estimated_time_remaining}s remaining
                        </span>
                      </div>
                    )}
                  </>
                )}

                {selectedJob.metrics && (
                  <>
                    <div className="detail-item">
                      <span className="label">Final Loss:</span>
                      <span className="value">{selectedJob.metrics.final_loss.toFixed(4)}</span>
                    </div>

                    <div className="detail-item">
                      <span className="label">Training Time:</span>
                      <span className="value">{selectedJob.metrics.training_time_seconds}s</span>
                    </div>
                  </>
                )}

                {selectedJob.error_message && (
                  <div className="detail-item error">
                    <span className="label">Error:</span>
                    <span className="value">{selectedJob.error_message}</span>
                  </div>
                )}
              </div>
            </div>
          </div>
        </div>
      )}

      {/* Create Job Modal */}
      {isCreateJobOpen && (
        <CreateFineTuneJobModal
          onSave={handleCreateJob}
          onClose={() => setIsCreateJobOpen(false)}
          isLoading={isLoading}
        />
      )}

      <style jsx>{`
        .fine-tuning-panel {
          padding: 20px;
          border: 1px solid #e1e5e9;
          border-radius: 8px;
          background: #ffffff;
          margin: 20px 0;
        }

        .panel-header {
          display: flex;
          justify-content: space-between;
          align-items: center;
          margin-bottom: 20px;
          border-bottom: 1px solid #e1e5e9;
          padding-bottom: 15px;
        }

        .panel-header h3 {
          margin: 0;
          color: #2d3748;
          font-size: 24px;
          font-weight: 600;
        }

        .primary-button {
          background: #3182ce;
          color: white;
          border: none;
          padding: 10px 20px;
          border-radius: 6px;
          cursor: pointer;
          font-weight: 500;
          transition: background-color 0.2s;
        }

        .primary-button:hover {
          background: #2c5282;
        }

        .primary-button:disabled {
          background: #cbd5e0;
          cursor: not-allowed;
        }

        .secondary-button {
          background: #e2e8f0;
          color: #4a5568;
          border: 1px solid #cbd5e0;
          padding: 8px 16px;
          border-radius: 6px;
          cursor: pointer;
          font-weight: 500;
          transition: background-color 0.2s;
        }

        .secondary-button:hover {
          background: #cbd5e0;
        }

        .empty-state {
          text-align: center;
          padding: 40px 20px;
          color: #718096;
        }

        .empty-state p {
          margin: 0 0 20px 0;
          font-size: 18px;
        }

        .jobs-grid {
          display: grid;
          gap: 16px;
        }

        .job-card {
          border: 1px solid #e1e5e9;
          border-radius: 8px;
          padding: 16px;
          background: #f7fafc;
          transition: box-shadow 0.2s;
        }

        .job-card:hover {
          box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
        }

        .job-header {
          display: flex;
          justify-content: space-between;
          align-items: flex-start;
          margin-bottom: 12px;
        }

        .job-header h4 {
          margin: 0;
          color: #2d3748;
          font-size: 16px;
          font-weight: 600;
        }

        .status-badge {
          padding: 4px 8px;
          border-radius: 4px;
          font-size: 12px;
          font-weight: 500;
          text-transform: uppercase;
        }

        .status-created {
          background: #e2e8f0;
          color: #4a5568;
        }
        .status-initializing {
          background: #fff5f5;
          color: #d69e2e;
        }
        .status-training {
          background: #c6f6d5;
          color: #276749;
        }
        .status-evaluating {
          background: #bee3f8;
          color: #2c3338;
        }
        .status-saving {
          background: #fef5e7;
          color: #7b341e;
        }
        .status-completed {
          background: #c6f6d5;
          color: #276749;
        }
        .status-failed {
          background: #fed7d7;
          color: #c53030;
        }
        .status-cancelled {
          background: #e2e8f0;
          color: #4a5568;
        }

        .job-info {
          display: flex;
          flex-direction: column;
          gap: 8px;
        }

        .info-row {
          display: flex;
          justify-content: space-between;
          align-items: center;
        }

        .info-row .label {
          font-weight: 500;
          color: #718096;
        }

        .progress-section {
          background: #f7fafc;
          padding: 12px;
          border-radius: 6px;
          border: 1px solid #e1e5e9;
        }

        .progress-bar {
          width: 100%;
          height: 8px;
          background: #e1e5e9;
          border-radius: 4px;
          margin-bottom: 8px;
        }

        .progress-fill {
          height: 100%;
          background: #3182ce;
          border-radius: 4px;
          transition: width 0.3s ease;
        }

        .progress-text {
          font-size: 14px;
          color: #4a5568;
          text-align: center;
        }

        .job-actions {
          display: flex;
          gap: 8px;
          margin-top: 12px;
        }

        .action-button {
          background: #3182ce;
          color: white;
          border: none;
          padding: 6px 12px;
          border-radius: 4px;
          cursor: pointer;
          font-size: 14px;
          transition: background-color 0.2s;
        }

        .action-button:hover {
          background: #2c5282;
        }

        .cancel-button {
          background: #e53e3e;
          color: white;
          border: none;
          padding: 6px 12px;
          border-radius: 4px;
          cursor: pointer;
          font-size: 14px;
          transition: background-color 0.2s;
        }

        .cancel-button:hover {
          background: #c53030;
        }

        .loading,
        .modal-overlay {
          display: flex;
          justify-content: center;
          align-items: center;
          position: fixed;
          top: 0;
          left: 0;
          right: 0;
          bottom: 0;
          background: rgba(0, 0, 0, 0.5);
          z-index: 1000;
        }

        .modal-overlay {
          position: fixed;
        }

        .modal-content {
          background: white;
          border-radius: 8px;
          padding: 0;
          max-width: 600px;
          width: 90%;
          max-height: 80vh;
          overflow-y: auto;
          box-shadow: 0 10px 25px rgba(0, 0, 0, 0.1);
        }

        .modal-header {
          display: flex;
          justify-content: space-between;
          align-items: center;
          padding: 20px;
          border-bottom: 1px solid #e1e5e9;
        }

        .modal-header h3 {
          margin: 0;
          color: #2d3748;
          font-size: 20px;
          font-weight: 600;
        }

        .close-button {
          background: none;
          border: none;
          font-size: 24px;
          cursor: pointer;
          color: #718096;
          padding: 0;
          width: 24px;
          height: 24px;
          display: flex;
          align-items: center;
          justify-content: center;
        }

        .close-button:hover {
          color: #4a5568;
        }

        .modal-body {
          padding: 20px;
        }

        .detail-grid {
          display: grid;
          gap: 16px;
        }

        .detail-item {
          display: flex;
          justify-content: space-between;
          align-items: flex-start;
          padding-bottom: 8px;
          border-bottom: 1px solid #f1f5f9;
        }

        .detail-item:last-child {
          border-bottom: none;
        }

        .detail-item .label {
          font-weight: 500;
          color: #4a5568;
          min-width: 120px;
        }

        .detail-item .value {
          color: #2d3748;
          text-align: right;
          flex: 1;
        }

        .detail-item.error {
          background: #fed7d7;
          padding: 12px;
          border-radius: 4px;
          border-bottom: 1px solid #fc8181;
        }

        .detail-item.error .value {
          color: #c53030;
        }
      `}</style>
    </div>
  );
};

// Create Fine-Tune Job Modal Component
interface CreateFineTuneJobModalProps {
  onSave: (request: FineTuningRequest) => Promise<void>;
  onClose: () => void;
  isLoading: boolean;
}

const CreateFineTuneJobModal: React.FC<CreateFineTuneJobModalProps> = ({
  onSave,
  onClose,
  isLoading,
}) => {
  const [formData, setFormData] = useState<{
    jobName: string;
    description: string;
    baseModel: string;
    datasetPath: string;
    config: TrainingConfigInfo;
  }>({
    jobName: '',
    description: '',
    baseModel: 'codellama-7b',
    datasetPath: '',
    config: {
      learningRate: 5e-5,
      batchSize: 8,
      maxEpochs: 3,
      mixedPrecision: true,
      maxSeqLength: 2048,
      datasetSize: 10000,
    },
  });

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();

    if (!formData.jobName.trim() || !formData.datasetPath.trim()) {
      return;
    }

    const request: FineTuningRequest = {
      jobName: formData.jobName,
      description: formData.description || undefined,
      baseModel: formData.baseModel,
      datasetPath: formData.datasetPath,
      config: formData.config,
      enableMonitoring: true,
    };

    await onSave(request);
  };

  const updateFormData = (field: string, value: any) => {
    setFormData((prev) => ({
      ...prev,
      [field]: value,
    }));
  };

  const updateConfig = (field: string, value: any) => {
    setFormData((prev) => ({
      ...prev,
      config: {
        ...prev.config,
        [field]: value,
      },
    }));
  };

  return (
    <div className="modal-overlay" onClick={onClose}>
      <div className="modal-content form-modal" onClick={(e) => e.stopPropagation()}>
        <div className="modal-header">
          <h3>Create Fine-Tuning Job</h3>
          <button className="close-button" onClick={onClose}>
            ×
          </button>
        </div>

        <form onSubmit={handleSubmit} className="modal-body">
          <div className="form-grid">
            <div className="form-group">
              <label htmlFor="jobName" className="required">
                Job Name
              </label>
              <input
                type="text"
                id="jobName"
                value={formData.jobName}
                onChange={(e) => updateFormData('jobName', e.target.value)}
                placeholder="e.g., rust-code-analysis-model"
                required
                disabled={isLoading}
              />
            </div>

            <div className="form-group">
              <label htmlFor="description">Description</label>
              <textarea
                id="description"
                value={formData.description}
                onChange={(e) => updateFormData('description', e.target.value)}
                placeholder="Optional description of the training job"
                rows={3}
                disabled={isLoading}
              />
            </div>

            <div className="form-group">
              <label htmlFor="baseModel" className="required">
                Base Model
              </label>
              <select
                id="baseModel"
                value={formData.baseModel}
                onChange={(e) => updateFormData('baseModel', e.target.value)}
                required
                disabled={isLoading}
              >
                <option value="codellama-7b">CodeLlama 7B</option>
                <option value="codellama-13b">CodeLlama 13B</option>
                <option value="starcoder-7b">StarCoder 7B</option>
                <option value="starcoder-15b">StarCoder 15B</option>
              </select>
            </div>

            <div className="form-group">
              <label htmlFor="datasetPath" className="required">
                Dataset Path
              </label>
              <input
                type="text"
                id="datasetPath"
                value={formData.datasetPath}
                onChange={(e) => updateFormData('datasetPath', e.target.value)}
                placeholder="Path to training dataset (JSONL format)"
                required
                disabled={isLoading}
              />
            </div>

            <div className="form-section">
              <h4>Training Configuration</h4>

              <div className="config-grid">
                <div className="form-group">
                  <label htmlFor="learningRate">Learning Rate</label>
                  <input
                    type="number"
                    id="learningRate"
                    step="1e-6"
                    min="1e-6"
                    max="1e-3"
                    value={formData.config.learningRate}
                    onChange={(e) => updateConfig('learningRate', parseFloat(e.target.value))}
                    disabled={isLoading}
                  />
                </div>

                <div className="form-group">
                  <label htmlFor="batchSize">Batch Size</label>
                  <input
                    type="number"
                    id="batchSize"
                    min="1"
                    max="64"
                    value={formData.config.batchSize}
                    onChange={(e) => updateConfig('batchSize', parseInt(e.target.value, 10))}
                    disabled={isLoading}
                  />
                </div>

                <div className="form-group">
                  <label htmlFor="maxEpochs">Max Epochs</label>
                  <input
                    type="number"
                    id="maxEpochs"
                    min="1"
                    max="20"
                    value={formData.config.maxEpochs}
                    onChange={(e) => updateConfig('maxEpochs', parseInt(e.target.value, 10))}
                    disabled={isLoading}
                  />
                </div>

                <div className="form-group">
                  <label htmlFor="maxSeqLength">Max Sequence Length</label>
                  <input
                    type="number"
                    id="maxSeqLength"
                    min="512"
                    max="8192"
                    step="512"
                    value={formData.config.maxSeqLength}
                    onChange={(e) => updateConfig('maxSeqLength', parseInt(e.target.value, 10))}
                    disabled={isLoading}
                  />
                </div>
              </div>

              <div className="form-group checkbox">
                <label htmlFor="mixedPrecision">
                  <input
                    type="checkbox"
                    id="mixedPrecision"
                    checked={formData.config.mixedPrecision}
                    onChange={(e) => updateConfig('mixedPrecision', e.target.checked)}
                    disabled={isLoading}
                  />
                  Enable Mixed Precision Training
                </label>
              </div>
            </div>
          </div>

          <div className="form-actions">
            <button type="button" className="cancel-button" onClick={onClose} disabled={isLoading}>
              Cancel
            </button>
            <button
              type="submit"
              className="primary-button"
              disabled={isLoading || !formData.jobName.trim() || !formData.datasetPath.trim()}
            >
              {isLoading ? 'Creating...' : 'Create Training Job'}
            </button>
          </div>
        </form>
      </div>

      <style jsx>{`
        .form-modal {
          max-width: 700px;
        }

        .modal-body {
          padding: 24px;
        }

        .form-grid {
          display: grid;
          gap: 20px;
        }

        .form-section {
          border-top: 1px solid #e1e5e9;
          padding-top: 20px;
        }

        .form-section h4 {
          margin: 0 0 16px 0;
          color: #2d3748;
          font-size: 16px;
          font-weight: 600;
        }

        .config-grid {
          display: grid;
          grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
          gap: 16px;
          margin-bottom: 16px;
        }

        .form-group {
          display: flex;
          flex-direction: column;
          gap: 6px;
        }

        .form-group label {
          font-weight: 500;
          color: #4a5568;
          font-size: 14px;
        }

        .form-group label.required::after {
          content: ' *';
          color: #e53e3e;
        }

        .form-group input,
        .form-group select,
        .form-group textarea {
          padding: 8px 12px;
          border: 1px solid #e1e5e9;
          border-radius: 4px;
          font-size: 14px;
          transition: border-color 0.2s;
        }

        .form-group input:focus,
        .form-group select:focus,
        .form-group textarea:focus {
          outline: none;
          border-color: #3182ce;
        }

        .form-group input:disabled,
        .form-group select:disabled,
        .form-group textarea:disabled {
          background: #f7fafc;
          cursor: not-allowed;
        }

        .form-group.checkbox label {
          display: flex;
          align-items: center;
          gap: 8px;
          cursor: pointer;
          font-weight: 400;
          margin: 8px 0;
        }

        .form-actions {
          display: flex;
          justify-content: flex-end;
          gap: 12px;
          margin-top: 24px;
          padding-top: 20px;
          border-top: 1px solid #e1e5e9;
        }

        .cancel-button {
          background: #718096;
          color: white;
          border: none;
          padding: 10px 20px;
          border-radius: 6px;
          cursor: pointer;
          font-weight: 500;
          transition: background-color 0.2s;
        }

        .cancel-button:hover:not(:disabled) {
          background: #4a5568;
        }
      `}</style>
    </div>
  );
};

export default FineTuningPanel;
