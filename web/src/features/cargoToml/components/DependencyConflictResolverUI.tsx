import React, { useState, useMemo } from 'react';
import { Card, Table, Tag, Button, Space, Select, Alert, Typography } from 'antd';
import { InfoCircleOutlined, CheckCircleOutlined, WarningOutlined } from '@ant-design/icons';
import { detectAndResolveConflicts, VersionConflict, ResolutionStrategy } from '../conflictResolver';
import { CargoManifest } from '../../../types/cargo';

const { Text } = Typography;
const { Option } = Select;

interface DependencyConflictResolverUIProps {
  manifest: CargoManifest;
  lockfile: any;
  onResolve?: (resolutions: Record<string, string>) => void;
  className?: string;
}

export const DependencyConflictResolverUI: React.FC<DependencyConflictResolverUIProps> = ({
  manifest,
  lockfile,
  onResolve,
  className = '',
}) => {
  const [strategy, setStrategy] = useState<ResolutionStrategy>({
    preferStable: true,
    preferHighest: true,
  });

  // Detect and resolve conflicts
  const { conflicts, resolutions } = useMemo(() => {
    const conflicts = detectAndResolveConflicts(manifest, lockfile, strategy);
    const resolutions = conflicts.reduce((acc, conflict) => {
      if (conflict.resolution) {
        acc[conflict.package] = conflict.resolution.version;
      }
      return acc;
    }, {} as Record<string, string>);
    
    return { conflicts, resolutions };
  }, [manifest, lockfile, strategy]);

  const handleResolve = () => {
    onResolve?.(resolutions);
  };

  const columns = [
    {
      title: 'Package',
      dataIndex: 'package',
      key: 'package',
      render: (text: string) => <Text strong>{text}</Text>,
    },
    {
      title: 'Requested Versions',
      dataIndex: 'requestedVersions',
      key: 'versions',
      render: (versions: Array<{ version: string; by: string[] }>) => (
        <Space direction="vertical" size={4}>
          {versions.map((v, i) => (
            <div key={i}>
              <Tag color="blue">{v.version}</Tag>
              <Text type="secondary" style={{ marginLeft: 8 }}>
                Required by: {v.by.join(', ')}
              </Text>
            </div>
          ))}
        </Space>
      ),
    },
    {
      title: 'Resolution',
      dataIndex: 'resolution',
      key: 'resolution',
      render: (resolution: { version: string; reason: string } | undefined) =>
        resolution ? (
          <div>
            <Tag color="green" icon={<CheckCircleOutlined />}>
              {resolution.version}
            </Tag>
            <div style={{ marginTop: 4 }}>
              <Text type="secondary">{resolution.reason}</Text>
            </div>
          </div>
        ) : (
          <Tag color="orange" icon={<WarningOutlined />}>
            No resolution found
          </Tag>
        ),
    },
  ];

  if (conflicts.length === 0) {
    return (
      <Alert
        message="No dependency conflicts found"
        type="success"
        showIcon
        icon={<InfoCircleOutlined />}
        className={className}
      />
    );
  }

  return (
    <Card 
      title="Dependency Conflicts" 
      className={className}
      extra={
        <Space>
          <Select
            value={strategy.preferHighest ? 'highest' : 'lowest'}
            onChange={(value) => {
              setStrategy({
                ...strategy,
                preferHighest: value === 'highest',
                preferLowest: value === 'lowest',
              });
            }}
            style={{ width: 120 }}
          >
            <Option value="highest">Highest Version</Option>
            <Option value="lowest">Lowest Version</Option>
          </Select>
          
          <Select
            value={strategy.preferStable ? 'stable' : 'all'}
            onChange={(value) => {
              setStrategy({
                ...strategy,
                preferStable: value === 'stable',
              });
            }}
            style={{ width: 120 }}
          >
            <Option value="stable">Stable Only</Option>
            <Option value="all">Include Pre-release</Option>
          </Select>
          
          <Button 
            type="primary" 
            onClick={handleResolve}
            disabled={Object.keys(resolutions).length === 0}
          >
            Apply Resolutions
          </Button>
        </Space>
      }
    >
      <Alert
        message={
          <>
            Found <strong>{conflicts.length}</strong> dependency conflict{conflicts.length > 1 ? 's' : ''}.
            Select a resolution strategy and click "Apply Resolutions" to fix them.
          </>
        }
        type="warning"
        showIcon
        style={{ marginBottom: 16 }}
      />
      
      <Table
        dataSource={conflicts}
        columns={columns}
        rowKey="package"
        pagination={false}
        size="small"
      />
      
      <div style={{ marginTop: 16, textAlign: 'right' }}>
        <Button 
          type="primary" 
          onClick={handleResolve}
          disabled={Object.keys(resolutions).length === 0}
        >
          Apply Resolutions
        </Button>
      </div>
    </Card>
  );
};

export default DependencyConflictResolverUI;
