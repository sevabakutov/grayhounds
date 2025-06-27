import {
  Box,
  Button,
  Radio,
  RadioGroup,
  FormControlLabel,
  FormControl,
  FormLabel,
} from '@mui/material'
import { SelectChangeEvent } from '@mui/material/Select'
import { Dayjs } from 'dayjs'
import { LocalizationProvider, TimeField } from '@mui/x-date-pickers'
import { AdapterDayjs } from '@mui/x-date-pickers/AdapterDayjs'
import { TimeRangePicker } from '@/components/TimeRangePicker'
import { DistanceControl } from '@/components/DistanceControl'

interface Props {
  timeMode: 'fixed' | 'range'
  fixedTime: Dayjs | null
  rangeTime: [Dayjs | null, Dayjs | null]
  distanceMode: 'all' | 'range' | 'select'
  minDistance: number
  maxDistance: number
  distances: number[]
  handleTimeMode: (v: 'fixed' | 'range') => void
  setFixedTime: (v: Dayjs | null) => void
  setRangeTime: (v: [Dayjs | null, Dayjs | null]) => void
  handleDistanceMode: (v: 'all' | 'range' | 'select') => void
  setMinDistance: (n: number) => void
  setMaxDistance: (n: number) => void
  handleSelectChange: (e: SelectChangeEvent<string[]>) => void
  onSubmit: (e: React.FormEvent) => void
  errors: {
    fixedTime?: boolean
    startTime?: boolean
    endTime?: boolean
    minDistance?: boolean
    maxDistance?: boolean
    distances?: boolean
  }
}

export const InitialView: React.FC<Props> = ({
  timeMode,
  fixedTime,
  rangeTime,
  distanceMode,
  minDistance,
  maxDistance,
  distances,
  handleTimeMode,
  setFixedTime,
  setRangeTime,
  handleDistanceMode,
  setMinDistance,
  setMaxDistance,
  handleSelectChange,
  onSubmit,
  errors,
}) => (
  <form
    onSubmit={onSubmit}
    style={{ flex: 1, display: 'flex', flexDirection: 'column' }}
  >
    <Box
      sx={{ display: 'flex', justifyContent: 'space-between', gap: 2, flex: 1 }}
    >
      <FormControl
        error={
          timeMode === 'fixed'
            ? Boolean(errors.fixedTime)
            : Boolean(errors.startTime || errors.endTime)
        }
      >
        <FormLabel>Время гонок</FormLabel>
        <RadioGroup
          value={timeMode}
          onChange={e => handleTimeMode(e.target.value as any)}
        >
          <FormControlLabel value="fixed" control={<Radio />} label="Фиксированное время" />
          <FormControlLabel value="range" control={<Radio />} label="Диапазон" />
        </RadioGroup>
        <LocalizationProvider dateAdapter={AdapterDayjs}>
          {timeMode === 'fixed' ? (
            <TimeField
              label="Фиксированное время"
              value={fixedTime}
              onChange={setFixedTime}
              ampm={false}
            />
          ) : (
            <TimeRangePicker
              start={rangeTime[0]}
              end={rangeTime[1]}
              onChange={setRangeTime}
            />
          )}
        </LocalizationProvider>
      </FormControl>

      <DistanceControl
        mode={distanceMode}
        min={minDistance}
        max={maxDistance}
        selected={distances}
        onModeChange={handleDistanceMode}
        onMinChange={setMinDistance}
        onMaxChange={setMaxDistance}
        onSelectChange={handleSelectChange}
        errors={{
          min: errors.minDistance,
          max: errors.maxDistance,
          selected: errors.distances,
        }}
      />
    </Box>

    <Button variant="contained" type="submit" sx={{ alignSelf: 'center', mb: 2 }}>
      Предсказать
    </Button>
  </form>
)
