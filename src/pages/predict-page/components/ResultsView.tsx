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
} from '@mui/material';
import KeyboardArrowLeft from '@mui/icons-material/KeyboardArrowLeft';
import KeyboardArrowRight from '@mui/icons-material/KeyboardArrowRight';
import { Prediction } from '@/types';

interface Props {
  predictions: Prediction[];
}

export const ResultsView: React.FC<Props> = ({ predictions }) => {
  const [activeStep, setActiveStep] = useState(0);
  const maxSteps = predictions.length;

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

  console.log("Predictions:", predictions);

  const steps = predictions.map((pred, idx) => (
    <Box key={idx} sx={{ p: 2 }}>
      <Typography variant="h6">Race:</Typography>
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
          <Button size="small" onClick={() => setActiveStep(prev => prev + 1)} disabled={activeStep === maxSteps - 1}>
            Next
            <KeyboardArrowRight />
          </Button>
        }
        backButton={
          <Button size="small" onClick={() => setActiveStep(prev => prev - 1)} disabled={activeStep === 0}>
            <KeyboardArrowLeft />
            Back
          </Button>
        }
      />
    </Box>
  );
};
