import React, { useState, useEffect } from 'react';
import { RefactoringWizard, WizardStep } from './RefactoringWizard';
import { ExtractInterfaceWizard } from './ExtractInterfaceWizard';
import { AsyncAwaitConversionWizard } from './AsyncAwaitConversionWizard';
import { PatternConversionWizard } from './PatternConversionWizard';
import { BatchRefactoringWizard } from './BatchRefactoringWizard';
import type { RefactoringContext, RefactoringType } from '../../../../types/refactoring';

export interface UnifiedWizardProps {
  refactoringType: RefactoringType;
  context: RefactoringContext | null;
  open: boolean;
  onComplete: (config: Record<string, any>) => void;
  onCancel: () => void;
  onClose: () => void;
}

/**
 * Unified wizard that consolidates all refactoring wizards into a shared system
 * This eliminates duplicate wizard code and provides a consistent user experience
 */
export const UnifiedRefactoringWizard: React.FC<UnifiedWizardProps> = ({
  refactoringType,
  context,
  open,
  onComplete,
  onCancel,
  onClose,
}) => {
  const [steps, setSteps] = useState<WizardStep[]>([]);
  const [isLoading, setIsLoading] = useState(true);

  useEffect(() => {
    if (!open) {
      setSteps([]);
      setIsLoading(true);
      return;
    }

    setIsLoading(true);

    // Define steps based on refactoring type
    const wizardSteps: WizardStep[] = [];

    switch (refactoringType) {
      case 'extract-interface':
        wizardSteps.push({
          id: 'extract-interface-config',
          title: 'Extract Interface Configuration',
          description:
            'Configure which methods and properties to include in the extracted interface.',
          component: ExtractInterfaceWizard,
          validation: async (config: Record<string, any>) => {
            const isValid = config.selectedClasses?.length > 0 && config.interfaceName?.trim();
            return {
              isValid,
              errors: isValid ? [] : ['Please select classes and provide an interface name'],
            };
          },
        });
        break;

      case 'convert-to-async':
      case 'async-await-conversion':
        wizardSteps.push({
          id: 'async-conversion-config',
          title: 'Async/Await Conversion Configuration',
          description: 'Select functions to convert from promise-based to async/await syntax.',
          component: AsyncAwaitConversionWizard,
          validation: async (config: Record<string, any>) => {
            const isValid = config.conversionTargets?.length > 0;
            return {
              isValid,
              errors: isValid ? [] : ['Please select at least one function to convert'],
            };
          },
        });
        break;

      case 'pattern-conversion':
        wizardSteps.push({
          id: 'pattern-conversion-config',
          title: 'Pattern Conversion Configuration',
          description: 'Select code patterns to convert to more modern or efficient alternatives.',
          component: PatternConversionWizard,
          validation: async (config: Record<string, any>) => {
            const isValid = config.selectedPatternIds?.length > 0;
            return {
              isValid,
              errors: isValid ? [] : ['Please select at least one pattern to convert'],
            };
          },
        });
        break;

      case 'batch-interface-extraction':
      case 'batch-pattern-conversion':
        wizardSteps.push({
          id: 'batch-config',
          title: 'Batch Refactoring Configuration',
          description: 'Configure multiple refactoring operations to execute together.',
          component: BatchRefactoringWizard,
          validation: async (config: Record<string, any>) => {
            const isValid = config.operations?.length > 0;
            return {
              isValid,
              errors: isValid ? [] : ['Please add at least one refactoring operation'],
            };
          },
        });
        break;

      // Add other refactoring types here as needed
      default:
        wizardSteps.push({
          id: 'basic-config',
          title: 'Refactoring Configuration',
          description: 'Configure the refactoring parameters for the selected operation.',
          component: React.memo(() => (
            <div>
              <p>Configuration options for {refactoringType} will be available here.</p>
              <p>This is a placeholder for future configuration options.</p>
            </div>
          )),
        });
    }

    setSteps(wizardSteps);
    setIsLoading(false);
  }, [refactoringType, open]);

  // Helper function to get refactoring display name
  const getRefactoringDisplayName = (type: RefactoringType): string => {
    const names: Record<RefactoringType, string> = {
      'extract-interface': 'Extract Interface',
      'convert-to-async': 'Convert to Async',
      'async-await-conversion': 'Async/Await Conversion',
      'pattern-conversion': 'Pattern Conversion',
      'batch-interface-extraction': 'Batch Interface Extraction',
      'batch-pattern-conversion': 'Batch Pattern Conversion',
      rename: 'Rename',
      'extract-method': 'Extract Method',
      'extract-function': 'Extract Function',
      'extract-variable': 'Extract Variable',
      'extract-class': 'Extract Class',
      'move-method': 'Move Method',
      'move-class': 'Move Class',
      'move-file': 'Move File',
      'inline-method': 'Inline Method',
      'inline-function': 'Inline Function',
      'inline-variable': 'Inline Variable',
      'remove-parameter': 'Remove Parameter',
      'introduce-parameter': 'Introduce Parameter',
      'replace-constructor': 'Replace Constructor',
      'replace-conditionals': 'Replace Conditionals',
      'convert-method-to-function': 'Convert to Function',
      'split-class': 'Split Class',
      'merge-classes': 'Merge Classes',
      'change-signature': 'Change Signature',
      'add-delegation': 'Add Delegation',
      'remove-delegation': 'Remove Delegation',
      'encapsulate-field': 'Encapsulate Field',
      'localize-variable': 'Localize Variable',
      'add-missing-imports': 'Add Missing Imports',
      'sort-imports': 'Sort Imports',
      'generate-getters-setters': 'Generate Getters/Setters',
      'interface-extraction': 'Interface Extraction',
      'async-await-pattern-conversion': 'Async/Await Pattern Conversion',
    };
    return names[type] || type.replace('-', ' ').replace(/\b\w/g, (l) => l.toUpperCase());
  };

  if (!open || isLoading) {
    return null;
  }

  return (
    <RefactoringWizard
      open={open}
      title={`${getRefactoringDisplayName(refactoringType)} Configuration`}
      steps={steps}
      context={context}
      onComplete={onComplete}
      onCancel={onCancel}
      onClose={onClose}
    />
  );
};
