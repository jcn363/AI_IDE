import React, { useCallback, useState, useMemo } from 'react';
import {
    Box,
    CircularProgress,
    IconButton,
    List,
    ListItem,
    ListItemText,
    Typography,
    Theme,
    SxProps,
    useTheme,
} from '@mui/material';
import { SxProps as SystemSxProps } from '@mui/system';
import ExpandMoreIcon from '@mui/icons-material/ExpandMore';
import ChevronRightIcon from '@mui/icons-material/ChevronRight';
import { DebuggerService } from '../services/debuggerService';

/**
 * Represents a node in the variable tree structure
 */
export interface VariableNode {
    name: string;
    value: string;
    type: string;
    children?: VariableNode[];
}

/**
 * Represents a variable in the debugger
 */
export interface Variable {
    name: string;
    value: string;
    type_name: string;
}

/**
 * Props for the VariablesList component
 */
export interface VariablesListProps {
    /** Array of variables to display */
    variables: Variable[];
    /** Whether the component is in a loading state */
    isLoading?: boolean;
    /** Error message to display */
    error?: string | null;
    /** Callback when an error occurs */
    onError?: (error: string) => void;
    /** Callback when a variable is expanded or collapsed */
    onVariableExpand?: (variable: Variable, isExpanded: boolean) => void;
    /** Additional CSS class name */
    className?: string;
    /** Additional styles */
    style?: React.CSSProperties;
    /** Maximum height of the variables list */
    maxHeight?: number | string;
    /** Whether to show variable types */
    showTypes?: boolean;
    /** Custom renderer for variable values */
    renderVariableValue?: (value: string, type: string) => React.ReactNode;
}

// Memoize styles to prevent unnecessary re-renders
const useStyles = (theme: Theme) => ({
    listItem: {
        '&:hover': {
            bgcolor: 'action.hover',
        },
        py: 0.5,
        transition: theme.transitions.create('background-color', {
            duration: theme.transitions.duration.shortest,
        }),
    },
    variableName: {
        fontFamily: 'monospace',
        fontWeight: 500,
    },
    variableValue: {
        fontFamily: 'monospace',
    },
    variableType: {
        fontSize: '0.75rem',
        opacity: 0.7,
        ml: 1,
    },
    loadingContainer: {
        display: 'flex',
        justifyContent: 'center',
        alignItems: 'center',
        height: 100,
    },
    errorContainer: {
        p: 2,
        color: 'error.main',
        bgcolor: 'error.light',
        borderRadius: 1,
    },
    emptyState: {
        p: 2,
        textAlign: 'center',
        color: 'text.secondary',
    },
    listContainer: (maxHeight: string | number) => ({
        maxHeight,
        overflow: 'auto',
        border: '1px solid',
        borderColor: 'divider',
        borderRadius: 1,
    }),
});

