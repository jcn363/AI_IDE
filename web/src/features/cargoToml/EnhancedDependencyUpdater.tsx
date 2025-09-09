import React, { useCallback, useEffect, useMemo, useState } from 'react';
import { Badge, Button, Card, Checkbox, Input, Select, Space, Spin, Switch, Table, Tag, Tooltip, Typography, message } from 'antd';
import { InfoCircleOutlined, SearchOutlined, SyncOutlined } from '@ant-design/icons';
import { invoke } from '@tauri-apps/api/core';
import { DependencyUpdateInfo } from './types/dependencyUpdate';

const { Text } = Typography;
const { Option } = Select;

type UpdateType = 'major' | 'minor' | 'patch' | 'unknown';

interface EnhancedDependencyUpdaterProps {
  projectPath: string;
  onUpdateDependency: (updates: Array<{ name: string; version: string }>) => Promise<void>;
}

const getUpdateType = (current: string, latest: string): UpdateType => {
  try {
    const currentParts = current.split('.').map(Number);
    const latestParts = latest.split('.').map(Number);
    
    if (currentParts[0] < latestParts[0]) return 'major';
    if (currentParts[1] < latestParts[1]) return 'minor';
    if (currentParts[2] < latestParts[2]) return 'patch';
    return 'unknown';
  } catch (e) {
    return 'unknown';
  }
};

