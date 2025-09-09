import React, { useState } from 'react';
import {
  Box,
  Button,
  Typography,
  Paper,
} from '@mui/material';
import CheckCircle from '@mui/icons-material/CheckCircle';

export interface StepComponentProps {
  context: any;
  config: Record<string, any>;
  onConfigChange: (config: any) => void;
  onValidation?: (isValid: boolean) => void;
}

// Updated wizard step structure
export interface WizardStep {
  id: string;
  title: string;
  description: string;
  component: React.ComponentType<StepComponentProps>;
  validateStep?: (data: any) => boolean;
}
export interface WizardStepProps {
  title: string;
  description?: string;
  content: React.ReactNode;
  isActive: boolean;
  isCompleted: boolean;
  context?: any;
  onConfigChange?: (config: any) => void;
  onValidation?: (isValid: boolean) => void;
}

// Individual wizard step component (renamed to avoid collision)
export const WizardStepCard: React.FC<WizardStepProps> = ({
  title,
  description,
  content,
  isActive,
  isCompleted
}) => {
  return (
    <Box
      sx={{
        mb: 2,
        border: 2,
        borderColor: isCompleted ? 'success.main' : isActive ? 'primary.main' : 'grey.300',
        borderRadius: 2,
        bgcolor: isActive ? 'grey.50' : 'white',
      }}
    >
      <Box
        sx={{
          display: 'flex',
          p: 2,
          alignItems: 'center',
        }}
      >
        <Box
          sx={{
            width: 32,
            height: 32,
            borderRadius: '50%',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            mr: 2,
            fontSize: 14,
            fontWeight: 600,
            color: 'white',
            bgcolor: isCompleted ? 'success.main' : isActive ? 'primary.main' : 'grey.400',
          }}
        >
          {isCompleted ? <CheckCircle sx={{ fontSize: 16 }} /> : isActive ? title.charAt(0) : ''}
        </Box>
        <Box sx={{ flex: 1 }}>
          <Typography variant="h6" sx={{ m: 0, mb: 0.5, fontSize: '1rem', fontWeight: 500 }}>
            {title}
          </Typography>
          {description && (
            <Typography variant="body2" sx={{ m: 0, color: 'text.secondary' }}>
              {description}
            </Typography>
          )}
        </Box>
      </Box>
      {isActive && (
        <Box sx={{ px: 2, pb: 2, pl: '66px' }}> {/* Account for indicator width */}
          {content}
        </Box>
      )}
    </Box>
  );
};

// Main wizard component
export interface RefactoringWizardProps {
  open: boolean;
  title: string;
  steps: WizardStep[];
  context: any;
  onComplete: (config: Record<string, any>) => void;
  onCancel: () => void;
  onClose: () => void;
  onConfigChange?: (config: any) => void;
}

export const RefactoringWizard: React.FC<RefactoringWizardProps> = ({
  open,
  title,
  steps,
  context,
  onComplete,
  onCancel,
  onClose,
  onConfigChange
}) => {
  const [currentStep, setCurrentStep] = useState(0);
  const [wizardConfig, setWizardConfig] = useState<Record<string, any>>({});

  // Return null if not open
  if (!open) return null;

  const handleNext = () => {
    if (currentStep < steps.length - 1) {
      setCurrentStep(currentStep + 1);
    } else {
      onComplete(wizardConfig);
    }
  };

  const handlePrevious = () => {
    if (currentStep > 0) {
      setCurrentStep(currentStep - 1);
    }
  };

  const handleConfigChange = (newConfig: Record<string, any>) => {
    const updatedConfig = { ...wizardConfig, ...newConfig };
    setWizardConfig(updatedConfig);
    if (onConfigChange) {
      onConfigChange(updatedConfig);
    }
  };

  const currentStepData = steps[currentStep];

  // Create step component with proper props
  const StepComponent = steps[currentStep].component;

  return (
    <Paper
      sx={{
        display: 'flex',
        flexDirection: 'column',
        bgcolor: 'background.paper',
        borderRadius: 2,
        boxShadow: 3,
        minHeight: 400,
        maxWidth: 600,
        mx: 'auto',
        position: 'absolute',
        top: '50%',
        left: '50%',
        transform: 'translate(-50%, -50%)',
        zIndex: 1300,
      }}
    >
      <Box
        sx={{
          p: 3,
          borderBottom: 1,
          borderColor: 'divider',
          display: 'flex',
          justifyContent: 'space-between',
          alignItems: 'center',
        }}
      >
        <Typography variant="h5" sx={{ m: 0, color: 'text.primary', fontWeight: 600 }}>
          {title}
        </Typography>
        <Typography variant="body2" sx={{ color: 'text.secondary' }}>
          Step {currentStep + 1} of {steps.length}
        </Typography>
      </Box>

      <Box sx={{ flex: 1, p: 3 }}>
        <Box sx={{ mb: 3 }}>
          <Typography variant="h6" sx={{ mb: 1 }}>
            {currentStepData.title}
          </Typography>
          <Typography variant="body2" sx={{ color: 'text.secondary', mb: 2 }}>
            {currentStepData.description}
          </Typography>
        </Box>

        <StepComponent
          context={context}
          config={wizardConfig}
          onConfigChange={handleConfigChange}
          onValidation={(isValid: boolean) => {
            // Handle validation if needed
            console.log(`Step validation: ${isValid}`);
          }}
        />
      </Box>

      <Box
        sx={{
          p: 3,
          borderTop: 1,
          borderColor: 'divider',
          display: 'flex',
          justifyContent: 'space-between',
          alignItems: 'center',
        }}
      >
        {/* Step indicator */}
        <Box sx={{ display: 'flex', gap: 1 }}>
          {steps.map((_, index) => (
            <Box
              key={index}
              sx={{
                width: 8,
                height: 8,
                borderRadius: '50%',
                bgcolor: index === currentStep ? 'primary.main' : index < currentStep ? 'success.main' : 'grey.300',
              }}
            />
          ))}
        </Box>

        {/* Navigation buttons */}
        <Box sx={{ display: 'flex', gap: 2 }}>
          <Button
            onClick={onClose}
            variant="outlined"
            color="inherit"
          >
            Cancel
          </Button>
          {currentStep > 0 && (
            <Button
              onClick={handlePrevious}
              variant="outlined"
              color="inherit"
            >
              Previous
            </Button>
          )}
          <Button
            onClick={handleNext}
            variant="contained"
            color="primary"
          >
            {currentStep === steps.length - 1 ? 'Complete' : 'Next'}
          </Button>
        </Box>
      </Box>
    </Paper>
  );
};

export default RefactoringWizard;