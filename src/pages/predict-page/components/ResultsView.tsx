import React, { useEffect, useState } from 'react';
import {
  Box,
  MobileStepper,
  Button,
  Typography,
  TableContainer,
  Table,
  TableHead,
  TableRow,
  TableCell,
  TableBody,
  Paper,
  Snackbar,
  Alert,
} from '@mui/material';
import KeyboardArrowLeft from '@mui/icons-material/KeyboardArrowLeft';
import KeyboardArrowRight from '@mui/icons-material/KeyboardArrowRight';
import { invoke } from '@tauri-apps/api/core';
import { Prediction } from '@/types';

type PredictInput = {
  input: {
    time:
      | { fixedTime: string }
      | { rangeTime: { startTime: string; endTime: string } };
    distances: number[];
  };
};

interface Props {
  predictions: Prediction[];
  copyInput: PredictInput | null;
}

export const ResultsView: React.FC<Props> = ({ predictions, copyInput }) => {
  const [activeStep, setActiveStep] = useState(0);
  const maxSteps = predictions.length;

  const [copyStatus, setCopyStatus] = useState<'success'|'error'|null>(null);
  const [copyMessage, setCopyMessage] = useState<string>('');

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'ArrowRight' && activeStep < maxSteps - 1) {
        setActiveStep(prev => prev + 1);
      } else if (e.key === 'ArrowLeft' && activeStep > 0) {
        setActiveStep(prev => prev - 1);
      }
    };
    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [activeStep, maxSteps]);

  // Только стандартный Web API
  const copyText = async (payload: any) => {
    await navigator.clipboard.writeText(JSON.stringify(payload, null, 2));
  };

  const handleCopyPredictRequest = async () => {
    try {
      if (!copyInput) throw new Error('Нет активных фильтров');
      // Команда возвращает JSON-строку — копируем её
      const json = await invoke<string>('copy_predict_request', copyInput);
      await copyText(json);
      setCopyMessage('Данные races по фильтрам скопированы');
      setCopyStatus('success');
    } catch (err) {
      console.error('copy_predict_request error', err);
      setCopyMessage('Не удалось скопировать данные');
      setCopyStatus('error');
    }
  };

  const handleCopyPredictResults = async () => {
    try {
      if (!copyInput) throw new Error('Нет активных фильтров');
      // Предполагается, что на бэке есть соответствующая команда, которая тоже возвращает строку JSON
      await copyText(predictions);
      setCopyMessage('Результаты предсказаний скопированы');
      setCopyStatus('success');
    } catch (err) {
      console.error('copy_predict_results error', err);
      setCopyMessage('Не удалось скопировать результаты');
      setCopyStatus('error');
    }
  };

  const steps = predictions.map((pred, idx) => (
    <Box key={idx} sx={{ p: 2 }}>
      {/* Верхняя панель действий на странице результатов */}
      <Box sx={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', mb: 1 }}>
        <Typography variant="h6">Результаты гонки</Typography>
        <Box sx={{ display: 'flex', gap: 1 }}>
          <Button
            variant="outlined"
            size="small"
            onClick={handleCopyPredictRequest}
            disabled={!copyInput}
          >
            Копировать запрос
          </Button>
          <Button
            variant="outlined"
            size="small"
            onClick={handleCopyPredictResults}
            disabled={!copyInput}
          >
            Копировать результаты
          </Button>
        </Box>
      </Box>

      <Typography>Time: {pred.meta.time}</Typography>
      <Typography>Distance: {pred.meta.distance}</Typography>
      <Typography>Track: {pred.meta.track}</Typography>
      <Typography>Grade: {pred.meta.grade}</Typography>

      <Typography variant="h6" sx={{ mt: 2 }}>Results:</Typography>
      <TableContainer component={Paper}>
        <Table size="small">
          <TableHead>
            <TableRow>
              <TableCell>Dog Name</TableCell>
              <TableCell>RawScore</TableCell>
              <TableCell>Percentage</TableCell>
              <TableCell>Rank</TableCell>
              <TableCell>Comment</TableCell>
            </TableRow>
          </TableHead>
          <TableBody>
            {pred.predictions.map((prediction, i) => (
              <React.Fragment key={i}>
                <TableRow>
                  <TableCell>{prediction.name}</TableCell>
                  <TableCell>{prediction.rawScore.toFixed(2)}</TableCell>
                  <TableCell>{prediction.percentage}</TableCell>
                  <TableCell>{prediction.rank}</TableCell>
                  <TableCell colSpan={6}>{prediction.comment}</TableCell>
                </TableRow>
              </React.Fragment>
            ))}
          </TableBody>
        </Table>
      </TableContainer>

      <Typography sx={{ mt: 2 }}>{pred.summary}</Typography>
    </Box>
  ));

  return (
    <Box sx={{ flex: 1, display: 'flex', flexDirection: 'column', justifyContent: 'space-between' }}>
      <Box>{steps[activeStep]}</Box>

      <MobileStepper
        variant="text"
        steps={maxSteps}
        position="static"
        activeStep={activeStep}
        nextButton={
          <Button
            size="small"
            onClick={() => setActiveStep(prev => prev + 1)}
            disabled={activeStep === maxSteps - 1}
          >
            Next
            <KeyboardArrowRight />
          </Button>
        }
        backButton={
          <Button
            size="small"
            onClick={() => setActiveStep(prev => prev - 1)}
            disabled={activeStep === 0}
          >
            <KeyboardArrowLeft />
            Back
          </Button>
        }
      />

      {/* Snackbar для статуса копирования */}
      <Snackbar
        open={copyStatus !== null}
        autoHideDuration={3000}
        onClose={() => setCopyStatus(null)}
      >
        <Alert onClose={() => setCopyStatus(null)} severity={copyStatus ?? 'success'}>
          {copyMessage}
        </Alert>
      </Snackbar>
    </Box>
  );
};