export const EnhancedDependencyUpdater: React.FC<EnhancedDependencyUpdaterProps> = ({
  projectPath,
  onUpdateDependency,
}) => {
  const [state, setState] = useState({
    updates: [] as DependencyUpdateInfo[],
    loading: false,
    selectedUpdates: new Set<string>(),
    searchText: '',
    filterType: 'all',
    showOnlyDirect: false,
    isUpdating: false,
  });

  const { 
    updates, 
    loading, 
    selectedUpdates, 
    searchText, 
    filterType, 
    showOnlyDirect, 
    isUpdating, 
  } = state;

  const filteredUpdates = useMemo(() => {
    return updates.filter(update => {
      // Filter by search text
      const matchesSearch = !searchText || 
        update.name.toLowerCase().includes(searchText.toLowerCase()) ||
        update.usedIn.some(pkg => pkg.toLowerCase().includes(searchText.toLowerCase()));
      
      // Filter by update type
      const matchesType = filterType === 'all' || update.updateType === filterType;
      
      // Filter direct dependencies if needed
      const matchesDirect = !showOnlyDirect || update.isDirect;
      
      return matchesSearch && matchesType && matchesDirect;
    });
  }, [updates, searchText, filterType, showOnlyDirect]);

  const fetchUpdates = useCallback(async () => {
    if (!projectPath) return;
    
    setState(prev => ({ ...prev, loading: true }));
    try {
      const updates = await invoke('check_dependency_updates', { projectPath }) as DependencyUpdateInfo[];
      const processedUpdates = updates.map(update => ({
        ...update,
        updateType: getUpdateType(update.currentVersion, update.latestVersion),
      }));
      setState(prev => ({ ...prev, updates: processedUpdates }));
    } catch (error) {
      console.error('Failed to fetch updates:', error);
      message.error('Failed to fetch dependency updates');
    } finally {
      setState(prev => ({ ...prev, loading: false }));
    }
  }, [projectPath]);

  useEffect(() => {
    fetchUpdates();
  }, [fetchUpdates]);

  const toggleUpdate = (name: string) => {
    setState(prev => {
      const newSelection = new Set(prev.selectedUpdates);
      if (newSelection.has(name)) {
        newSelection.delete(name);
      } else {
        newSelection.add(name);
      }
      return { ...prev, selectedUpdates: newSelection };
    });
  };

  const applyUpdates = async () => {
    if (selectedUpdates.size === 0) {
      message.warning('Please select at least one dependency to update');
      return;
    }

    setState(prev => ({ ...prev, isUpdating: true }));
    try {
      const updatesToApply = updates
        .filter(update => selectedUpdates.has(update.name))
        .map(({ name, latestVersion }) => ({ name, version: latestVersion }));
      
      await onUpdateDependency(updatesToApply);
      message.success('Dependencies updated successfully');
      setState(prev => ({ ...prev, selectedUpdates: new Set<string>() }));
      await fetchUpdates();
    } catch (error) {
      console.error('Failed to update dependencies:', error);
      message.error('Failed to update dependencies');
    } finally {
      setState(prev => ({ ...prev, isUpdating: false }));
    }
  };

  const columns = [
    {
      title: 'Dependency',
      dataIndex: 'name',
      key: 'name',
      render: (_: any, record: DependencyUpdateInfo) => (
        <div>
          <Checkbox 
            checked={selectedUpdates.has(record.name)}
            onChange={() => toggleUpdate(record.name)}
            style={{ marginRight: 8 }}
          />
          <span style={{ marginRight: 8 }}>{record.name}</span>
          {record.isDirect && (
            <Tooltip title="Direct dependency">
              <InfoCircleOutlined style={{ color: '#1890ff' }} />
            </Tooltip>
          )}
          <div style={{ fontSize: 12, color: '#666', marginTop: 4 }}>
            Used in: {record.usedIn.join(', ')}
          </div>
        </div>
      ),
    },
    {
      title: 'Current',
      dataIndex: 'currentVersion',
      key: 'currentVersion',
      render: (text: string) => <Text delete>{text}</Text>,
    },
    {
      title: 'Latest',
      dataIndex: 'latestVersion',
      key: 'latestVersion',
      render: (text: string, record: DependencyUpdateInfo) => (
        <Tag color={
          record.updateType === 'major' ? 'red' : 
          record.updateType === 'minor' ? 'orange' : 'green'
        }>
          {text}
        </Tag>
      ),
    },
    {
      title: 'Type',
      dataIndex: 'updateType',
      key: 'updateType',
      render: (text: UpdateType) => (
        <Tag color={
          text === 'major' ? 'red' : 
          text === 'minor' ? 'orange' : 'green'
        }>
          {text.toUpperCase()}
        </Tag>
      ),
    },
    {
      title: 'Actions',
      key: 'actions',
      render: (_: any, record: DependencyUpdateInfo) => (
        <Space size="middle">
          {record.changelogUrl && (
            <a href={record.changelogUrl} target="_blank" rel="noopener noreferrer">
              Changelog
            </a>
          )}
        </Space>
      ),
    },
  ];

  return (
    <Card 
      title={
        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
          <span>Dependency Updates</span>
          <Button 
            icon={<SyncOutlined spin={loading} />} 
            onClick={fetchUpdates}
            disabled={loading}
          >
            Refresh
          </Button>
        </div>
      }
      extra={
        <Space>
          <Input
            prefix={<SearchOutlined />}
            placeholder="Search dependencies..."
            value={searchText}
            onChange={e => setState(prev => ({ ...prev, searchText: (e.target as any).value }))}
            style={{ width: 200 }}
            allowClear
          />
          <Select 
            value={filterType} 
            onChange={value => setState(prev => ({ ...prev, filterType: value }))} 
            style={{ width: 150 }}
          >
            <Option value="all">All Updates</Option>
            <Option value="major">Major</Option>
            <Option value="minor">Minor</Option>
            <Option value="patch">Patch</Option>
          </Select>
          <Tooltip title="Show only direct dependencies">
            <Badge count={updates.filter(u => u.isDirect).length}>
              <Switch 
                checkedChildren="Direct" 
                unCheckedChildren="All" 
                checked={showOnlyDirect}
                onChange={checked => setState(prev => ({ ...prev, showOnlyDirect: checked }))}
              />
            </Badge>
          </Tooltip>
          <Button 
            type="primary" 
            onClick={applyUpdates}
            loading={isUpdating}
            disabled={selectedUpdates.size === 0}
          >
            Update Selected ({selectedUpdates.size})
          </Button>
        </Space>
      }
    >
      <Spin spinning={loading}>
        <Table
          dataSource={filteredUpdates}
          columns={columns}
          rowKey="name"
          pagination={{ pageSize: 10 }}
          rowClassName={record => (selectedUpdates.has(record.name) ? 'selected-row' : '')}
        />
      </Spin>
      <style jsx global>{`
        .selected-row {
          background-color: #e6f7ff !important;
        }
      `}</style>
    </Card>
  );
};
