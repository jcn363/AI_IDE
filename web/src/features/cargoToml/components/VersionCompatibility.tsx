import { Progress, Tag, Typography } from 'antd';
import { CheckCircleOutlined, WarningOutlined, InfoCircleOutlined } from '@ant-design/icons';
import { VersionCompatibility } from '../types/dependencyUpdates';

const { Text } = Typography;

interface VersionCompatibilityProps {
  compatibility: VersionCompatibility;
  showDetails?: boolean;
}

export const VersionCompatibilityDisplay: React.FC<VersionCompatibilityProps> = ({
  compatibility,
  showDetails = true,
}) => {
  const { isCompatible, compatibilityScore, breakingChanges, recommendedVersion } = compatibility;
  
  const getStatusColor = () => {
    if (compatibilityScore >= 90) return '#52c41a'; // green
    if (compatibilityScore >= 70) return '#faad14'; // orange
    return '#ff4d4f'; // red
  };

  const statusIcon = isCompatible ? (
    <CheckCircleOutlined style={{ color: '#52c41a', marginRight: 4 }} />
  ) : (
    <WarningOutlined style={{ color: '#faad14', marginRight: 4 }} />
  );

  const statusText = isCompatible ? 'Compatible' : 'Potential Issues';
  
  return (
    <div className="version-compatibility">
      <div style={{ display: 'flex', alignItems: 'center', marginBottom: showDetails ? 8 : 0 }}>
        {statusIcon}
        <Text strong style={{ marginRight: 8 }}>{statusText}</Text>
        <Progress
          percent={compatibilityScore}
          size="small"
          strokeColor={getStatusColor()}
          style={{ width: 100, margin: '0 8px' }}
          showInfo={false}
        />
        <Text type="secondary">{compatibilityScore}%</Text>
      </div>

      {showDetails && (
        <div className="compatibility-details" style={{ marginTop: 8 }}>
          {!isCompatible && recommendedVersion && (
            <div style={{ marginBottom: 8 }}>
              <InfoCircleOutlined style={{ marginRight: 4, color: '#1890ff' }} />
              <Text>Recommended version: </Text>
              <Tag color="blue">{recommendedVersion}</Tag>
            </div>
          )}
          
          {breakingChanges.length > 0 && (
            <div>
              <Text strong style={{ display: 'block', marginBottom: 4 }}>
                Breaking Changes:
              </Text>
              <ul style={{ margin: 0, paddingLeft: 20 }}>
                {breakingChanges.map((change, idx) => (
                  <li key={idx}>
                    <Text>{change}</Text>
                  </li>
                ))}
              </ul>
            </div>
          )}
        </div>
      )}
    </div>
  );
};

export default VersionCompatibilityDisplay;
