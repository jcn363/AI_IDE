import React, { useState, useMemo, useCallback, useEffect } from 'react';
import { Card, Table, Input, Space, Button, Tag, Tooltip, Tabs, Spin } from 'antd';
import { SearchOutlined, InfoCircleOutlined } from '@ant-design/icons';
import { invoke } from '@tauri-apps/api/core';

interface DependencyNode {
  name: string;
  version: string;
  dependencies?: string[];
  isDirect: boolean;
}

export const CargoLockVisualization: React.FC<{ projectPath: string }> = ({ projectPath }) => {
  const [lockData, setLockData] = useState<DependencyNode[]>([]);
  const [searchText, setSearchText] = useState('');
  const [loading, setLoading] = useState(false);
  const [showOnlyDirect, setShowOnlyDirect] = useState(false);

  const loadLockData = useCallback(async () => {
    setLoading(true);
    try {
      const data = await invoke<DependencyNode[]>('parse_cargo_lock', { projectPath });
      setLockData(data);
    } catch (error) {
      console.error('Failed to load Cargo.lock:', error);
    } finally {
      setLoading(false);
    }
  }, [projectPath]);

  useEffect(() => {
    loadLockData();
  }, [loadLockData]);

  const filteredDependencies = useMemo(() => {
    return lockData.filter((dep) => {
      const matchesSearch =
        !searchText ||
        dep.name.toLowerCase().includes(searchText.toLowerCase()) ||
        dep.version.toLowerCase().includes(searchText.toLowerCase());
      const matchesDirectFilter = !showOnlyDirect || dep.isDirect;
      return matchesSearch && matchesDirectFilter;
    });
  }, [lockData, searchText, showOnlyDirect]);

  const columns = [
    {
      title: 'Dependency',
      dataIndex: 'name',
      key: 'name',
      render: (text: string, record: DependencyNode) => (
        <div>
          <span style={{ marginRight: 8 }}>{text}</span>
          {record.isDirect && (
            <Tooltip title="Direct dependency">
              <InfoCircleOutlined style={{ color: '#1890ff' }} />
            </Tooltip>
          )}
        </div>
      ),
    },
    {
      title: 'Version',
      dataIndex: 'version',
      key: 'version',
    },
    {
      title: 'Dependencies',
      dataIndex: 'dependencies',
      key: 'dependencies',
      render: (deps: string[] = []) => (
        <div
          style={{
            maxWidth: 300,
            whiteSpace: 'nowrap',
            overflow: 'hidden',
            textOverflow: 'ellipsis',
          }}
        >
          {deps.length > 0 ? deps.join(', ') : 'None'}
        </div>
      ),
    },
  ];

  return (
    <Card
      title="Cargo.lock Explorer"
      extra={
        <Space>
          <Input
            prefix={<SearchOutlined />}
            placeholder="Search dependencies..."
            value={searchText}
            onChange={(e) => setSearchText((e.target as any).value)}
            style={{ width: 200 }}
            allowClear
          />
          <Button
            type={showOnlyDirect ? 'primary' : 'default'}
            onClick={() => setShowOnlyDirect(!showOnlyDirect)}
          >
            {showOnlyDirect ? 'Showing Direct Only' : 'Show All'}
          </Button>
          <Button onClick={loadLockData} loading={loading}>
            Refresh
          </Button>
        </Space>
      }
    >
      <Tabs defaultActiveKey="dependencies">
        <Tabs.TabPane tab="Dependencies" key="dependencies">
          <Table
            dataSource={filteredDependencies}
            columns={columns}
            rowKey="name"
            loading={loading}
            pagination={{ pageSize: 10 }}
            scroll={{ x: true }}
          />
        </Tabs.TabPane>
        <Tabs.TabPane tab="Dependency Tree" key="tree" disabled={lockData.length === 0}>
          <div style={{ padding: 16 }}>
            <p>Dependency tree visualization will be implemented here</p>
          </div>
        </Tabs.TabPane>
      </Tabs>
    </Card>
  );
};
