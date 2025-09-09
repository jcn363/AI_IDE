import { createSlice, PayloadAction } from '@reduxjs/toolkit';

export interface Project {
  id: string;
  name: string;
  path: string;
}

export interface ProjectsState {
  currentProject: Project | null;
  projects: Project[];
  loading: boolean;
  error: string | null;
}

const initialState: ProjectsState = {
  currentProject: null,
  projects: [],
  loading: false,
  error: null,
};

const projectsSlice = createSlice({
  name: 'projects',
  initialState,
  reducers: {
    setCurrentProject: (state, action: PayloadAction<Project | null>) => {
      state.currentProject = action.payload;
    },
    setProjects: (state, action: PayloadAction<Project[]>) => {
      state.projects = action.payload;
    },
    setLoading: (state, action: PayloadAction<boolean>) => {
      state.loading = action.payload;
    },
    setError: (state, action: PayloadAction<string | null>) => {
      state.error = action.payload;
    },
  },
});

export const projectsActions = projectsSlice.actions;
export const { setCurrentProject, setProjects, setLoading, setError } = projectsSlice.actions;
export default projectsSlice.reducer;
