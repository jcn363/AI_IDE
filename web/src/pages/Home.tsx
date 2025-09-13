import React from 'react';
import { Box, Button, Container, Typography } from '@mui/material';
import { useNavigate } from 'react-router-dom';

const features = [
  'AI-powered code completion',
  'Integrated Rust analyzer',
  'Built-in terminal',
  'Version control integration',
  'Customizable interface',
];

export function Home() {
  const navigate = useNavigate();

  return (
    <Container maxWidth="lg">
      <Box sx={{ my: 4, textAlign: 'center' }}>
        <Typography variant="h3" component="h1" gutterBottom>
          Welcome to Rust AI IDE
        </Typography>

        <Typography variant="h5" component="h2" gutterBottom>
          A modern development environment for Rust with AI assistance
        </Typography>

        <Box sx={{ my: 4 }}>
          <Button
            variant="contained"
            size="large"
            color="primary"
            onClick={() => navigate('/editor')}
            sx={{ mr: 2 }}
          >
            Open Editor
          </Button>

          <Button
            variant="outlined"
            size="large"
            onClick={() =>
              (window as any).open('https://github.com/yourusername/rust-ai-ide', '_blank')
            }
          >
            View on GitHub
          </Button>
        </Box>

        <Box sx={{ mt: 6, textAlign: 'left', maxWidth: 600, mx: 'auto' }}>
          <Typography variant="h5" gutterBottom>
            Features:
          </Typography>
          <ul>
            {features.map((feature, index) => (
              <li key={index}>
                <Typography variant="body1">{feature}</Typography>
              </li>
            ))}
          </ul>
        </Box>
      </Box>
    </Container>
  );
}
