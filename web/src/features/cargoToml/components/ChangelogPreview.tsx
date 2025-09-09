import { Modal, Tag, Typography } from 'antd';
import { ChangelogData } from '../types/dependencyUpdates';

const { Title, Text } = Typography;

interface ChangelogPreviewProps {
  packageName: string;
  changelog: ChangelogData;
  visible: boolean;
  onClose: () => void;
}

const changeTypeColors = {
  added: 'green',
  changed: 'blue',
  deprecated: 'orange',
  removed: 'red',
  fixed: 'purple',
  security: 'red',
} as const;

export const ChangelogPreview: React.FC<ChangelogPreviewProps> = ({
  packageName,
  changelog,
  visible,
  onClose,
}) => {
  const groupByType = changelog.changes.reduce((acc, change) => {
    if (!acc[change.type]) {
      acc[change.type] = [];
    }
    acc[change.type].push(change);
    return acc;
  }, {} as Record<string, typeof changelog.changes>);

  return (
    <Modal
      title={
        <>
          <span>Changelog: </span>
          <Text strong>{packageName}</Text>
          <Tag color="blue" style={{ marginLeft: 8 }}>
            v{changelog.version}
          </Tag>
          {changelog.date && (
            <Text type="secondary" style={{ marginLeft: 8 }}>
              {new Date(changelog.date).toLocaleDateString()}
            </Text>
          )}
        </>
      }
      open={visible}
      onCancel={onClose}
      footer={null}
      width={800}
      bodyStyle={{ maxHeight: '60vh', overflowY: 'auto' }}
    >
      <div className="changelog-content">
        {Object.entries(groupByType).map(([type, changes]) => (
          <div key={type} style={{ marginBottom: 24 }}>
            <Title level={4} style={{ textTransform: 'capitalize' }}>
              <Tag color={changeTypeColors[type as keyof typeof changeTypeColors]}>
                {type}
              </Tag>
              {changes.length} {changes.length === 1 ? 'change' : 'changes'}
            </Title>
            <ul style={{ paddingLeft: 24 }}>
              {changes.map((change, idx) => (
                <li key={idx} style={{ marginBottom: 8 }}>
                  <Text>{change.description}</Text>
                </li>
              ))}
            </ul>
          </div>
        ))}
      </div>
    </Modal>
  );
};

export default ChangelogPreview;
