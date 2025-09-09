import { useState, useCallback } from 'react';

/**
 * Centralized loading state pattern
 */
export function useLoadingState(initialValue = false) {
  const [isLoading, setIsLoading] = useState<boolean>(initialValue);

  const withLoading = useCallback(async <T>(
    asyncFn: () => Promise<T>
  ): Promise<T | null> => {
    setIsLoading(true);
    try {
      return await asyncFn();
    } finally {
      setIsLoading(false);
    }
  }, []);

  return { isLoading, setIsLoading, withLoading };
}

/**
 * Centralized error state pattern
 */
export function useErrorState<T = string>(initialError: T | null = null) {
  const [error, setError] = useState<T | null>(initialError);

  const clearError = useCallback(() => setError(null), []);
  const setErrorFromException = useCallback((err: Error | unknown) => {
    const errorMessage = err instanceof Error ? err.message : String(err);
    setError(errorMessage as T);
  }, []);

  return { error, setError, clearError, setErrorFromException };
}

/**
 * Combined loading and error state pattern
 */
export function useAsyncOperation(initialLoading = false) {
  const loading = useLoadingState(initialLoading);
  const error = useErrorState<string>();

  const execute = useCallback(async <T>(
    asyncFn: () => Promise<T>
  ): Promise<T | null> => {
    loading.setIsLoading(true);
    error.clearError();

    try {
      const result = await asyncFn();
      return result;
    } catch (err) {
      error.setErrorFromException(err);
      return null;
    } finally {
      loading.setIsLoading(false);
    }
  }, [loading, error]);

  const reset = useCallback(() => {
    loading.setIsLoading(false);
    error.clearError();
  }, [loading, error]);

  return {
    isLoading: loading.isLoading,
    error: error.error,
    execute,
    reset,
    setLoading: loading.setIsLoading,
    setError: error.setError,
    clearError: error.clearError,
  };
}

/**
 * Toggle state pattern for checkbox-like components
 */
export function useToggleState(initialValue = false) {
  const [state, setState] = useState<boolean>(initialValue);

  const toggle = useCallback(() => setState(prev => !prev), []);
  const setTrue = useCallback(() => setState(true), []);
  const setFalse = useCallback(() => setState(false), []);
  const reset = useCallback(() => setState(initialValue), [initialValue]);

  return { state, setState, toggle, setTrue, setFalse, reset };
}

/**
 * Array state management pattern
 */
export function useArrayState<T>(initialArray: T[] = []) {
  const [array, setArray] = useState<T[]>(initialArray);

  const add = useCallback((item: T) => {
    setArray(prev => [...prev, item]);
  }, []);

  const remove = useCallback((index: number) => {
    setArray(prev => prev.filter((_, i) => i !== index));
  }, []);

  const removeItem = useCallback((item: T) => {
    setArray(prev => prev.filter(i => i !== item));
  }, []);

  const clear = useCallback(() => setArray([]), []);

  const update = useCallback((index: number, item: T) => {
    setArray(prev => prev.map((existing, i) => i === index ? item : existing));
  }, []);

  const findIndex = useCallback((predicate: (item: T) => boolean): number => {
    return array.findIndex(predicate);
  }, [array]);

  return { array, setArray, add, remove, removeItem, update, clear, findIndex };
}

/**
 * Form field state pattern with validation
 */
export function useFormField<T>(initialValue: T, validator?: (value: T) => string | null) {
  const [value, setValue] = useState<T>(initialValue);
  const [error, setError] = useState<string | null>(null);

  const setValueWithValidation = useCallback((newValue: T) => {
    setValue(newValue);
    if (validator) {
      const validationError = validator(newValue);
      setError(validationError);
    } else {
      setError(null);
    }
  }, [validator]);

  const validate = useCallback(() => {
    if (validator) {
      const validationError = validator(value);
      setError(validationError);
      return !validationError;
    }
    return true;
  }, [value, validator]);

  const reset = useCallback(() => {
    setValue(initialValue);
    setError(null);
  }, [initialValue]);

  return { value, error, setValue: setValueWithValidation, validate, reset };
}

/**
 * Modal/dialog state pattern
 */
export function useModalState(initialOpen = false) {
  const [isOpen, setIsOpen] = useState<boolean>(initialOpen);

  const open = useCallback(() => setIsOpen(true), []);
  const close = useCallback(() => setIsOpen(false), []);
  const toggle = useCallback(() => setIsOpen(prev => !prev), []);

  return { isOpen, open, close, toggle };
}

/**
 * Selection state pattern for list/table components
 */
export function useSelectionState<T>(multiple = false) {
  const [selected, setSelected] = useState<T | T[] | null>(multiple ? [] : null);

  const select = useCallback((item: T) => {
    if (multiple) {
      const current = selected as T[];
      const isSelected = current.includes(item);
      if (isSelected) {
        setSelected(current.filter(existing => existing !== item));
      } else {
        setSelected([...current, item]);
      }
    } else {
      setSelected(selected === item ? null : item);
    }
  }, [selected, multiple]);

  const selectAll = useCallback((items: T[]) => {
    if (multiple) {
      const current = selected as T[];
      const allSelected = items.every(item => current.includes(item));
      if (allSelected) {
        setSelected([]);
      } else {
        setSelected(items);
      }
    }
  }, [selected, multiple]);

  const clear = useCallback(() => {
    setSelected(multiple ? [] : null);
  }, [multiple]);

  const isSelected = useCallback((item: T): boolean => {
    if (multiple) {
      return (selected as T[]).includes(item);
    }
    return selected === item;
  }, [selected, multiple]);

  const isAllSelected = useCallback((items: T[]): boolean => {
    if (!multiple) return false;
    return items.length > 0 && items.every(item => (selected as T[]).includes(item));
  }, [selected, multiple]);

  const isIndeterminate = useCallback((items: T[]): boolean => {
    if (!multiple) return false;
    const selectedItems = selected as T[];
    const selectedCount = selectedItems.length;
    return selectedCount > 0 && selectedCount < items.length;
  }, [selected, multiple]);

  return {
    selected,
    select,
    selectAll,
    clear,
    isSelected,
    isAllSelected,
    isIndeterminate,
  };
}

export default {
  useLoadingState,
  useErrorState,
  useAsyncOperation,
  useToggleState,
  useArrayState,
  useFormField,
  useModalState,
  useSelectionState,
};