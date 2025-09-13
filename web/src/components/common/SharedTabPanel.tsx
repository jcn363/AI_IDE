import React from 'react';
import { Box } from '@mui/material';
import { BaseComponentProps } from '../../utils/consolidated';

/**
 * Props for SharedTabPanel component
 */
export interface SharedTabPanelProps extends BaseComponentProps {
  /** Children content to render */
  children?: React.ReactNode;
  /** Current active tab index */
  index: number;
  /** This tab's index value */
  value: number;
  /** Optional custom CSS class for the hidden panel */
  panelClassName?: string;
}

/**
 * SharedTabPanel - A reusable tab panel component
 *
 * Replaces the duplicate TabPanel implementations found in:
 * - PerformanceDashboard.tsx
 * - VersionAlignmentView.tsx
 * - AIFeaturesDemo.tsx
 * - CargoPanel.tsx (inline)
 *
 * Provides consistent tab panel behavior with proper accessibility.
 */
export const SharedTabPanel: React.FC<SharedTabPanelProps> = ({
  children,
  value,
  index,
  className,
  panelClassName,
  'data-testid': testId,
  ...other
}) => {
  return (
    <div
      role="tabpanel"
      hidden={value !== index}
      id={`shared-tabpanel-${index}`}
      aria-labelledby={`shared-tab-${index}`}
      className={panelClassName}
      data-testid={testId}
      {...other}
    >
      {value === index && <Box sx={{ pt: 2 }}>{children}</Box>}
    </div>
  );
};

/**
 * Accessibility helper for tab components
 * @param index - Tab index
 * @returns Object with accessibility properties
 */
export const createTabA11yProps = (index: number) => ({
  id: `shared-tab-${index}`,
  'aria-controls': `shared-tabpanel-${index}`,
});

/**
 * SharedTabsHelper - Helper component to create tabs with a11y
 */
export const SharedTabsHelper = {
  /**
   * Creates accessibility props for tabs consistently
   */
  getA11yProps: createTabA11yProps,

  /**
   * Generates tab items from configuration
   * @param tabs - Array of tab configurations
   * @param renderTab - Function to render each tab
   * @returns Array of rendered tab items
   */
  renderTabs: <T extends { label: string; disabled?: boolean }>(
    tabs: T[],
    renderTab: (tab: T, index: number) => React.ReactNode
  ) =>
    tabs.map((tab, index) => (
      <React.Fragment key={`${tab.label}-${index}`}>{renderTab(tab, index)}</React.Fragment>
    )),
};

export default SharedTabPanel;
