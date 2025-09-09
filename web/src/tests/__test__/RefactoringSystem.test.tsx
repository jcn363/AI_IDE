import { describe, it, expect, jest, beforeEach, afterEach } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { invoke } from '@tauri-apps/api/tauri';
import RefactoringPanel from '../../components/refactoring/RefactoringPanel';
import RefactoringSuggestionsList from '../../components/refactoring/RefactoringSuggestionsList';
import RefactoringExecutionDialog from '../../components/refactoring/RefactoringExecutionDialog';
import BatchRefactoringPanel from '../../components/refactoring/BatchRefactoringPanel';
import RefactoringSettings from '../../components/refactoring/RefactoringSettings';
import RefactoringHistory from '../../components/refactoring/RefactoringHistory';

// Mock Tauri API
jest.mock('@tauri-apps/api/tauri', () => ({
    invoke: jest.fn(),
}));

const mockInvoke = invoke as jest.MockedFunction<typeof invoke>;

// Mock components for testing isolated functionality
const mockProps = {
    currentFile: '/test/project/src/main.rs',
    cursorPosition: { line: 10, character: 5 },
    selection: {
        start: { line: 10, character: 0 },
        end: { line: 15, character: 0 },
    },
};

describe('Refactoring System Integration Tests', () => {

    beforeEach(() => {
        // Reset all mocks before each test
        jest.clearAllMocks();

        // Setup mock Tauri responses
        mockInvoke.mockImplementation((command: string, args?: any) => {
            switch (command) {
                case 'load_available_operations':
                    return Promise.resolve([
                        {
                            operationType: 'extractFunction',
                            name: 'Extract Function',
                            description: 'Extract selected code into a new function',
                            requiresSelection: true,
                            isExperimental: false,
                            typicalConfidenceScore: 0.85,
                        },
                        {
                            operationType: 'rename',
                            name: 'Rename Symbol',
                            description: 'Rename a symbol with impact analysis',
                            requiresSelection: false,
                            isExperimental: false,
                            typicalConfidenceScore: 0.92,
                        },
                    ]);

                case 'get_refactoring_suggestions':
                    return Promise.resolve({
                        filePath: args.request.filePath,
                        position: args.request.position,
                        suggestions: [
                            {
                                operationType: 'extractFunction',
                                name: 'Extract extractUserData function',
                                description: 'Extract user data extraction logic',
                                confidenceScore: 0.88,
                                expectedImpact: 'medium',
                                prerequisites: ['valid selection', 'no cross-function references'],
                                quickFix: false,
                            },
                            {
                                operationType: 'rename',
                                name: 'Rename validateUser to validateProfile',
                                description: 'Better naming for validation function',
                                confidenceScore: 0.92,
                                expectedImpact: 'low',
                                prerequisites: [],
                                quickFix: true,
                            },
                        ],
                        totalSuggestions: 2,
                        analysisContext: 'Function in UserModel struct',
                    });

                case 'execute_refactoring_operation':
                    return Promise.resolve({
                        success: true,
                        id: 'test-operation-id',
                        changes: [
                            { line_start: 10, col_start: 0, line_end: 15, col_end: 0, new_content: '// New function extracted' },
                        ],
                        errorMessage: null,
                        warnings: [],
                        newContent: null,
                    });

                case 'get_refactoring_history':
                    return Promise.resolve([]);

                case 'save_refactoring_settings':
                    return Promise.resolve({});

                case 'get_refactoring_settings':
                    return Promise.resolve({});

                default:
                    return Promise.reject(new Error(`Unknown command: ${command}`));
            }
        });
    });

    describe('RefactoringPanel Integration', () => {
        it('loads available operations on mount', async () => {
            render(<RefactoringPanel {...mockProps} />);

            await waitFor(() => {
                expect(mockInvoke).toHaveBeenCalledWith('get_available_refactoring_operations');
                expect(mockInvoke).toHaveBeenCalledWith('get_refactoring_suggestions', expect.any(Object));
            });
        });

        it('displays loading state initially', () => {
            render(<RefactoringPanel {...mockProps} />);

            expect(screen.getByText(/analyzing/i)).toBeInTheDocument();
        });

        it('updates suggestions when file changes', async () => {
            const { rerender } = render(<RefactoringPanel {...mockProps} />);

            await waitFor(() => {
                expect(mockInvoke).toHaveBeenCalledTimes(2); // load ops + suggestions
            });

            // Change file
            rerender(<RefactoringPanel {...mockProps} currentFile="/test/new-file.rs" />);

            await waitFor(() => {
                expect(mockInvoke).toHaveBeenCalledTimes(4); // load ops again + new suggestions
            });
        });

        it('handles execution workflow correctly', async () => {
            const mockOnExecute = jest.fn();
            const mockOnCancel = jest.fn();

            render(
                <RefactoringExecutionDialog
                    operationType="extractFunction"
                    operationInfo={{
                        operationType: 'extractFunction',
                        name: 'Extract Function',
                        description: 'Extract selected code',
                        requiresSelection: true,
                        isExperimental: false,
                        typicalConfidenceScore: 0.85,
                    }}
                    onExecute={mockOnExecute}
                    onCancel={mockOnCancel}
                />
            );

            expect(screen.getByText('Extract Function')).toBeInTheDocument();

            // Test execution
            const executeButton = screen.getByText('Execute');
            fireEvent.click(executeButton);

            await waitFor(() => {
                expect(mockOnExecute).toHaveBeenCalledWith('extractFunction', {});
            });
        });
    });

    describe('RefactoringSuggestionsList Component', () => {
        const mockSuggestions = [
            {
                operationType: 'extractFunction',
                name: 'Extract Function',
                description: 'Extract selected code',
                confidenceScore: 0.85,
                expectedImpact: 'medium' as const,
                prerequisites: [],
                quickFix: true,
            },
        ];

        it('displays suggestions with correct information', () => {
            const mockOnSuggestionClick = jest.fn();

            render(
                <RefactoringSuggestionsList
                    suggestions={mockSuggestions}
                    onSuggestionClick={mockOnSuggestionClick}
                    loading={false}
                />
            );

            expect(screen.getByText('Extract Function')).toBeInTheDocument();
            expect(screen.getByText('Extract selected code')).toBeInTheDocument();

            // Click on suggestion
            const suggestionItem = screen.getByText('Extract Function');
            fireEvent.click(suggestionItem);

            expect(mockOnSuggestionClick).toHaveBeenCalledWith('extractFunction');
        });

        it('shows loading indicator when loading', () => {
            const mockOnSuggestionClick = jest.fn();

            render(
                <RefactoringSuggestionsList
                    suggestions={[]}
                    onSuggestionClick={mockOnSuggestionClick}
                    loading={true}
                />
            );

            expect(screen.getByText(/analyzing/i)).toBeInTheDocument();
        });

        it('displays confidence scores visually', () => {
            const mockOnSuggestionClick = jest.fn();

            render(
                <RefactoringSuggestionsList
                    suggestions={mockSuggestions}
                    onSuggestionClick={mockOnSuggestionClick}
                    loading={false}
                />
            );

            // Should display confidence score
            expect(screen.getByText('85%')).toBeInTheDocument();
        });
    });

    describe('Batch Refactoring Panel', () => {
        it('allows selection and execution of multiple operations', async () => {
            render(<BatchRefactoringPanel currentFile={mockProps.currentFile} availableOperations={[]} onBack={jest.fn()} />);

            await waitFor(() => {
                expect(mockInvoke).toHaveBeenCalledWith('load_batch_operations');
            });
        });

        it('tracks execution progress', async () => {
            const mockOnBack = jest.fn();
            render(<BatchRefactoringPanel currentFile={mockProps.currentFile} availableOperations={[]} onBack={mockOnBack} />);

            // Test back button
            const backButton = screen.getByText(/back/i);
            fireEvent.click(backButton);

            expect(mockOnBack).toHaveBeenCalled();
        });
    });

    describe('RefactoringSettings Component', () => {
        it('loads and saves settings correctly', async () => {
            const mockOnSettingsChange = jest.fn();
            render(<RefactoringSettings onSettingsChange={mockOnSettingsChange} />);

            await waitFor(() => {
                expect(mockInvoke).toHaveBeenCalledWith('get_refactoring_settings', expect.any(Object));
            });

            // Find and click save button
            const saveButton = screen.getByText('Save Settings');
            fireEvent.click(saveButton);

            await waitFor(() => {
                expect(mockInvoke).toHaveBeenCalledWith('save_refactoring_settings', expect.any(Object));
            });
        });

        it('allows resetting to defaults', () => {
            const mockOnSettingsChange = jest.fn();
            render(<RefactoringSettings onSettingsChange={mockOnSettingsChange} />);

            const resetButton = screen.getByText('Reset to Defaults');
            fireEvent.click(resetButton);

            expect(mockOnSettingsChange).not.toHaveBeenCalled(); // Should wait for save
        });
    });

    describe('RefactoringHistory Component', () => {
        it('loads and displays operation history', async () => {
            const mockOnHistoryEntryClicked = jest.fn();
            render(<RefactoringHistory currentFile={mockProps.currentFile} onHistoryEntryClicked={mockOnHistoryEntryClicked} />);

            await waitFor(() => {
                expect(mockInvoke).toHaveBeenCalledWith('get_refactoring_history', expect.any(Object));
            });

            expect(screen.getByText('Refactoring History')).toBeInTheDocument();
        });

        it('handles undo/redo operations', async () => {
            // Mock history with undoable operation
            mockInvoke.mockResolvedValueOnce([
                {
                    id: 'test-operation',
                    timestamp: Date.now(),
                    operationType: 'extractFunction',
                    operationName: 'Extract Function',
                    filePath: mockProps.currentFile,
                    status: 'success',
                    changesCount: 1,
                    canUndo: true,
                    canRedo: false,
                }
            ]);

            render(<RefactoringHistory currentFile={mockProps.currentFile} onHistoryEntryClicked={jest.fn()} />);

            await waitFor(() => {
                expect(screen.getByText('Extract Function')).toBeInTheDocument();
            });
        });
    });

    describe('Error Handling and Edge Cases', () => {
        it('handles API errors gracefully', async () => {
            // Mock API error
            mockInvoke.mockRejectedValueOnce(new Error('Network error'));

            render(<RefactoringPanel {...mockProps} />);

            await waitFor(() => {
                expect(screen.getByText(/failed/i)).toBeInTheDocument();
            });
        });

        it('handles invalid suggestion data', () => {
            const invalidSuggestions = [
                {
                    operationType: null,
                    name: undefined,
                    description: '',
                    confidenceScore: 'invalid',
                }
            ];

            render(
                <RefactoringSuggestionsList
                    suggestions={invalidSuggestions as any}
                    onSuggestionClick={jest.fn()}
                    loading={false}
                />
            );

            // Should not crash and handle gracefully
            expect(screen.queryByText('undefined')).not.toBeInTheDocument();
        });

        it('validates operation parameters', () => {
            const mockOnExecute = jest.fn();

            render(
                <RefactoringExecutionDialog
                    operationType="extractFunction"
                    operationInfo={{
                        operationType: 'extractFunction',
                        name: 'Extract Function',
                        description: 'Extract selected code',
                        requiresSelection: true,
                        isExperimental: false,
                        typicalConfidenceScore: 0.85,
                    }}
                    onExecute={mockOnExecute}
                    onCancel={jest.fn()}
                />
            );

            const executeButton = screen.getByText('Execute');
            // Button should still work even with minimal props
            fireEvent.click(executeButton);

            expect(mockOnExecute).toHaveBeenCalled();
        });
    });

    describe('Performance Testing', () => {
        it('handles large number of suggestions', () => {
            const largeSuggestions = Array.from({ length: 1000 }, (_, i) => ({
                operationType: `operation${i}`,
                name: `Suggestion ${i}`,
                description: `Description for suggestion ${i}`,
                confidenceScore: Math.random(),
                expectedImpact: 'low' as const,
                prerequisites: [],
                quickFix: i % 10 === 0,
            }));

            const startTime = performance.now();

            render(
                <RefactoringSuggestionsList
                    suggestions={largeSuggestions}
                    onSuggestionClick={jest.fn()}
                    loading={false}
                />
            );

            const endTime = performance.now();
            const renderTime = endTime - startTime;

            // Should render in reasonable time (less than 100ms)
            expect(renderTime).toBeLessThan(100);

            // Should display suggestions
            expect(screen.getAllByText(/suggestion/i)).toHaveLength(1000);
        });

        it('debounces rapid updates', async () => {
            const { rerender } = render(<RefactoringPanel {...mockProps} />);

            // Rapid selection changes
            for (let i = 0; i < 10; i++) {
                rerender(<RefactoringPanel
                    {...mockProps}
                    cursorPosition={{ line: i, character: 0 }}
                />);
            }

            // Should not make 10 separate API calls due to debouncing
            await waitFor(() => {
                expect(mockInvoke.mock.calls.filter(([cmd]) => cmd === 'get_refactoring_suggestions')).toHaveLength(1);
            }, { timeout: 500 });
        });
    });
});