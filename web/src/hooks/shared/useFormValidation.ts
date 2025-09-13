import { useState, useCallback } from 'react';

/**
 * Validation rule definition
 */
export interface ValidationRule<T = any> {
  validate: (value: T, formData?: Record<string, any>) => string | null;
  required?: boolean;
}

/**
 * Hook for managing form validation state and validation rules
 */
export function useFormValidation(initialRules: Record<string, ValidationRule> = {}) {
  const [errors, setErrors] = useState<Record<string, string>>({});
  const [rules, setRules] = useState<Record<string, ValidationRule>>(initialRules);

  const validateField = useCallback(
    (fieldName: string, value: any, formData?: Record<string, any>): boolean => {
      const rule = rules[fieldName];
      if (!rule) return true;

      const error = rule.validate(value, formData);
      setErrors((prev) => ({
        ...prev,
        [fieldName]: error || '',
      }));

      return !error;
    },
    [rules]
  );

  const validateForm = useCallback(
    (formData: Record<string, any>): boolean => {
      const newErrors: Record<string, string> = {};
      let isValid = true;

      Object.entries(rules).forEach(([fieldName, rule]) => {
        const value = formData[fieldName];
        const error = rule.validate(value, formData);

        if (error) {
          newErrors[fieldName] = error;
          isValid = false;
        }
      });

      setErrors(newErrors);
      return isValid;
    },
    [rules]
  );

  const setFieldError = useCallback((fieldName: string, error: string) => {
    setErrors((prev) => ({
      ...prev,
      [fieldName]: error,
    }));
  }, []);

  const clearError = useCallback((fieldName: string) => {
    setErrors((prev) => ({
      ...prev,
      [fieldName]: '',
    }));
  }, []);

  const clearAllErrors = useCallback(() => {
    setErrors({});
  }, []);

  const addRule = useCallback((fieldName: string, rule: ValidationRule) => {
    setRules((prev) => ({
      ...prev,
      [fieldName]: rule,
    }));
  }, []);

  const removeRule = useCallback((fieldName: string) => {
    setRules((prev) => {
      const newRules = { ...prev };
      delete newRules[fieldName];
      return newRules;
    });
  }, []);

  return {
    errors,
    rules,
    validateField,
    validateForm,
    setFieldError,
    clearError,
    clearAllErrors,
    addRule,
    removeRule,
    hasErrors: Object.values(errors).some((error) => error.length > 0),
  };
}

/**
 * Common validation rules factory
 */
export const createValidationRules = {
  required: (message = 'This field is required'): ValidationRule => ({
    validate: (value: any) => {
      if (value === null || value === undefined) return message;
      if (typeof value === 'string' && value.trim().length === 0) return message;
      if (Array.isArray(value) && value.length === 0) return message;
      return null;
    },
    required: true,
  }),

  minLength: (minLength: number, message?: string): ValidationRule<string> => ({
    validate: (value: string) => {
      if (!value || value.length < minLength) {
        return message || `Must be at least ${minLength} characters`;
      }
      return null;
    },
  }),

  maxLength: (maxLength: number, message?: string): ValidationRule<string> => ({
    validate: (value: string) => {
      if (value && value.length > maxLength) {
        return message || `Must be less than ${maxLength} characters`;
      }
      return null;
    },
  }),

  email: (message = 'Invalid email format'): ValidationRule<string> => ({
    validate: (value: string) => {
      const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
      if (value && !emailRegex.test(value)) return message;
      return null;
    },
  }),

  numberRange: (min?: number, max?: number, message?: string): ValidationRule<number> => ({
    validate: (value: number) => {
      if (typeof value !== 'number' || isNaN(value)) return 'Must be a number';
      if (min !== undefined && value < min) return message || `Must be at least ${min}`;
      if (max !== undefined && value > max) return message || `Must be no more than ${max}`;
      return null;
    },
  }),

  pattern: (regex: RegExp, message = 'Invalid format'): ValidationRule<string> => ({
    validate: (value: string) => {
      if (value && !regex.test(value)) return message;
      return null;
    },
  }),

  fileExtension: (allowedExtensions: string[], message?: string): ValidationRule<string> => ({
    validate: (value: string) => {
      if (!value) return null;
      const extension = value.split('.').pop()?.toLowerCase();
      if (!extension || !allowedExtensions.includes(extension)) {
        return message || `File must be one of: ${allowedExtensions.join(', ')}`;
      }
      return null;
    },
  }),
};
