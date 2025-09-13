import { useState, useCallback } from 'react';
import { Button, Checkbox, Input, Select, Space, Card, Divider } from 'antd';
import { FilterOutlined, SearchOutlined, SyncOutlined, ExportOutlined } from '@ant-design/icons';

const { Option } = Select;

type DependencyType = 'all' | 'normal' | 'dev' | 'build' | 'workspace';

export interface DependencyGraphControlsProps {
  onFilterChange: (filters: {
    searchTerm: string;
    dependencyType: DependencyType;
    showFeatures: boolean;
    showTransitive: boolean;
  }) => void;
  onRefresh: () => void;
  onExport: (format: 'svg' | 'png' | 'json') => void;
  isLoading?: boolean;
}

export function DependencyGraphControls({
  onFilterChange,
  onRefresh,
  onExport,
  isLoading = false,
}: DependencyGraphControlsProps) {
  const [searchTerm, setSearchTerm] = useState('');
  const [dependencyType, setDependencyType] = useState<DependencyType>('all');
  const [showFeatures, setShowFeatures] = useState(true);
  const [showTransitive, setShowTransitive] = useState(true);

  const handleSearch = useCallback(() => {
    onFilterChange({
      searchTerm,
      dependencyType,
      showFeatures,
      showTransitive,
    });
  }, [searchTerm, dependencyType, showFeatures, showTransitive, onFilterChange]);

  const handleExport = (format: 'svg' | 'png' | 'json') => {
    onExport(format);
  };

  return (
    <Card
      size="small"
      title={
        <Space>
          <FilterOutlined />
          <span>Dependency Graph Controls</span>
        </Space>
      }
      extra={
        <Button
          icon={<SyncOutlined spin={isLoading} />}
          onClick={onRefresh}
          loading={isLoading}
          size="small"
        >
          Refresh
        </Button>
      }
    >
      <Space direction="vertical" style={{ width: '100%' }}>
        <Space wrap>
          <Input
            placeholder="Search dependencies..."
            prefix={<SearchOutlined />}
            value={searchTerm}
            onChange={(e) => setSearchTerm((e.target as any).value)}
            onPressEnter={handleSearch}
            style={{ width: 200 }}
          />

          <Select
            value={dependencyType}
            onChange={(value) => setDependencyType(value as DependencyType)}
            style={{ width: 150 }}
            onSelect={handleSearch}
          >
            <Option value="all">All Dependencies</Option>
            <Option value="normal">Normal Only</Option>
            <Option value="dev">Dev Only</Option>
            <Option value="build">Build Only</Option>
            <Option value="workspace">Workspace</Option>
          </Select>

          <Button type="primary" onClick={handleSearch} icon={<SearchOutlined />}>
            Apply Filters
          </Button>
        </Space>

        <Divider style={{ margin: '8px 0' }} />

        <Space>
          <Checkbox
            checked={showFeatures}
            onChange={(e) => {
              setShowFeatures(e.target.checked);
              handleSearch();
            }}
          >
            Show Features
          </Checkbox>

          <Checkbox
            checked={showTransitive}
            onChange={(e) => {
              setShowTransitive(e.target.checked);
              handleSearch();
            }}
          >
            Show Transitive Dependencies
          </Checkbox>
        </Space>

        <Divider style={{ margin: '8px 0' }} />

        <Space>
          <span>Export as:</span>
          <Button size="small" icon={<ExportOutlined />} onClick={() => handleExport('svg')}>
            SVG
          </Button>
          <Button size="small" icon={<ExportOutlined />} onClick={() => handleExport('png')}>
            PNG
          </Button>
          <Button size="small" icon={<ExportOutlined />} onClick={() => handleExport('json')}>
            JSON
          </Button>
        </Space>
      </Space>
    </Card>
  );
}
