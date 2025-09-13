import { useState, useCallback, useEffect, useRef } from 'react';

/**
 * Form validation rule
 */
export type ValidationRule<T> = {
  /** Validation function */
  validate: (value: any, formData?: T) => boolean;
  /** Error message when validation fails */
  message: string;
};

/**
 * Form field configuration
 */
export type FormField<T> = {
  /** Initial value */
  initialValue?: any;
  /** Validation rules */
  rules?: ValidationRule<T>[];
  /** Transform function for output */
  transform?: (value: any) => any;
  /** Custom validator function */
  validator?: (value: any, formData: Partial<T>) => string | null;
};

/**
 * Form configuration
 */
export type FormConfig<T> = {
  /** Field configurations */
  fields: Record<keyof T, FormField<T>>;
  /** Submit function */
  onSubmit?: (data: T) => Promise<void> | void;
  /** Custom validation function for entire form */
  validateForm?: (data: Partial<T>) => Record<string, string>;
  /** Whether to validate on change */
  validateOnChange?: boolean;
  /** Whether to validate on blur */
  validateOnBlur?: boolean;
};

/**
 * Form state
 */
export type FormState<T> = {
  /** Form values */
  values: Partial<T>;
  /** Field errors */
  errors: Record<string, string>;
  /** Field touched states */
  touched: Record<string, boolean>;
  /** Whether form is submitting */
  isSubmitting: boolean;
  /** Whether form is dirty (has unsaved changes) */
  isDirty: boolean;
  /** Whether form is valid */
  isValid: boolean;
};

/**
 * Return type for useFormManager hook
 */
export type FormManagerReturn<T> = {
  /** Form state */
  formState: FormState<T>;
  /** Set field value */
  setValue: (field: keyof T, value: any) => void;
  /** Get field value */
  getValue: (field: keyof T) => any;
  /** Set field error */
  setFieldError: (field: keyof T, error: string) => void;
  /** Clear field error */
  clearFieldError: (field: keyof T) => void;
  /** Validate field */
  validateField: (field: keyof T) => boolean;
  /** Validate entire form */
  validateForm: () => boolean;
  /** Reset form */
  reset: () => void;
  /** Submit form */
  submit: () => Promise<boolean>;
  /** Register field event handlers */
  register: (field: keyof T) => {
    value: any;
    onChange: (value: any) => void;
    onBlur: () => void;
    error?: string;
  };
};

/**
 * Hook for managing form state, validation, and submission
 *
 * @param config - Form configuration
 * @returns Form management functions and state
 *
 * @example
 * ```tsx
 * const form = useFormManager<User>({
 *   fields: {
 *     name: {
 *       initialValue: '',
 *       rules: [{ validate: (val) => val.length > 0, message: 'Required' }]
 *     },
 *     email: {
 *       initialValue: '',
 *       rules: [{ validate: (val) => /\S+@\S+\.\S+/.test(val), message: 'Invalid email' }]
 *     }
 *   },
 *   onSubmit: async (data) => {
 *     await api.createUser(data);
 *   }
 * });
 *
 * return (
 *   <form onSubmit={form.submit}>
 *     <TextField {...form.register('name')} label="Name" />
 *     <TextField {...form.register('email')} label="Email" />
 *     <Button type="submit" disabled={form.formState.isSubmitting}>
 *       {form.formState.isSubmitting ? 'Saving...' : 'Save'}
 *     </Button>
 *   </form>
 * );
 * ```
 */
