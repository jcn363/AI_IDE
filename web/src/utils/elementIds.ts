/**
 * Utilities for generating consistent, unique element IDs across the application
 */

/**
 * Base ID for different component types
 */
export const ID_PREFIXES = {
  CARGO_COMMANDS: 'cargo-cmd',
  DEPENDENCY_TABLE: 'dep-table',
  FORM_INPUT: 'form-input',
  MODAL_OVERLAY: 'modal-overlay',
  BUTTON_ACTION: 'btn-action',
  LIST_ITEM: 'list-item',
  GRAPH_NODE: 'graph-node',
  VALIDATION_ERROR: 'validation-err',
  TAB_PANEL: 'tab-panel',
  COMMAND_OUTPUT: 'cmd-output'
} as const;

/**
 * Generates a unique ID for cargo command elements
 */
export function generateCargoCommandId(action: string, index?: number): string {
  const suffix = index !== undefined ? `-${index}` : '';
  return `${ID_PREFIXES.CARGO_COMMANDS}-${action}${suffix}`;
}

/**
 * Generates a unique ID for dependency table elements
 */
export function generateDependencyTableId(tableName: string, row?: number, column?: string): string {
  let id = `${ID_PREFIXES.DEPENDENCY_TABLE}-${tableName}`;
  if (row !== undefined) id += `-${row}`;
  if (column) id += `-${column}`;
  return id;
}

/**
 * Generates a unique ID for form input elements
 */
export function generateFormInputId(formName: string, fieldName: string): string {
  return `${ID_PREFIXES.FORM_INPUT}-${formName}-${fieldName}`;
}

/**
 * Generates a unique ID for modal overlays and dialogs
 */
export function generateModalId(modalName: string, element = 'overlay'): string {
  return `${ID_PREFIXES.MODAL_OVERLAY}-${modalName}-${element}`;
}

/**
 * Generates a unique ID for action buttons
 */
export function generateButtonId(action: string, target?: string): string {
  return `${ID_PREFIXES.BUTTON_ACTION}-${action}${target ? `-${target}` : ''}`;
}

/**
 * Generates a unique ID for list items
 */
export function generateListItemId(listName: string, index: number): string {
  return `${ID_PREFIXES.LIST_ITEM}-${listName}-${index}`;
}

/**
 * Generates a unique ID for graph nodes
 */
export function generateGraphNodeId(graphType: string, nodeKey: string): string {
  return `${ID_PREFIXES.GRAPH_NODE}-${graphType}-${nodeKey}`;
}

/**
 * Generates a unique ID for validation error elements
 */
export function generateValidationErrorId(fieldName: string): string {
  return `${ID_PREFIXES.VALIDATION_ERROR}-${fieldName}`;
}

/**
 * Generates a unique ID for tab panel elements
 */
export function generateTabPanelId(panelName: string, tabIndex?: number): string {
  return `${ID_PREFIXES.TAB_PANEL}-${panelName}${tabIndex !== undefined ? `-${tabIndex}` : ''}`;
}

/**
 * Generates a unique ID for command output elements
 */
export function generateCommandOutputId(commandName: string, outputType?: string): string {
  return `${ID_PREFIXES.COMMAND_OUTPUT}-${commandName}${outputType ? `-${outputType}` : ''}`;
}

/**
 * Generates accessible ARIA label for form fields
 */
export function generateAriaLabel(fieldName: string, context?: string): string {
  const readableName = fieldName
    .replace(/([A-Z])/g, ' $1')
    .replace(/_/g, ' ')
    .toLowerCase()
    .trim();
  return context ? `${readableName} in ${context}` : readableName;
}

/**
 * Generates a unique HTML name attribute for form elements
 */
export function generateFormElementName(formName: string, fieldName: string): string {
  return `form-${formName}-${fieldName}`;
}

/**
 * Generates an ID for related form elements (label-input pairs)
 */
export function generateFormAssociationIds(formName: string, fieldName: string): {
  labelId: string;
  inputId: string;
  errorId: string;
} {
  const inputId = generateFormInputId(formName, fieldName);
  const labelId = `label-${inputId}`;
  const errorId = generateValidationErrorId(fieldName);

  return { labelId, inputId, errorId };
}

export default {
  ID_PREFIXES,
  generateCargoCommandId,
  generateDependencyTableId,
  generateFormInputId,
  generateModalId,
  generateButtonId,
  generateListItemId,
  generateGraphNodeId,
  generateValidationErrorId,
  generateTabPanelId,
  generateCommandOutputId,
  generateAriaLabel,
  generateFormElementName,
  generateFormAssociationIds,
};