import { useEffect, useState } from 'react';
import { Box, Tab, Tabs } from '@mui/material';
import { invoke } from '@tauri-apps/api/core';
import { TimeRange } from '@/types';

export const CacheTabs: React.FC<{ onSelect: (r: TimeRange) => void }> = ({ onSelect }) => {
  const [ranges, setRanges] = useState<TimeRange[]>([]);
  const [current, setCurrent] = useState<number | false>(false);

  useEffect(() => {
    (async () => {
      try {
        let timeRanges = await invoke<TimeRange[]>('load_time_ranges');
        
        setRanges(timeRanges);
      } catch (e) {
        console.error('load_time_ranges', e);
      }
    })();
  }, []);

  const change = (_: React.SyntheticEvent, idx: number) => {
    setCurrent(idx);
    onSelect(ranges[idx]);
  };

  return (
    <Box sx={{ borderBottom: 1, borderColor: 'divider' }}>
      <Tabs value={current} onChange={change} variant="scrollable" scrollButtons="auto">
        {ranges.map((r, i) => (
          <Tab
            key={i}
            label={r.endTime ? `${r.startTime} – ${r.endTime}` : r.startTime}
          />
        ))}
      </Tabs>
    </Box>
  );
};
