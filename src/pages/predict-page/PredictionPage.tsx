import dayjs from 'dayjs';
import ArrowBackIcon from '@mui/icons-material/ArrowBack';
import { useState, useEffect } from 'react';
import { Box, CircularProgress, IconButton, Snackbar, Alert } from '@mui/material';
import { InitialView } from './components/InitialView';
import { ResultsView } from './components/ResultsView';
import { CacheTabs } from '@/components/CacheTabs';
import { invoke } from '@tauri-apps/api/core';
import { Prediction, TimeRange } from '@/types';
import { DISATNCES, DOGS_TIMEZONE, MAX_DISTANCE, MIN_DISTANCE } from '@/utils/constants';

type PredictInput = {
  input: {
    time:
      | { fixedTime: string }
      | { rangeTime: { startTime: string; endTime: string } };
    distances: number[];
  };
};

const PredictionPage = () => {
  const [isLoading, setIsLoading] = useState(false);
  const [step, setStep] = useState(0);
  const [predictions, setPredictions] = useState<Prediction[]>([]);
  const [predictionsCache, setPredictionsCache] = useState<Record<string, Prediction[]>>({});

  const [timeMode, setTimeMode] = useState<'fixed'|'range'>('fixed');
  const [fixedTime, setFixedTime] = useState<any>(dayjs().tz(DOGS_TIMEZONE));
  const [rangeTime, setRangeTime] = useState<any>([dayjs().tz(DOGS_TIMEZONE), dayjs().tz(DOGS_TIMEZONE).add(1, 'hour')]);
  const [distanceMode, setDistanceMode] = useState<'all'|'range'|'select'>('all');
  const [minDistance, setMinDistance] = useState<number>(MIN_DISTANCE);
  const [maxDistance, setMaxDistance] = useState<number>(MAX_DISTANCE);
  const [distances, setDistances] = useState<number[]>(DISATNCES);

  const [errors, setErrors] = useState<{
    fixedTime?: boolean;
    startTime?: boolean;
    endTime?: boolean;
    minDistance?: boolean;
    maxDistance?: boolean;
    distances?: boolean;
  }>({});
  const [alertStatus, setAlertStatus] = useState<'success'|'error'|null>(null);

  // Фильтры для копирования (те же, что использовались для предсказаний)
  const [copyInput, setCopyInput] = useState<PredictInput | null>(null);

  useEffect(() => {
    if (distanceMode === 'all') {
      setDistances(DISATNCES);
    } else if (distanceMode === 'range') {
      setDistances(DISATNCES.filter(d => d >= minDistance && d <= maxDistance));
    } else {
      setDistances([]);
    }
  }, [distanceMode, minDistance, maxDistance]);

  const handleSelectChange = (e: any) => {
    const vals = Array.isArray(e.target.value) ? e.target.value : [e.target.value];
    setDistances(vals.map((v: string | number) => Number(v)));
  };

  const formatTime = (d: Date) => d.toISOString().substr(11, 8);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();

    const newErrors: typeof errors = {};
    let valid = true;

    if (timeMode === 'fixed') {
      if (!fixedTime) {
        newErrors.fixedTime = true;
        valid = false;
      }
    } else {
      if (!rangeTime[0]) { newErrors.startTime = true; valid = false; }
      if (!rangeTime[1]) { newErrors.endTime = true; valid = false; }
      if (rangeTime[0] && rangeTime[1] && rangeTime[0].isAfter(rangeTime[1])) {
        newErrors.startTime = true;
        newErrors.endTime = true;
        valid = false;
      }
    }

    if (distanceMode === 'range' && minDistance > maxDistance) {
      newErrors.minDistance = true;
      newErrors.maxDistance = true;
      valid = false;
    }

    if (distanceMode === 'select' && distances.length === 0) {
      newErrors.distances = true;
      valid = false;
    }

    if (!valid) {
      setErrors(newErrors);
      return;
    }

    setErrors({});
    setIsLoading(true);

    try {
      const time = timeMode === 'fixed'
        ? { fixedTime: formatTime(fixedTime) }
        : {
            rangeTime: {
              startTime: formatTime(rangeTime[0]),
              endTime: formatTime(rangeTime[1]),
            },
          };

      const payload: PredictInput = { input: { time, distances } };

      // Сохраняем фильтры для кнопок копирования
      setCopyInput(payload);

      const preds = await invoke<Prediction[]>('run_predict', payload);

      setPredictions(preds);
      setAlertStatus('success');
      setStep(1);
    } catch (error) {
      setAlertStatus('error');
      console.error('run_predict error', error);
    } finally {
      setIsLoading(false);
    }
  };

  const handleTabSelect = async (range: TimeRange) => {
    const key = `${range.startTime}-${range.endTime ?? ''}`;

    // Восстановим copyInput из preds и выбранного диапазона
    const setCopyFromPreds = (preds: Prediction[], r: TimeRange) => {
      const uniqDistances = Array.from(new Set(preds.map(p => p.meta.distance)));
      if (r.endTime) {
        setCopyInput({
          input: {
            time: {
              rangeTime: {
                startTime: `${r.startTime}:00`,
                endTime: `${r.endTime}:00`,
              },
            },
            distances: uniqDistances,
          },
        });
      } else {
        setCopyInput({
          input: {
            time: { fixedTime: `${r.startTime}:00` },
            distances: uniqDistances,
          },
        });
      }
    };

    if (predictionsCache[key]) {
      const preds = predictionsCache[key];
      setPredictions(preds);
      setCopyFromPreds(preds, range);
      setStep(1);
      return;
    }

    setIsLoading(true);
    try {
      const start = `${range.startTime}:00`;
      const end   = range.endTime ? `${range.endTime}:00` : null;
      const preds = await invoke<Prediction[]>('load_predictions', {
        input: { timeRange: { startTime: start, endTime: end } },
      });

      setPredictionsCache(prev => ({ ...prev, [key]: preds }));
      setPredictions(preds);
      setCopyFromPreds(preds, range);
      setStep(1);
      setAlertStatus('success');
    } catch (err) {
      setAlertStatus('error');
      console.error('load_predictions error', err);
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <Box sx={{ position: 'relative', height: '100vh', display: 'flex', flexDirection: 'column', p: 4 }}>
      {step === 1 && (
        <IconButton onClick={() => setStep(0)} sx={{ alignSelf: 'flex-start', mb: 2 }}>
          <ArrowBackIcon />
        </IconButton>
      )}

      <CacheTabs key={step} onSelect={handleTabSelect} />

      {step === 0 && (
        <InitialView
          timeMode={timeMode}
          fixedTime={fixedTime}
          rangeTime={rangeTime}
          distanceMode={distanceMode}
          minDistance={minDistance}
          maxDistance={maxDistance}
          distances={distances}
          handleTimeMode={setTimeMode}
          setFixedTime={setFixedTime}
          setRangeTime={setRangeTime}
          handleDistanceMode={setDistanceMode}
          setMinDistance={setMinDistance}
          setMaxDistance={setMaxDistance}
          handleSelectChange={handleSelectChange}
          onSubmit={handleSubmit}
          errors={errors}
        />
      )}

      {step === 1 && (
        <ResultsView
          key={JSON.stringify(predictions.map(p => p.meta.time))}
          predictions={predictions}
          copyInput={copyInput}
        />
      )}

      {isLoading && (
        <Box
          sx={{
            position: 'absolute',
            inset: 0,
            bgcolor: 'rgba(255,255,255,0.5)',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
          }}
        >
          <CircularProgress />
        </Box>
      )}

      <Snackbar
        open={alertStatus === 'success'}
        autoHideDuration={4000}
        onClose={() => setAlertStatus(null)}
      >
        <Alert onClose={() => setAlertStatus(null)} severity="success">
          Данные успешно загружены
        </Alert>
      </Snackbar>

      <Snackbar
        open={alertStatus === 'error'}
        autoHideDuration={4000}
        onClose={() => setAlertStatus(null)}
      >
        <Alert onClose={() => setAlertStatus(null)} severity="error">
          Ошибка при получении данных
        </Alert>
      </Snackbar>
    </Box>
  );
};

export default PredictionPage;
