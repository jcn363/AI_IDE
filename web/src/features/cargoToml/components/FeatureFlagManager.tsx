import React, { useMemo, useState } from 'react';
import {
  Alert,
  Button,
  Card,
  Form,
  Input,
  Modal,
  Space,
  Switch,
  Table,
  Tag,
  Tooltip,
  Typography,
  message,
} from 'antd';
import { DeleteOutlined, InfoCircleOutlined, PlusOutlined } from '@ant-design/icons';
import { CargoManifest, FeatureUsage } from '../../../types/cargo';
import {
  analyzeFeatureFlags,
  getFeatureFlagSuggestions,
  optimizeFeatureFlags,
} from '../featureFlags';

const { Text } = Typography;

interface FeatureFlagManagerProps {
  manifest: CargoManifest;
  onChange: (updatedManifest: CargoManifest) => void;
  className?: string;
}

interface FeatureRow {
  key: string;
  name: string;
  enabled: boolean;
  usedBy: string[];
  isDefault: boolean;
  isUsed: boolean;
}

export const FeatureFlagManager: React.FC<FeatureFlagManagerProps> = ({
  manifest,
  onChange,
  className = '',
}) => {
  const [isAddFeatureModalVisible, setIsAddFeatureModalVisible] = useState(false);
  const [isAddingFeature, setIsAddingFeature] = useState(false);
  const [showUnused, setShowUnused] = useState(false);
  const [showSuggestions, setShowSuggestions] = useState(true);
  const [featuresState, setFeaturesState] = useState<{
    features: FeatureUsage[];
    suggestions: string[];
  }>({ features: [], suggestions: [] });

  const [form] = Form.useForm();

  // Form field types
  interface FeatureFormValues {
    name: string;
    description: string;
    enabled: boolean;
    default: boolean;
  }

  const { features, suggestions } = featuresState;

  // Use useEffect for side effects like data fetching
  React.useEffect(() => {
    let isMounted = true;

    const loadData = async () => {
      try {
        const features = await analyzeFeatureFlags(manifest);
        const suggestions = getFeatureFlagSuggestions(features);

        // Only update state if component is still mounted
        if (isMounted) {
          setFeaturesState({
            features: Array.isArray(features) ? features : [],
            suggestions: Array.isArray(suggestions) ? suggestions : [],
          });
        }
      } catch (error) {
        console.error('Error analyzing feature flags:', error);
        if (isMounted) {
          setFeaturesState({ features: [], suggestions: [] });
        }
      }
    };

    loadData();

    // Cleanup function to prevent state updates after unmount
    return () => {
      isMounted = false;
    };
  }, [manifest]);

  // Convert features to table rows
  const dataSource = useMemo<FeatureRow[]>(() => {
    const featuresMap = (Array.isArray(features) ? features : []).reduce(
      (acc, f) => (f?.name ? { ...acc, [f.name]: f } : acc),
      {} as Record<string, FeatureUsage>,
    );

    return Object.entries(manifest.features || {})
      .filter(([name]) => {
        const feature = featuresMap[name];
        return showUnused || !feature || feature.isUsed;
      })
      .map(([name]) => {
        const feature = featuresMap[name] || {
          name,
          usedBy: [],
          enabledByDefault: false,
          isUsed: false,
        };

        return {
          key: name,
          name,
          enabled: feature.enabledByDefault,
          usedBy: feature.usedBy,
          isDefault: feature.enabledByDefault,
          isUsed: feature.isUsed,
        };
      });
  }, [manifest.features, features, showUnused]);

  const handleToggleFeature = (name: string, enabled: boolean) => {
    const updatedManifest = { ...manifest };

    // Ensure package exists with proper type
    if (!updatedManifest.package) {
      updatedManifest.package = {};
    }

    // Initialize defaultFeatures if it doesn't exist
    if (!updatedManifest.package.defaultFeatures) {
      updatedManifest.package.defaultFeatures = [];
    }

    // Create a new array to avoid mutating the original
    const defaultFeatures = [...(updatedManifest.package.defaultFeatures as string[])];

    if (enabled && !defaultFeatures.includes(name)) {
      defaultFeatures.push(name);
    } else if (!enabled) {
      const index = defaultFeatures.indexOf(name);
      if (index > -1) {
        defaultFeatures.splice(index, 1);
      }
    }

    updatedManifest.package.defaultFeatures =
      defaultFeatures.length > 0 ? defaultFeatures : undefined;
    onChange(updatedManifest);
  };

  const handleDeleteFeature = (name: string) => {
    const updatedManifest = { ...manifest };

    // Remove from features
    if (updatedManifest.features) {
      delete updatedManifest.features[name];
      if (Object.keys(updatedManifest.features).length === 0) {
        delete updatedManifest.features;
      }
    }

    // Ensure package exists
    if (!updatedManifest.package) {
      updatedManifest.package = {};
    }

    // Remove from default features
    if (updatedManifest.package.defaultFeatures) {
      const defaultFeatures = updatedManifest.package.defaultFeatures.filter(
        (f: string) => f !== name,
      );
      updatedManifest.package.defaultFeatures =
        defaultFeatures.length > 0 ? defaultFeatures : undefined;
    }

    onChange(updatedManifest);
  };

  const handleOptimizeFeatures = () => {
    const optimizedManifest = optimizeFeatureFlags(manifest, features);
    onChange(optimizedManifest);
  };

  const columns = [
    {
      title: 'Feature',
      dataIndex: 'name',
      key: 'name',
      render: (name: string, record: FeatureRow) => (
        <Space>
          <Text strong>{name}</Text>
          {record.isDefault && <Tag color="blue">Default</Tag>}
          {!record.isUsed && <Tag color="orange">Unused</Tag>}
        </Space>
      ),
    },
    {
      title: 'Used By',
      dataIndex: 'usedBy',
      key: 'usedBy',
      render: (usedBy: string[]) =>
        (usedBy.length > 0 ? (
          <Space size={[0, 4]} wrap>
            {usedBy.map((dep) => (
              <Tag key={dep} color="geekblue">
                {dep}
              </Tag>
            ))}
          </Space>
        ) : (
          <Text type="secondary">Not used by any dependency</Text>
        )),
    },
    {
      title: 'Status',
      dataIndex: 'enabled',
      key: 'status',
      width: 120,
      render: (enabled: boolean, record: FeatureRow) => (
        <Switch
          checked={enabled}
          onChange={(checked) => handleToggleFeature(record.name, checked)}
          disabled={record.isDefault}
        />
      ),
    },
    {
      title: 'Actions',
      key: 'actions',
      width: 100,
      render: (_: any, record: FeatureRow) => (
        <Space>
          <Tooltip title="Delete feature">
            <Button
              type="text"
              danger
              icon={<DeleteOutlined />}
              onClick={() => handleDeleteFeature(record.name)}
              disabled={record.isUsed}
            />
          </Tooltip>
        </Space>
      ),
    },
  ];

  return (
    <div className={className}>
      <div style={{ marginBottom: 16 }}>
        <Space>
          <Button
            type="primary"
            icon={<PlusOutlined />}
            onClick={() => {
              form.resetFields();
              setIsAddFeatureModalVisible(true);
            }}
          >
            Add Feature
          </Button>

          <Button onClick={handleOptimizeFeatures} disabled={suggestions.length === 0}>
            Optimize Features
          </Button>

          <Switch
            checked={showUnused}
            onChange={setShowUnused}
            checkedChildren="Show All"
            unCheckedChildren="Hide Unused"
          />

          <Switch
            checked={showSuggestions}
            onChange={setShowSuggestions}
            checkedChildren="Show Suggestions"
            unCheckedChildren="Hide Suggestions"
          />
        </Space>
      </div>

      {showSuggestions && suggestions.length > 0 && (
        <Alert
          message={
            <div>
              <Text strong>Suggestions:</Text>
              <ul style={{ marginBottom: 0, paddingLeft: 20 }}>
                {suggestions.map((suggestion, i) => (
                  <li key={i}>{suggestion}</li>
                ))}
              </ul>
            </div>
          }
          type="info"
          showIcon
          icon={<InfoCircleOutlined />}
          style={{ marginBottom: 16 }}
        />
      )}

      <Card>
        <Table
          dataSource={dataSource}
          columns={columns}
          pagination={false}
          size="middle"
          rowClassName={(record) => (!record.isUsed ? 'unused-feature' : '')}
        />
      </Card>

      <style jsx global>{`
        n .unused-feature {
          opacity: 0.7;
        }
        .unused-feature:hover {
          opacity: 1;
        }
      `}</style>

      {/* Add Feature Modal */}
      <Modal
        title="Add New Feature Flag"
        open={isAddFeatureModalVisible}
        onCancel={() => setIsAddFeatureModalVisible(false)}
        footer={[
          <Button key="cancel" onClick={() => setIsAddFeatureModalVisible(false)}>
            Cancel
          </Button>,
          <Button
            key="submit"
            type="primary"
            loading={isAddingFeature}
            onClick={async () => {
              try {
                const values = await form.validateFields();
                setIsAddingFeature(true);

                // Update the manifest with the new feature
                const updatedManifest = { ...manifest };
                if (!updatedManifest.features) {
                  updatedManifest.features = {};
                }

                // Add the new feature
                updatedManifest.features[values.name] = [];

                // If this is a default feature, add it to default features
                if (values.default) {
                  if (!updatedManifest.features.default) {
                    updatedManifest.features.default = [];
                  }
                  updatedManifest.features.default.push(values.name);
                }

                // Update the manifest
                onChange(updatedManifest);

                // Show success message
                message.success(`Feature "${values.name}" added successfully`);

                // Close the modal
                setIsAddFeatureModalVisible(false);
              } catch (error) {
                console.error('Error adding feature:', error);
                message.error('Failed to add feature. Please check the form and try again.');
              } finally {
                setIsAddingFeature(false);
              }
            }}
          >
            Add Feature
          </Button>,
        ]}
      >
        <Form
          form={form}
          layout="vertical"
          initialValues={{ enabled: true, default: false }}
        >
          <Form.Item
            name="name"
            label="Feature Name"
            rules={[
              { required: true, message: 'Please enter a feature name' },
              {
                pattern: /^[a-z0-9_-]+$/,
                message: 'Feature name can only contain lowercase letters, numbers, underscores, and hyphens',
              },
              {
                validator: (_, value) => {
                  if (manifest.features?.[value] !== undefined) {
                    return Promise.reject(new Error('A feature with this name already exists'));
                  }
                  return Promise.resolve();
                },
              },
            ]}
          >
            <Input placeholder="e.g., serde, tokio-runtime, wasm-bindgen" />
          </Form.Item>

          <Form.Item
            name="description"
            label="Description"
            rules={[{ required: true, message: 'Please enter a description' }]}
          >
            <Input.TextArea rows={2} placeholder="Describe what this feature enables" />
          </Form.Item>

          <Form.Item
            name="enabled"
            label="Enabled by Default"
            valuePropName="checked"
          >
            <Switch />
          </Form.Item>

          <Form.Item
            name="default"
            label="Include in Default Features"
            valuePropName="checked"
            help="If enabled, this feature will be included when the 'default' feature is enabled"
          >
            <Switch />
          </Form.Item>
        </Form>
      </Modal>
    </div>
  );
};

export default FeatureFlagManager;
