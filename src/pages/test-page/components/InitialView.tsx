import {
  Box,
  Button,
  Radio,
  RadioGroup,
  FormControlLabel,
  TextField,
  SelectChangeEvent,
	Switch,
	Snackbar,
	Alert,
} from '@mui/material';
import { Dayjs } from 'dayjs';
import { DateTimePicker, LocalizationProvider } from '@mui/x-date-pickers';
import { AdapterDayjs } from '@mui/x-date-pickers/AdapterDayjs';
import { DateTimeRangePicker } from '@/components/DateTimeRangePicker';
import { DistanceControl } from '@/components/DistanceControl';

const ITEM_HEIGHT = 48;
const ITEM_PADDING_TOP = 8;
export const MenuProps = {
  PaperProps: {
    style: {
      maxHeight: ITEM_HEIGHT * 4.5 + ITEM_PADDING_TOP,
      width: 250,
    },
  },
};

interface Props {
  timeMode: 'fixed' | 'range';
  fixedTime: Dayjs | null;
  rangeTime: [Dayjs | null, Dayjs | null];
  distanceMode: 'all' | 'range' | 'select';
  minDistance: number;
  maxDistance: number;
  distances: number[];
	initialStake: number | "";
	errors: Record<string, string>;
	initialBalance: number | "";
	favoriteProtection: boolean;
	oddsMin: number | string;
	oddsMax: number | string;
	runStatus: "success" | "error" | null;
  handleTimeMode: (v: 'fixed' | 'range') => void;
  setFixedTime: (v: Dayjs | null) => void;
  setRangeTime: (v: [Dayjs | null, Dayjs | null]) => void;
  handleDistanceMode: (v: 'all' | 'range' | 'select') => void;
  setMinDistance: (n: number) => void;
  setMaxDistance: (n: number) => void;
  handleSelectChange: (e: SelectChangeEvent<string[]>) => void;
	handleInitialStake: (v: number | "") => void;
	// handleErrors: (v: Record<string,string>) => void;
  handleInitialBalance: (v: number | "") => void;
	handleFavoriteProtection: (v: boolean) => void;
	handleOddsMin: (v: number | "") => void;
	handleOddsMax: (v: number | "") => void;
	handleRunStatus: (v: "success" | "error" | null) => void;
	onSubmit: (e: React.FormEvent) => void;
}

export const InitialView: React.FC<Props> = ({
  timeMode,
  fixedTime,
  rangeTime,
  distanceMode,
  minDistance,
  maxDistance,
  distances,
	initialStake,
	errors,
	initialBalance,
	favoriteProtection,
	oddsMin,
	oddsMax,
	runStatus,
	handleTimeMode,
  setFixedTime,
  setRangeTime,
  handleDistanceMode,
  setMinDistance,
  setMaxDistance,
  handleSelectChange,
	handleInitialStake,
	handleInitialBalance,
	handleFavoriteProtection,
	handleOddsMin,
	handleOddsMax,
	handleRunStatus,
	// handleErrors,
  onSubmit,
}) => (
    <form
			onSubmit={onSubmit}
			style={{ display: 'flex', flexDirection: 'column', gap: 24, flex: 1 }}
		>
			<Box sx={{ display: 'flex', gap: 4 }}>
				<Box sx={{ flex: 1 }}>
					<RadioGroup
						value={timeMode}
						onChange={e => handleTimeMode(e.target.value as any)}
					>
						<FormControlLabel value="fixed" control={<Radio />} label="Fixed" />
						<FormControlLabel value="range" control={<Radio />} label="Range" />
					</RadioGroup>

					<LocalizationProvider dateAdapter={AdapterDayjs}>
						{timeMode === 'fixed' ? (
							<DateTimePicker
								label="Дата и время"
								value={fixedTime}
								onChange={setFixedTime}
								ampm={false}
							/>
						) : (
							<DateTimeRangePicker
								start={rangeTime[0]}
								end={rangeTime[1]}
								onChange={setRangeTime}
							/>
						)}
					</LocalizationProvider>
				</Box>

				<DistanceControl
					mode={distanceMode}
					min={minDistance}
					max={maxDistance}
					selected={distances}
					onModeChange={handleDistanceMode}
					onMinChange={setMinDistance}
					onMaxChange={setMaxDistance}
					onSelectChange={handleSelectChange}
					errors={{}}
				/>
			</Box>

			<Box sx={{ display: 'flex', flexDirection: 'column', gap: 2, maxWidth: 300 }}>
				<TextField
						label="Неизменяемая ставка"
						type="number"
						value={initialStake}
						onChange={e => handleInitialStake(e.target.value === '' ? '' : +e.target.value)}
						error={Boolean(errors.stake)}
						helperText={errors.stake}
				/>

			<TextField
					label="Начальный баланс"
					type="number"
					value={initialBalance}
					onChange={e => handleInitialBalance(e.target.value === '' ? '' : +e.target.value)}
					error={Boolean(errors.balance)}
					helperText={errors.balance}
			/>

			<FormControlLabel
					control={
						<Switch
							checked={favoriteProtection}
							onChange={e => handleFavoriteProtection(e.target.checked)}
						/>
					}
					label="Защита от фаворита"
			/>
			</Box>

			<Box sx={{ display: 'flex', flexDirection: 'column', gap: 2, maxWidth: 300 }}>
			<TextField
					label="Минимальный кэфф"
					type="number"
					value={oddsMin}
					onChange={e => handleOddsMin(e.target.value === '' ? '' : +e.target.value)}
					error={Boolean(errors.oddsMin)}
					helperText={errors.oddsMin}
			/>

			<TextField
					label="Максимальный кэфф"
					type="number"
					value={oddsMax}
					onChange={e => handleOddsMax(e.target.value === '' ? '' : +e.target.value)}
					error={Boolean(errors.oddsMax)}
					helperText={errors.oddsMax}
			/>
			</Box>

			<Button
			type="submit"
			variant="contained"
			sx={{ alignSelf: 'center', mb: 2 }}
			>
				Запустить тест
			</Button>

		<Snackbar
			open={runStatus === 'success'}
			autoHideDuration={4000}
			onClose={() => handleRunStatus(null)}
		>
			<Alert onClose={() => handleRunStatus(null)} severity="success">
				Тест успешно запущен
			</Alert>
		</Snackbar>

		<Snackbar
			open={runStatus === 'error'}
			autoHideDuration={4000}
			onClose={() => handleRunStatus(null)}
		>
			<Alert onClose={() => handleRunStatus(null)} severity="error">
				Ошибка при запуске теста
			</Alert>
		</Snackbar>
	</form>
);