export function useFormManager<T extends Record<string, any>>(
  config: FormConfig<T>
): FormManagerReturn<T> {
  const {
    fields,
    onSubmit,
    validateForm: customValidateForm,
    validateOnChange = true,
    validateOnBlur = true,
  } = config;

  // Initialize form state
  const getInitialValues = useCallback(() => {
    const initial: Record<string, any> = {};
    const errors: Record<string, string> = {};
    const touched: Record<string, boolean> = {};

    Object.entries(fields).forEach(([fieldName, fieldConfig]) => {
      initial[fieldName] = fieldConfig.initialValue ?? '';
      errors[fieldName] = '';
      touched[fieldName] = false;
    });

    return { initial, errors, touched };
  }, [fields]);

  const { initial, errors: initialErrors, touched: initialTouched } = getInitialValues();

  const [values, setValues] = useState<Partial<T>>(initial as Partial<T>);
  const [errors, setErrors] = useState<Record<string, string>>(initialErrors);
  const [touched, setTouched] = useState<Record<string, boolean>>(initialTouched);
  const [isSubmitting, setIsSubmitting] = useState(false);

  // Track if form is dirty
  const isDirty = Object.entries(fields).some(([fieldName, fieldConfig]) => {
    const currentValue = values[fieldName];
    const initialValue = fieldConfig.initialValue ?? '';
    return currentValue !== initialValue;
  });

  // Validate field
  const validateField = useCallback(
    (field: keyof T): boolean => {
      const fieldName = field as string;
      const fieldConfig = fields[field];
      if (!fieldConfig) return true;

      const value = values[fieldName];
      const formData = values as Partial<T>;

      // Check validation rules
      if (fieldConfig.rules) {
        for (const rule of fieldConfig.rules) {
          if (!rule.validate(value, formData as T)) {
            setErrors((prev) => ({ ...prev, [fieldName]: rule.message }));
            return false;
          }
        }
      }

      // Check custom validator
      if (fieldConfig.validator) {
        const error = fieldConfig.validator(value, formData);
        if (error) {
          setErrors((prev) => ({ ...prev, [fieldName]: error }));
          return false;
        }
      }

      // Clear error if validation passes
      setErrors((prev) => ({ ...prev, [fieldName]: '' }));
      return true;
    },
    [fields, values]
  );

  // Validate entire form
  const validateForm = useCallback((): boolean => {
    let isValid = true;

    // Validate all fields
    Object.keys(fields).forEach((fieldName) => {
      if (!validateField(fieldName as keyof T)) {
        isValid = false;
      }
    });

    // Run custom form validator
    if (customValidateForm && isValid) {
      const formErrors = customValidateForm(values);
      if (Object.keys(formErrors).length > 0) {
        setErrors((prev) => ({ ...prev, ...formErrors }));
        isValid = false;
      }
    }

    return isValid;
  }, [fields, validateField, customValidateForm, values]);

  // Update form validity whenever values change
  useEffect(() => {
    const currentIsValid = Object.values(errors).every((error) => !error || error === '');
    // Update isValid could be added to state if needed
  }, [errors]);

  // Set field value
  const setValue = useCallback(
    (field: keyof T, value: any) => {
      const fieldName = field as string;

      setValues((prev) => ({ ...prev, [fieldName]: value }));

      if (validateOnChange || touched[fieldName]) {
        // Mark as touched
        setTouched((prev) => ({ ...prev, [fieldName]: true }));

        // Validate field
        setTimeout(() => validateField(field), 0); // Debounce validation
      }
    },
    [validateOnChange, touched, validateField]
  );

  // Get field value
  const getValue = useCallback(
    (field: keyof T) => {
      return values[field as string];
    },
    [values]
  );

  // Set field error
  const setFieldError = useCallback((field: keyof T, error: string) => {
    const fieldName = field as string;
    setErrors((prev) => ({ ...prev, [fieldName]: error }));
  }, []);

  // Clear field error
  const clearFieldError = useCallback((field: keyof T) => {
    const fieldName = field as string;
    setErrors((prev) => ({ ...prev, [fieldName]: '' }));
  }, []);

  // Handle field blur
  const handleFieldBlur = useCallback(
    (field: keyof T) => {
      const fieldName = field as string;

      setTouched((prev) => ({ ...prev, [fieldName]: true }));

      if (validateOnBlur) {
        validateField(field);
      }
    },
    [validateOnBlur, validateField]
  );

  // Reset form
  const reset = useCallback(() => {
    const { initial: newInitial, errors: newErrors, touched: newTouched } = getInitialValues();
    setValues(newInitial as Partial<T>);
    setErrors(newErrors);
    setTouched(newTouched);
    setIsSubmitting(false);
  }, [getInitialValues]);

  // Submit form
  const submit = useCallback(async (): Promise<boolean> => {
    if (isSubmitting) return false;

    // Validate all fields
    if (!validateForm()) return false;

    setIsSubmitting(true);

    try {
      // Transform data
      const finalData: Record<string, any> = {};
      Object.entries(fields).forEach(([fieldName, fieldConfig]) => {
        const value = values[fieldName];
        const transform = fieldConfig.transform;
        finalData[fieldName] = transform ? transform(value) : value;
      });

      if (onSubmit) {
        await onSubmit(finalData as T);
      }

      return true;
    } catch (error) {
      console.error('Form submission error:', error);
      const errorMessage = error instanceof Error ? error.message : 'Submission failed';
      setErrors({ form: errorMessage });
      return false;
    } finally {
      setIsSubmitting(false);
    }
  }, [isSubmitting, validateForm, fields, values, onSubmit]);

  // Register field event handlers
  const register = useCallback(
    (field: keyof T) => {
      const fieldName = field as string;
      const value = values[fieldName];

      return {
        value,
        onChange: (newValue: any) => setValue(field, newValue),
        onBlur: () => handleFieldBlur(field),
        error: errors[fieldName] || undefined,
      };
    },
    [values, setValue, handleFieldBlur, errors]
  );

  // Form state
  const formState: FormState<T> = {
    values,
    errors,
    touched,
    isSubmitting,
    isDirty,
    isValid: Object.values(errors).every((error) => !error || error === ''),
  };

  return {
    formState,
    setValue,
    getValue,
    setFieldError,
    clearFieldError,
    validateField,
    validateForm,
    reset,
    submit,
    register,
  };
}
