import { useAppDispatch, useAppSelector } from '../store/store';
import { Project, setCurrentProject, setError, setLoading } from '../store/slices/projectsSlice';

export const useCurrentProject = () => {
  const dispatch = useAppDispatch();
  const currentProject = useAppSelector((state) => state.projects.currentProject);
  const projects = useAppSelector((state) => state.projects.projects);
  const loading = useAppSelector((state) => state.projects.loading);
  const error = useAppSelector((state) => state.projects.error);

  const setProject = (project: Project | null) => {
    dispatch(setCurrentProject(project));
  };

  const setLoadingState = (isLoading: boolean) => {
    dispatch(setLoading(isLoading));
  };

  const setErrorState = (errorMessage: string | null) => {
    dispatch(setError(errorMessage));
  };

  return {
    currentProject,
    projects,
    loading,
    error,
    setCurrentProject: setProject,
    setLoading: setLoadingState,
    setError: setErrorState,
  };
};

// Helper hook to get the current project path
export const useCurrentProjectPath = (): string | null => {
  const { currentProject } = useCurrentProject();
  return currentProject?.path || null;
};
