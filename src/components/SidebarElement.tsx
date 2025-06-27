import { NavLink } from 'react-router';
import { Box } from '@mui/material';

interface Props {
  to: string;
  icon: React.ReactNode;
  children: string;
  first?: boolean;
}

export const SidebarElement: React.FC<Props> = ({ to, icon, children, first }) => (
  <NavLink
    to={to}
    style={({ isActive }) => ({
      display: 'flex',
      alignItems: 'center',
      marginBottom: '17px',
      marginTop: first ? '41px' : 0,
      color: isActive ? '#ffffff' : '#88919F',
      fontSize: 13,
      textDecoration: 'none',
      borderRadius: 5,
      padding: '12px 10px',
      backgroundColor: isActive ? '#2880E6' : 'transparent',
    })}
  >
    <Box sx={{ width: 20, height: 20, mr: 1, display: 'flex' }}>{icon}</Box>
    <Box component="span">{children}</Box>
  </NavLink>
);
