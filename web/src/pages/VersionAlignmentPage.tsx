import React from 'react';
import { Box, Container, Typography } from '@mui/material';
import { VersionAlignmentView } from '../features/dependency/components/version-alignment';

const VersionAlignmentPage: React.FC = () => {
  return (
    <Container maxWidth={false} sx={{ py: 4, height: '100%', overflow: 'auto' }}>
      <Box mb={4}>
        <Typography variant="h4" component="h1" gutterBottom>
          Dependency Version Alignment
        </Typography>
        <Typography variant="body1" color="textSecondary" paragraph>
          View and manage version conflicts across your workspace dependencies.
        </Typography>
      </Box>
      <VersionAlignmentView />
    </Container>
  );
};

export default VersionAlignmentPage;
