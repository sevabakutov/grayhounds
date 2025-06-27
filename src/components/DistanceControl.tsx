import {
  Box,
  FormControl,
  FormControlLabel,
  FormLabel,
  Radio,
  RadioGroup,
  TextField,
  InputLabel,
  Select,
  OutlinedInput,
  MenuItem,
  Chip,
  SelectChangeEvent,
} from '@mui/material'
import { DISATNCES } from '../utils/constants'

const ITEM_HEIGHT = 48
const ITEM_PADDING_TOP = 8

export const MenuProps = {
  PaperProps: {
    style: {
      maxHeight: ITEM_HEIGHT * 4.5 + ITEM_PADDING_TOP,
      width: 250,
    },
  },
}

interface Props {
  mode: 'all' | 'range' | 'select'
  min: number
  max: number
  selected: number[]
  onModeChange: (v: 'all' | 'range' | 'select') => void
  onMinChange: (n: number) => void
  onMaxChange: (n: number) => void
  onSelectChange: (e: SelectChangeEvent<string[]>) => void
  errors: {
    min?: boolean
    max?: boolean
    selected?: boolean
  }
}

export const DistanceControl: React.FC<Props> = ({
  mode,
  min,
  max,
  selected,
  onModeChange,
  onMinChange,
  onMaxChange,
  onSelectChange,
  errors,
}) => (
  <FormControl
    sx={{ width: '48%' }}
    error={
      mode === 'range'
        ? Boolean(errors.min || errors.max)
        : mode === 'select'
        ? Boolean(errors.selected)
        : false
    }
  >
    <FormLabel>Дистанция</FormLabel>
    <RadioGroup
      value={mode}
      onChange={e => onModeChange(e.target.value as any)}
    >
      <FormControlLabel value="all" control={<Radio />} label="Все" />
      <FormControlLabel value="range" control={<Radio />} label="Диапазон" />
      <FormControlLabel value="select" control={<Radio />} label="Выбрать" />
    </RadioGroup>

    {mode === 'range' && (
      <Box sx={{ display: 'flex', gap: 1, mt: 1 }}>
        <TextField
          label="Мин дистанция"
          type="number"
          value={min}
          onChange={e => onMinChange(Number(e.target.value))}
        />
        <TextField
          label="Макс дистанция"
          type="number"
          value={max}
          onChange={e => onMaxChange(Number(e.target.value))}
        />
      </Box>
    )}

    {mode === 'select' && (
      <FormControl sx={{ mt: 1 }}>
        <InputLabel>Дистанции</InputLabel>
        <Select
          multiple
          value={selected.map(String)}
          onChange={onSelectChange}
          input={<OutlinedInput label="Дистанции" sx={{ width: 300 }} />}
          renderValue={vals => (
            <Box sx={{ display: 'flex', flexWrap: 'wrap', gap: 0.5 }}>
              {(vals as string[]).map(v => (
                <Chip key={v} label={`${v}м`} />
              ))}
            </Box>
          )}
          MenuProps={MenuProps}
          error={Boolean(errors.selected)}
        >
          {DISATNCES.map(d => (
            <MenuItem key={d} value={String(d)}>
              {d}м
            </MenuItem>
          ))}
        </Select>
      </FormControl>
    )}
  </FormControl>
)
