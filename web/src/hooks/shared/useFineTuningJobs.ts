import { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import type { FineTuneJobInfo } from '../../features/ai/types';

interface RealTimeMetrics {
  gpuUtilization: number;
  memoryUsage: number;
  loss: number;
  learningRate: number;
  samplesProcessed: number;
  eta?: string;
}

interface UseFineTuningJobsOptions {
  autoPoll?: boolean;
  pollInterval?: number;
}

interface UseFineTuningJobsReturn {
  // Job list management
  jobs: FineTuneJobInfo[];
  isLoading: boolean;
  error: string | null;
  refetchJobs: () => Promise<void>;

  // Real-time metrics
  metrics: { [jobId: string]: RealTimeMetrics | null };
  isPolling: boolean;

  // Job operations
  startJob: (jobId: string) => Promise<void>;
  stopJob: (jobId: string) => Promise<void>;
  pauseJob: (jobId: string) => Promise<void>;
  resumeJob: (jobId: string) => Promise<void>;
  cancelJob: (jobId: string) => Promise<void>;
  deleteJob: (jobId: string) => Promise<void>;

  // Utility functions
  getJobById: (jobId: string) => FineTuneJobInfo | undefined;
  formatDuration: (seconds: number) => string;
  pollJobMetrics: (jobId: string) => Promise<void>;
}

export const useFineTuningJobs = (options: UseFineTuningJobsOptions = {}): UseFineTuningJobsReturn => {
  const { autoPoll = true, pollInterval = 5000 } = options;

  const [jobs, setJobs] = useState<FineTuneJobInfo[]>([]);
  const [metrics, setMetrics] = useState<{ [jobId: string]: RealTimeMetrics | null }>({});
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [isPolling, setIsPolling] = useState(false);
  const [pollTimers, setPollTimers] = useState<{ [jobId: string]: NodeJS.Timeout | null }>({});

  // Fetch all fine-tuning jobs
  const refetchJobs = useCallback(async () => {
    setIsLoading(true);
    setError(null);
    try {
      const fetchedJobs = await invoke<FineTuneJobInfo[]>('list_finetune_jobs');
      setJobs(fetchedJobs);

      // Start polling for active jobs
      if (autoPoll) {
        const activeJobs = fetchedJobs.filter(job =>
          job.status === 'Training' || job.status === 'Initializing'
        );

        activeJobs.forEach(job => {
          if (!pollTimers[job.jobId]) {
            startPollingJob(job.jobId);
          }
        });

        setIsPolling(activeJobs.length > 0);
      }
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to fetch jobs';
      setError(errorMessage);
      console.error('Failed to fetch fine-tuning jobs:', err);
    } finally {
      setIsLoading(false);
    }
  }, [autoPoll, pollTimers]);

  // Start polling a specific job
  const startPollingJob = useCallback((jobId: string) => {
    const timer = setInterval(() => {
      pollJobMetrics(jobId);
    }, pollInterval);

    setPollTimers(prev => ({ ...prev, [jobId]: timer }));
  }, [pollInterval]);

  // Stop polling a specific job
  const stopPollingJob = useCallback((jobId: string) => {
    if (pollTimers[jobId]) {
      clearInterval(pollTimers[jobId]!);
      setPollTimers(prev => ({ ...prev, [jobId]: null }));
    }
  }, [pollTimers]);

  // Poll metrics for a job
  const pollJobMetrics = useCallback(async (jobId: string) => {
    try {
      const realTimeMetrics = await invoke<RealTimeMetrics>('get_finetune_job_metrics', { jobId });
      setMetrics(prev => ({ ...prev, [jobId]: realTimeMetrics }));
    } catch (err) {
      console.error(`Failed to fetch real-time metrics for job ${jobId}:`, err);
      if (err instanceof Error && err.message.includes('not found')) {
        stopPollingJob(jobId);
      }
    }
  }, [stopPollingJob]);

  // Job control operations
  const startJob = useCallback(async (jobId: string) => {
    try {
      await invoke('start_finetune_job', { jobId });
      await refetchJobs(); // Refresh job list
    } catch (err) {
      const errorMessage = `Failed to start job: ${err instanceof Error ? err.message : String(err)}`;
      setError(errorMessage);
      throw err;
    }
  }, [refetchJobs]);

  const stopJob = useCallback(async (jobId: string) => {
    try {
      await invoke('cancel_finetune_job', { jobId });
      await refetchJobs(); // Refresh job list
      stopPollingJob(jobId);
    } catch (err) {
      const errorMessage = `Failed to stop job: ${err instanceof Error ? err.message : String(err)}`;
      setError(errorMessage);
      throw err;
    }
  }, [refetchJobs, stopPollingJob]);

  const pauseJob = useCallback(async (jobId: string) => {
    try {
      await invoke('pause_finetune_job', { jobId });
      await refetchJobs(); // Refresh job list
      stopPollingJob(jobId);
    } catch (err) {
      const errorMessage = `Failed to pause job: ${err instanceof Error ? err.message : String(err)}`;
      setError(errorMessage);
      throw err;
    }
  }, [refetchJobs, stopPollingJob]);

  const resumeJob = useCallback(async (jobId: string) => {
    try {
      await invoke('resume_finetune_job', { jobId });
      await refetchJobs(); // Refresh job list
      startPollingJob(jobId);
    } catch (err) {
      const errorMessage = `Failed to resume job: ${err instanceof Error ? err.message : String(err)}`;
      setError(errorMessage);
      throw err;
    }
  }, [refetchJobs, startPollingJob]);

  const cancelJob = useCallback(async (jobId: string) => {
    try {
      await invoke('cancel_finetune_job', { jobId });
      await refetchJobs(); // Refresh job list
      stopPollingJob(jobId);
    } catch (err) {
      const errorMessage = `Failed to cancel job: ${err instanceof Error ? err.message : String(err)}`;
      setError(errorMessage);
      throw err;
    }
  }, [refetchJobs, stopPollingJob]);

  const deleteJob = useCallback(async (jobId: string) => {
    try {
      await invoke('delete_finetune_job', { jobId });
      await refetchJobs(); // Refresh job list
      stopPollingJob(jobId);
    } catch (err) {
      const errorMessage = `Failed to delete job: ${err instanceof Error ? err.message : String(err)}`;
      setError(errorMessage);
      throw err;
    }
  }, [refetchJobs, stopPollingJob]);

  // Utility functions
  const getJobById = useCallback((jobId: string) => {
    return jobs.find(job => job.jobId === jobId);
  }, [jobs]);

  const formatDuration = useCallback((seconds: number) => {
    const hours = Math.floor(seconds / 3600);
    const minutes = Math.floor((seconds % 3600) / 60);
    const remainingSeconds = Math.floor(seconds % 60);
    return `${hours}h ${minutes}m ${remainingSeconds}s`;
  }, []);

  // Cleanup on unmount
  useEffect(() => {
    return () => {
      Object.values(pollTimers).forEach(timer => {
        if (timer) clearInterval(timer);
      });
    };
  }, [pollTimers]);

  // Initial load
  useEffect(() => {
    refetchJobs();
  }, [refetchJobs]);

  return {
    jobs,
    isLoading,
    error,
    metrics,
    isPolling,
    refetchJobs,
    startJob,
    stopJob,
    pauseJob,
    resumeJob,
    cancelJob,
    deleteJob,
    getJobById,
    formatDuration,
    pollJobMetrics,
  };
};