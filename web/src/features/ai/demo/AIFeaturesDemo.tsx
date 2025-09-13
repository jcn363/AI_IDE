import React from 'react';
import { Box, Container, Typography, Tabs, Tab, Paper } from '@mui/material';
import { AIProvider } from '..';
import AIFeatureExample from '../examples/AIFeatureExample';

interface TabPanelProps {
  children?: React.ReactNode;
  index: number;
  value: number;
}

function TabPanel(props: TabPanelProps) {
  const { children, value, index, ...other } = props;

  return (
    <div
      role="tabpanel"
      hidden={value !== index}
      id={`ai-feature-tabpanel-${index}`}
      aria-labelledby={`ai-feature-tab-${index}`}
      {...other}
    >
      {value === index && <Box sx={{ p: 3 }}>{children}</Box>}
    </div>
  );
}

function a11yProps(index: number) {
  return {
    id: `ai-feature-tab-${index}`,
    'aria-controls': `ai-feature-tabpanel-${index}`,
  };
}

const AIFeaturesDemo: React.FC = () => {
  const [value, setValue] = React.useState(0);

  const handleChange = (_: React.SyntheticEvent, newValue: number) => {
    setValue(newValue);
  };

  return (
    <AIProvider>
      <Container maxWidth="lg" sx={{ py: 4 }}>
        <Typography variant="h4" component="h1" gutterBottom>
          AI-Enhanced Development Features
        </Typography>

        <Typography variant="body1" paragraph>
          Explore the AI-powered features that enhance your development experience. Try out
          different AI capabilities on the example code or your own code.
        </Typography>

        <Paper sx={{ width: '100%', mb: 4 }}>
          <Box sx={{ borderBottom: 1, borderColor: 'divider' }}>
            <Tabs
              value={value}
              onChange={handleChange}
              aria-label="AI features tabs"
              variant="scrollable"
              scrollButtons="auto"
            >
              <Tab label="Interactive Demo" {...a11yProps(0)} />
              <Tab label="Code Analysis" {...a11yProps(1)} />
              <Tab label="Error Resolution" {...a11yProps(2)} />
              <Tab label="Code Generation" {...a11yProps(3)} />
            </Tabs>
          </Box>

          <TabPanel value={value} index={0}>
            <AIFeatureExample />
          </TabPanel>

          <TabPanel value={value} index={1}>
            <Typography variant="h6" gutterBottom>
              Advanced Code Analysis
            </Typography>
            <Typography paragraph>The AI analyzes your code for:</Typography>
            <ul>
              <li>Code smells and anti-patterns</li>
              <li>Performance bottlenecks</li>
              <li>Security vulnerabilities</li>
              <li>Style inconsistencies</li>
              <li>Architecture improvements</li>
            </ul>
            <Typography paragraph>Try the interactive demo to see it in action!</Typography>
          </TabPanel>

          <TabPanel value={value} index={2}>
            <Typography variant="h6" gutterBottom>
              Smart Error Resolution
            </Typography>
            <Typography paragraph>Get intelligent help with errors and warnings:</Typography>
            <ul>
              <li>Context-aware fixes</li>
              <li>Step-by-step explanations</li>
              <li>Links to documentation</li>
              <li>Learning from previous fixes</li>
            </ul>
          </TabPanel>

          <TabPanel value={value} index={3}>
            <Typography variant="h6" gutterBottom>
              AI-Powered Code Generation
            </Typography>
            <Typography paragraph>Generate high-quality code with AI:</Typography>
            <ul>
              <li>Test cases</li>
              <li>Documentation</li>
              <li>Boilerplate code</li>
              <li>Example usage</li>
              <li>Implementation stubs</li>
            </ul>
          </TabPanel>
        </Paper>

        <Box sx={{ mt: 4, p: 3, bgcolor: 'background.paper', borderRadius: 1 }}>
          <Typography variant="h6" gutterBottom>
            Getting Started
          </Typography>
          <Typography paragraph>
            1. Enter your code in the input area or use the example provided
          </Typography>
          <Typography paragraph>2. Select an action from the dropdown menu</Typography>
          <Typography paragraph>3. Click "Run" to see the AI in action</Typography>
          <Typography>
            Pro tip: Right-click in the code area for quick access to AI features!
          </Typography>
        </Box>
      </Container>
    </AIProvider>
  );
};

export default AIFeaturesDemo;
