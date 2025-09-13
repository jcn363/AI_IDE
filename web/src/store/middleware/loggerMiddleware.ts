import { Middleware } from '@reduxjs/toolkit';
import { debug, info, warn } from '@/utils/logging/config';

/**
 * Logs all actions and state changes in development mode
 */
const loggerMiddleware: Middleware = (store) => (next) => (action: unknown) => {
  // Skip logging in test environment
  if (process.env.NODE_ENV === 'test') {
    return next(action);
  }

  const startTime = Date.now();
  const prevState = store.getState();

  // Type guard to check if action has the expected shape
  const isAction = (a: unknown): a is { type: string; payload?: unknown; meta?: unknown } => {
    return a !== null && typeof a === 'object' && 'type' in a;
  };

  // Log the action if it has the expected shape
  if (isAction(action)) {
    debug('Dispatching action:', {
      type: action.type,
      payload: action.payload,
      meta: action.meta,
    });
  } else {
    // Log a warning for non-standard actions
    warn('Received non-standard action', { action });
  }

  // Call the next dispatch method in the middleware chain
  const result = next(action);

  const endTime = Date.now();
  const nextState = store.getState();
  const duration = endTime - startTime;

  // Log state changes
  if (process.env.NODE_ENV === 'development' && isAction(action)) {
    debug('State changed:', {
      action: action.type,
      prevState,
      nextState,
      duration: `${duration}ms`,
    });
  }

  return result;
};

/**
 * Middleware to catch and log errors in actions
 */
const errorMiddleware: Middleware = (store) => (next) => (action) => {
  try {
    return next(action);
  } catch (error) {
    console.error('Error in action:', error);
    throw error;
  }
};

export const middleware = [
  process.env.NODE_ENV !== 'production' && loggerMiddleware,
  errorMiddleware,
].filter(Boolean) as Middleware[];
