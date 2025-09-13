import { useState, useCallback } from 'react';

export interface TabConfig {
  id: string;
  label: string;
  content: React.ReactNode;
  disabled?: boolean;
  icon?: React.ReactNode;
}

/**
 * Hook for managing tab state and navigation
 */
export function useTabManager(initialTab = 0) {
  const [activeTab, setActiveTab] = useState<number>(initialTab);

  const changeTab = useCallback((event: React.SyntheticEvent, newValue: number) => {
    setActiveTab(newValue);
  }, []);

  const goToTab = useCallback((tabIndex: number) => {
    setActiveTab(tabIndex);
  }, []);

  const nextTab = useCallback((totalTabs: number) => {
    setActiveTab((prev) => Math.min(prev + 1, totalTabs - 1));
  }, []);

  const prevTab = useCallback(() => {
    setActiveTab((prev) => Math.max(prev - 1, 0));
  }, []);

  const resetTab = useCallback(() => {
    setActiveTab(initialTab);
  }, [initialTab]);

  return {
    activeTab,
    setActiveTab,
    changeTab,
    goToTab,
    nextTab,
    prevTab,
    resetTab,
  };
}

/**
 * Hook for managing tab state with tab-specific data storage
 */
export function useTabManagerWithData<T = any>(initialTab = 0) {
  const [activeTab, setActiveTab] = useState<number>(initialTab);
  const [tabData, setTabData] = useState<Record<string, T>>({});

  const changeTab = useCallback((event: React.SyntheticEvent, newValue: number) => {
    setActiveTab(newValue);
  }, []);

  const goToTab = useCallback((tabIndex: number) => {
    setActiveTab(tabIndex);
  }, []);

  const updateTabData = useCallback((tabKey: string, data: T) => {
    setTabData((prev) => ({
      ...prev,
      [tabKey]: data,
    }));
  }, []);

  const getTabData = useCallback(
    (tabKey: string): T | undefined => {
      return tabData[tabKey];
    },
    [tabData]
  );

  const modifyTabData = useCallback((tabKey: string, updater: (prevData: T | undefined) => T) => {
    setTabData((prev) => ({
      ...prev,
      [tabKey]: updater(prev[tabKey]),
    }));
  }, []);

  const clearTabData = useCallback((tabKey: string) => {
    setTabData((prev) => {
      const newData = { ...prev };
      delete newData[tabKey];
      return newData;
    });
  }, []);

  return {
    activeTab,
    setActiveTab,
    changeTab,
    goToTab,
    updateTabData,
    getTabData,
    modifyTabData,
    clearTabData,
    tabData,
  };
}

/**
 * Hook for managing tab history/navigation state
 */
export function useTabNavigation(initialTab = 0, historySize = 10) {
  const [activeTab, setActiveTab] = useState<number>(initialTab);
  const [tabHistory, setTabHistory] = useState<number[]>([initialTab]);

  const changeTab = useCallback(
    (event: React.SyntheticEvent, newValue: number) => {
      setActiveTab(newValue);
      setTabHistory((prev) => {
        const newHistory = [...prev, newValue];
        if (newHistory.length > historySize) {
          newHistory.shift();
        }
        return newHistory;
      });
    },
    [historySize]
  );

  const goBack = useCallback(() => {
    if (tabHistory.length > 1) {
      const newHistory = [...tabHistory];
      newHistory.pop();
      const prevTab = newHistory[newHistory.length - 1];
      setActiveTab(prevTab);
      setTabHistory(newHistory);
      return true;
    }
    return false;
  }, [tabHistory]);

  const canGoBack = tabHistory.length > 1;

  return {
    activeTab,
    setActiveTab,
    changeTab,
    goBack,
    canGoBack,
    tabHistory,
  };
}
