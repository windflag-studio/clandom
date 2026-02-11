import '@fontsource/roboto/300.css';
import '@fontsource/roboto/400.css';
import '@fontsource/roboto/500.css';
import '@fontsource/roboto/700.css';
import React from 'react';
import Tabs from '@mui/material/Tabs';
import Tab from '@mui/material/Tab';
import Box from '@mui/material/Box';
import { ThemeProvider, createTheme } from '@mui/material/styles';
import CssBaseline from '@mui/material/CssBaseline';
import DrawPage from './Pages/DrawPage';
import SyncRoundedIcon from '@mui/icons-material/SyncRounded';
import TimelineRoundedIcon from '@mui/icons-material/TimelineRounded';
import SettingsRoundedIcon from '@mui/icons-material/SettingsRounded';

export default function App() {
  return (
    <ThemeProvider theme={darkTheme}>
      <CssBaseline />
      <Context />
    </ThemeProvider>
  );
}

interface TabPanelProps {
  children?: React.ReactNode;
  index: number;
  value: number;
}

function TabPanel(props: TabPanelProps) {
  const { children, value, index, ...other } = props;

  return (
    <div
      style={{
        flex: 1,
        overflow: 'auto'
      }}
      role="tabpanel"
      hidden={value !== index}
      id={`tabpanel-${index}`}
      aria-labelledby={`tab-${index}`}
      {...other}
    >
      {value === index && (
        <Box
          sx={{
            p: 2,
            width: '100%',
            height: '100%'
          }}>
          {children}
        </Box>
      )
      }
    </div >
  );
}

function a11yProps(index: number) {
  return {
    id: `tab-${index}`,
    'aria-controls': `tabpanel-${index}`,
  };
}

const darkTheme = createTheme({
  palette: {
    mode: 'dark',
  },
});

function Context() {
  const [value, setValue] = React.useState(0);

  const handleChange = (_: React.SyntheticEvent, newValue: number) => {
    setValue(newValue);
  };

  return (
    <Box sx={{
      flexGrow: 1,
      display: 'flex',
      width: '100vw',
      height: '100vh',
      position: 'fixed',
      top: 0,
      left: 0,
      overflow: 'hidden',
      bgcolor: 'background.paper'
    }}>
      <Tabs value={value} onChange={handleChange} orientation='vertical' aria-label="Clandom选项" centered>
        <Tab icon={<SyncRoundedIcon />} label="抽取" {...a11yProps(0)} />
        <Tab icon={<TimelineRoundedIcon />} label="统计" {...a11yProps(1)} />
        <Tab icon={<SettingsRoundedIcon />} label="设置" {...a11yProps(2)} />
      </Tabs>
      <TabPanel value={value} index={0}>
        <DrawPage />
      </TabPanel>
      <TabPanel value={value} index={1}>
        Item Two
      </TabPanel>
      <TabPanel value={value} index={2}>
        Item Three
      </TabPanel>
    </Box>
  )
}