export const VariablesList: React.FC<VariablesListProps> = ({
    variables = [],
    isLoading = false,
    error: externalError = null,
    onError,
    onVariableExpand,
    className = '',
    style = {},
    maxHeight = 300,
    showTypes = true,
    renderVariableValue,
}) => {
    const theme = useTheme();
    const styles = useStyles(theme);
    // State management with useReducer might be better for complex state logic
    type ComponentState = {
        expanded: Record<string, boolean>;
        varObjects: Record<string, unknown>;
        childrenMap: Record<string, VariableNode[]>;
        error: string | null;
        loadingVars: Record<string, boolean>;
    };

    const [state, setState] = useState<ComponentState>({
        expanded: {},
        varObjects: {},
        childrenMap: {},
        error: null,
        loadingVars: {},
    });

    // Memoize derived state
    const { expanded, varObjects, childrenMap, error, loadingVars } = state;

    // Update state helper to reduce re-renders
    const updateState = useCallback((updates: Partial<ComponentState>) => {
        setState((prev: ComponentState) => ({
            ...prev,
            ...updates,
        }));
    }, []);

    const handleError = useCallback(
        (err: unknown, context: string) => {
            const errorMessage = err instanceof Error ? err.message : String(err);
            const fullMessage = `${context}: ${errorMessage}`;
            console.error(fullMessage, err);
            updateState({ error: fullMessage });
            if (onError) {
                onError(fullMessage);
            }
        },
        [onError, updateState]
    );

    const fetchVariableChildren = useCallback(
        async (varName: string, varObj: unknown) => {
            try {
                updateState({
                    loadingVars: { ...loadingVars, [varName]: true }
                });

                const obj = varObj || (await DebuggerService.varCreate(varName));
                
                if (!varObj) {
                    updateState({
                        varObjects: { ...varObjects, [varName]: obj }
                    });
                }

                // Ensure we're passing the correct type to varChildren
                // Convert the object to a string if needed
                const children = await DebuggerService.varChildren(
                    typeof obj === 'string' ? obj : JSON.stringify(obj), 
                    true
                ) as unknown as VariableNode[];
                return children;
            } catch (err) {
                const error = new Error(
                    `Failed to fetch variable children: ${err instanceof Error ? err.message : String(err)}`
                );
                handleError(error, 'Variable expansion failed');
                throw error;
            } finally {
                updateState({
                    loadingVars: { ...loadingVars, [varName]: false }
                });
            }
        },
        [handleError, loadingVars, updateState, varObjects]
    );

    const updateChildrenMap = useCallback((varName: string, children: VariableNode[]) => {
        updateState({
            childrenMap: {
                ...childrenMap,
                [varName]: children,
            }
        });
    }, [childrenMap, updateState]);
    
    const handleToggleExpand = useCallback(
        async (variable: Variable) => {
            const varName = variable.name;
            const isExpanded = expanded[varName];
            const varObj = varObjects[varName];

            try {
                updateState({
                    loadingVars: { ...loadingVars, [varName]: true },
                    error: null
                });

                if (!isExpanded && !childrenMap[varName]) {
                    const children = await fetchVariableChildren(varName, varObj);
                    updateChildrenMap(varName, children);
                }

                updateState({
                    expanded: {
                        ...expanded,
                        [varName]: !isExpanded,
                    }
                });

                onVariableExpand?.(variable, !isExpanded);
            } catch (err) {
                handleError(err, 'Error expanding variable');
            } finally {
                updateState({
                    loadingVars: { ...loadingVars, [varName]: false }
                });
            }
        },
        [expanded, varObjects, childrenMap, fetchVariableChildren, handleError, loadingVars, onVariableExpand, updateState]
    );

    const defaultRenderVariableValue = useCallback((value: string, type: string) => {
        return (
            <Box component="span" display="inline-flex" alignItems="center">
                <Box component="span" sx={styles.variableValue}>
                    {value}
                </Box>
                {showTypes && (
                    <Box component="span" sx={styles.variableType}>
                        {type}
                    </Box>
                )}
            </Box>
        );
    }, [showTypes, styles]);

    const renderVariableContent = useCallback(
        (value: string, type: string) => {
            return renderVariableValue
                ? renderVariableValue(value, type)
                : defaultRenderVariableValue(value, type);
        },
        [defaultRenderVariableValue, renderVariableValue]
    );

    const renderVariable = useCallback((variable: Variable, depth = 0): React.ReactNode => {
        const { name, value, type_name } = variable;
        const isExpanded = expanded[name] || false;
        const children = childrenMap[name] || [];
        const isLoading = loadingVars[name] || false;
        const hasChildren = children.length > 0;

        return (
            <Box component="li" key={name} role="treeitem" aria-expanded={hasChildren ? isExpanded : undefined}>
                <ListItem
                    component="div"
                    onClick={hasChildren ? () => handleToggleExpand(variable) : undefined}
                    sx={{
                        ...styles.listItem,
                        pl: 2 + depth * 2,
                        borderLeft: (theme: Theme) => `2px solid ${theme.palette.divider}`,
                        cursor: hasChildren ? 'pointer' : 'default',
                        opacity: isLoading ? 0.7 : 1,
                        pointerEvents: isLoading ? 'none' : 'auto',
                        '&:hover': {
                            backgroundColor: hasChildren ? 'action.hover' : 'transparent'
                        },
                        display: 'flex',
                        alignItems: 'center',
                        width: '100%',
                        boxSizing: 'border-box',
                        minHeight: '48px',
                        padding: '8px 16px',
                        textDecoration: 'none',
                        position: 'relative'
                    }}
                >
                    <ListItemText
                        primary={
                            <Box component="span" display="flex" alignItems="center">
                                <Box component="span" sx={styles.variableName}>
                                    {name}:
                                </Box>{' '}
                                <Box component="span" sx={{ ml: 1, flex: 1 }}>
                                    {renderVariableContent(value, type_name)}
                                </Box>
                            </Box>
                        }
                        sx={{ my: 0 }}
                    />
                    {hasChildren && (
                        <IconButton
                            size="small"
                            edge="end"
                            onClick={(e) => {
                                e.stopPropagation();
                                handleToggleExpand(variable);
                            }}
                            disabled={isLoading}
                            sx={{ ml: 1 }}
                        >
                            {isLoading ? (
                                <CircularProgress size={20} />
                            ) : isExpanded ? (
                                <ExpandMoreIcon fontSize="small" />
                            ) : (
                                <ChevronRightIcon fontSize="small" />
                            )}
                        </IconButton>
                    )}
                </ListItem>
                {hasChildren && isExpanded && (
                    <Box component="ul" sx={{ pl: 0, m: 0, listStyle: 'none' }} role="group">
                        {children.map((child) => (
                            <ListItem 
                                key={`${name}-${child.name}`} 
                                sx={styles.listItem}
                                component="li"
                                role="treeitem"
                            >
                                <ListItemText
                                    primary={
                                        <Box component="span" display="flex" alignItems="center">
                                            <Box component="span" sx={styles.variableName}>
                                                {child.name}:
                                            </Box>{' '}
                                            <Box component="span" sx={{ ml: 1, flex: 1 }}>
                                                {renderVariableContent(child.value, child.type)}
                                            </Box>
                                        </Box>
                                    }
                                    sx={{ my: 0 }}
                                />
                            </ListItem>
                        ))}
                    </Box>
                )}
            </Box>
        );
    }, [expanded, childrenMap, loadingVars, handleToggleExpand, renderVariableValue, styles]);

    // Loading state
    if (isLoading) {
        return (
            <Box
                sx={[styles.loadingContainer, style] as SystemSxProps<Theme>}
                className={className}
                role="status"
                aria-live="polite"
                aria-busy="true"
            >
                <CircularProgress size={24} aria-label="Loading variables" />
            </Box>
        );
    }

    // Error state
    const displayError = error || externalError;
    if (displayError) {
        return (
            <Box
                sx={[styles.errorContainer, style] as SystemSxProps<Theme>}
                className={className}
                role="alert"
                aria-live="assertive"
            >
                <Typography variant="body2">{displayError}</Typography>
            </Box>
        );
    }

    // Empty state
    if (!variables?.length) {
        return (
            <Box
                sx={[styles.emptyState, style] as SystemSxProps<Theme>}
                className={className}
                aria-live="polite"
            >
                <Typography variant="body2">No variables available</Typography>
            </Box>
        );
    }

    // Memoize the list container styles
    const listContainerStyles = useMemo(
        () => [styles.listContainer(maxHeight), style],
        [maxHeight, style, styles]
    );

    // Memoize rendered variables to prevent unnecessary re-renders
    const renderedVariables = useMemo(
        () => variables.map((variable) => (
            <React.Fragment key={variable.name}>
                {renderVariable(variable)}
            </React.Fragment>
        )),
        [variables, renderVariable]
    );

    return (
        <List
            dense
            disablePadding
            sx={listContainerStyles as SystemSxProps<Theme>}
            className={className}
            aria-label="Variables list"
            role="tree"
        >
            {renderedVariables}
        </List>
    );
};
