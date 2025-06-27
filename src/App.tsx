// App.tsx
import { CssBaseline, GlobalStyles } from '@mui/material';
import Router from './components/Router';

function App() {
  return (
    <>
      <CssBaseline />
      <GlobalStyles
        styles={{
          '*': {
            margin: 0,
            padding: 0,
            boxSizing: 'border-box',
          },
        }}
      />
      <Router />
    </>
  );
}

export default App;
