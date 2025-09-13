import { TypedUseSelectorHook, useDispatch, useSelector } from 'react-redux';
import type { RootState, AppThunk } from './types';
import { ThunkDispatch } from 'redux-thunk';
import { UnknownAction } from '@reduxjs/toolkit';
import store from './store';

type AppDispatch = ThunkDispatch<RootState, unknown, UnknownAction> & {
  <Returned = unknown, Arg = unknown>(asyncAction: AppThunk<Returned, Arg>): Promise<Returned>;
};

// Use throughout your app instead of plain `useDispatch` and `useSelector`
export const useAppDispatch = () => useDispatch<AppDispatch>();
export const useAppSelector: TypedUseSelectorHook<RootState> = useSelector;

export type { RootState, AppDispatch };
export { store };
