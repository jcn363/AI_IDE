import React from 'react';
import { Avatar, Badge, Tooltip, Box } from '@mui/material';
import { Person } from '@mui/icons-material';
import type { PresenceIndicatorProps } from '../types';

export const PresenceIndicator: React.FC<PresenceIndicatorProps> = ({
  user,
  showAvatar = true,
  showName = false,
  size = 'small',
}) => {
  const getInitials = (name: string) => {
    return name
      .split(' ')
      .map(n => n[0])
      .join('')
      .toUpperCase()
      .slice(0, 2);
  };

  const sizeMap = {
    small: 24,
    medium: 32,
    large: 40,
  };

  const sizePx = sizeMap[size];

  const statusColor = {
    online: 'success',
    away: 'warning',
    offline: 'default',
  } as const;

  const tooltipContent = `${user.name}${user.currentLine ? ` (line ${user.currentLine})` : ''}`;

  return (
    <Tooltip title={tooltipContent}>
      <Box sx={{ position: 'relative', display: 'inline-block' }}>
        <Badge
          overlap="circular"
          anchorOrigin={{ vertical: 'bottom', horizontal: 'right' }}
          variant="dot"
          color={statusColor[user.status]}
          sx={{
            '& .MuiBadge-badge': {
              width: 12,
              height: 12,
              borderRadius: '50%',
              border: '2px solid #fff',
            },
          }}
        >
          {showAvatar ? (
            <Avatar
              src={user.avatar}
              sx={{
                width: sizePx,
                height: sizePx,
                fontSize: sizePx * 0.4,
                bgcolor: user.color,
              }}
            >
              {user.avatar ? null : getInitials(user.name)}
            </Avatar>
          ) : (
            <Box
              sx={{
                width: sizePx,
                height: sizePx,
                borderRadius: '50%',
                bgcolor: user.color,
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'center',
              }}
            >
              <Person sx={{ color: 'white', fontSize: sizePx * 0.6 }} />
            </Box>
          )}
        </Badge>
        {showName && (
          <Box
            sx={{
              position: 'absolute',
              bottom: -16,
              left: '50%',
              transform: 'translateX(-50%)',
              fontSize: '0.75rem',
              fontWeight: 'medium',
              color: 'text.primary',
              backgroundColor: 'background.paper',
              px: 1,
              borderRadius: 1,
              boxShadow: 1,
              whiteSpace: 'nowrap',
            }}
          >
            {user.name}
          </Box>
        )}
      </Box>
    </Tooltip>
  );
};