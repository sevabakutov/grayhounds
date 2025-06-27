import Box from '@mui/material/Box'
import { TimePicker } from '@mui/x-date-pickers/TimePicker'
import { Dayjs } from 'dayjs'

interface Props {
  start: Dayjs | null
  end: Dayjs | null
  onChange: (newRange: [Dayjs | null, Dayjs | null]) => void
  startLabel?: string
  endLabel?: string
}

export const TimeRangePicker: React.FC<Props> = ({
  start,
  end,
  onChange,
  startLabel = 'Начало',
  endLabel = 'Конец',
}) => {
  const handleStart = (newValue: Dayjs | null) => {
    onChange([newValue, end])
  }
  const handleEnd = (newValue: Dayjs | null) => {
    onChange([start, newValue])
  }

  return (
    <Box display="flex" gap={2}>
      <TimePicker
        label={startLabel}
        value={start}
        onChange={handleStart}
        ampm={false}
      />
      <TimePicker
        label={endLabel}
        value={end}
        onChange={handleEnd}
        ampm={false}
      />
    </Box>
  )
}
