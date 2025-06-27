import { Box } from '@mui/material';
import { Sidebar } from '@/components/Sidebar';
import { Outlet } from 'react-router';

const MainLayout = () => (
  <Box sx={{ display: 'flex', height: '100vh' }}>
    <Sidebar />
    <Box
      component="main"
      sx={{
        flex: 1,
        bgcolor: '#F5F7FB',
        overflow: 'auto',
      }}
    >
      <Outlet />
    </Box>
  </Box>
);

export default MainLayout;