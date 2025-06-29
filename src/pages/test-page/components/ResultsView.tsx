import { useState, useMemo, useEffect } from 'react';
import {
  Box,
  Paper,
  Typography,
  ToggleButton,
  ToggleButtonGroup,
  Table,
  TableBody,
  TableCell,
  TableContainer,
  TableHead,
  TableRow,
  Collapse,
  IconButton,
  Button,
  Snackbar,
  Alert,
} from '@mui/material';
import KeyboardArrowDownIcon from '@mui/icons-material/KeyboardArrowDown';
import KeyboardArrowUpIcon from '@mui/icons-material/KeyboardArrowUp';
import CheckIcon from '@mui/icons-material/Check';
import CloseIcon from '@mui/icons-material/Close';
import { LineChart } from '@mui/x-charts/LineChart';
import { TestResults } from '@/types';

interface Props {
  data: TestResults;
}

const ResultsView: React.FC<Props> = ({ data }) => {
  const [stage, setStage] = useState<1 | 2>(1);
  const [expandedRow, setExpandedRow] = useState<number | null>(null);
  const [alertStatus, setAlertStatus] = useState<'success'|'error'|null>(null);

  const copyAllToClipboard = async (type: 'model' | 'real' | 'request') => {
    try {
      let racesPayload;
      if (type === 'request') {
        racesPayload = data.requests;
      } else {
        racesPayload = data.races.map(r => ({
          raceId: r.raceId,
          ...r.meta,
          dogs: r.dogs.map(d => ({
            dogName: d.dogName,
            ...(type === 'model' ? d.modelPrediction : d.realResults),
          })),
        }));
      }
      await navigator.clipboard.writeText(JSON.stringify(racesPayload, null, 2));

      setAlertStatus("success");
    } catch (error) {
      console.log(error);
      setAlertStatus("error");
    }
  };

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'ArrowRight' && stage < 2) setStage(2);
      if (e.key === 'ArrowLeft' && stage > 1) setStage(1);
    };
    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [stage]);

  const chartData = useMemo(() => {
    const result: { index: number; balance: number }[] = [];

    data.races.forEach(r => {
      const curr = r.meta.currentBalance;
      if (result.length === 0 || curr !== result[result.length - 1].balance) {
        result.push({ index: result.length + 1, balance: curr });
      }
    });

    return result;
  }, [data.races]);

  const {
    raceCount,
    oddsRange,
    positionInfo,
    skipInfo,
    balance,
    errors,
    initialStake,
    percentage,
  } = data.meta;

  const sections = [
    { title: 'Race Count', items: { 'Total Races': raceCount.totalRaces, 'Tracked Races': raceCount.racesTracked } },
    { title: 'Odds Range', items: { Low: oddsRange.low, High: oddsRange.high } },
    { title: 'Position Info', items: { 'Bad Hit 4 Pos': positionInfo.badHit4Pos, 'Bad Hit 5 Pos': positionInfo.badHit5Pos, 'Bad Hit 6 Pos': positionInfo.badHit6Pos } },
    { title: 'Skip Info', items: { 'Skipped Races <5': skipInfo.skippedRacesLt5, 'Skipped Races >6': skipInfo.skippedRacesGt6, 'Skipped Odds Range': skipInfo.skippedOddsRange, 'Skipped Favorite': skipInfo.skippedFavorite } },
    { title: 'Balance', items: { 'Initial Balance': balance.initialBalance, 'Final Balance': balance.finalBalance } },
    { title: 'Errors', items: { 'Empty Content Errors': errors.totalEmptyContent, 'MongoDB Errors': errors.totalMongoDbError, 'Race Parse Errors': errors.totalRaceParseError } },
    { title: 'Initial Stake', items: { 'Stake Amount': initialStake } },
    { title: 'Profit Percentage', items: { 'Profit %': `${percentage}%` } },
  ];

  const firstColumn = ['Race Count', 'Balance', 'Initial Stake', 'Profit Percentage'];

  const renderStageOne = () => (
    <>
      <Box sx={{ mb: 2, display: 'flex', gap: 2 }}>
        <Button variant="outlined" onClick={() => copyAllToClipboard('model')}>
          Скопировать ответ модели
        </Button>
        <Button variant="outlined" onClick={() => copyAllToClipboard('real')}>
          Скопировать реальные результаты
        </Button>
        <Button variant="outlined" onClick={() => copyAllToClipboard('request')}>
          Скопировать запрос
        </Button>
      </Box>

      <Paper sx={{ p: 2 }}>
        <LineChart
          width={800}
          height={350}
          xAxis={[
            {
              data: chartData.map(d => d.index),
              label: 'Race #',
            },
          ]}
          series={[{ data: chartData.map(d => d.balance) }]}
        />
      </Paper>
      <Box sx={{ display: 'flex', gap: 2 }}>
        <Box sx={{ flex: 1, display: 'flex', flexDirection: 'column', gap: 2 }}>
          {sections
            .filter(s => firstColumn.includes(s.title))
            .map(({ title, items }) => (
              <Paper key={title} sx={{ p: 2 }}>
                <Typography variant="h6" gutterBottom>
                  {title}
                </Typography>
                <Box
                  component="ul"
                  sx={{ listStyle: 'none', p: 0, m: 0, display: 'flex', flexWrap: 'wrap', gap: 2 }}
                >
                  {Object.entries(items).map(([label, value]) => (
                    <Box component="li" key={label} sx={{ minWidth: 120 }}>
                      <Typography variant="body2" fontWeight="bold">
                        {label}:
                      </Typography>
                      <Typography variant="body2">{value}</Typography>
                    </Box>
                  ))}
                </Box>
              </Paper>
            ))}
        </Box>
        <Box sx={{ flex: 1, display: 'flex', flexDirection: 'column', gap: 2 }}>
          {sections
            .filter(s => !firstColumn.includes(s.title))
            .map(({ title, items }) => (
              <Paper key={title} sx={{ p: 2 }}>
                <Typography variant="h6" gutterBottom>
                  {title}
                </Typography>
                <Box
                  component="ul"
                  sx={{ listStyle: 'none', p: 0, m: 0, display: 'flex', flexWrap: 'wrap', gap: 2 }}
                >
                  {Object.entries(items).map(([label, value]) => (
                    <Box component="li" key={label} sx={{ minWidth: 120 }}>
                      <Typography variant="body2" fontWeight="bold">
                        {label}:
                      </Typography>
                      <Typography variant="body2">{value}</Typography>
                    </Box>
                  ))}
                </Box>
              </Paper>
            ))}
        </Box>
      </Box>

      <Snackbar
        open={alertStatus === 'success'}
        autoHideDuration={4000}
        onClose={() => setAlertStatus(null)}
      >
        <Alert onClose={() => setAlertStatus(null)} severity="success">
          Данные успешно скопированны в буфер
        </Alert>
      </Snackbar>

      <Snackbar
        open={alertStatus === 'error'}
        autoHideDuration={4000}
        onClose={() => setAlertStatus(null)}
      >
        <Alert onClose={() => setAlertStatus(null)} severity="error">
          Ошибка копирования
        </Alert>
      </Snackbar>
    </>
  );

  const renderStageTwo = () => {
    let prevBalance = Number.MIN_VALUE;

    return (
      <Paper sx={{ p: 2, overflowX: 'auto' }}>
        <TableContainer>
          <Table>
            <TableHead>
              <TableRow>
                <TableCell />
                <TableCell>Race #</TableCell>
                <TableCell>Date</TableCell>
                <TableCell>Distance</TableCell>
                <TableCell>Time</TableCell>
                <TableCell>Grade</TableCell>
                <TableCell>Track</TableCell>
                <TableCell>Balance</TableCell>
                <TableCell>Profit</TableCell>
                <TableCell align="center">Skipped</TableCell>
              </TableRow>
            </TableHead>
            <TableBody>
              {data.races.map((race, idx) => {
                const meta = race.meta;
                const wasSkipped = meta.currentBalance === (idx !== 0 && prevBalance) ? true : false;
                prevBalance = meta.currentBalance;

                return (
                  <>
                    <TableRow key={race.raceId} hover>
                      <TableCell>
                        <IconButton
                          size="small"
                          onClick={() => setExpandedRow(expandedRow === idx ? null : idx)}
                        >
                          {expandedRow === idx ? <KeyboardArrowUpIcon /> : <KeyboardArrowDownIcon />}
                        </IconButton>
                      </TableCell>
                      <TableCell>{idx + 1}</TableCell>
                      <TableCell>{meta.date}</TableCell>
                      <TableCell>{meta.distance}</TableCell>
                      <TableCell>{meta.time}</TableCell>
                      <TableCell>{meta.grade}</TableCell>
                      <TableCell>{meta.track}</TableCell>
                      <TableCell>{meta.currentBalance}</TableCell>
                      <TableCell>{meta.profit}</TableCell>
                      <TableCell align="center">
                        {wasSkipped ? <CheckIcon color="success" /> : <CloseIcon color="error" />}
                      </TableCell>
                    </TableRow>
                    <TableRow>
                      <TableCell style={{ paddingBottom: 0, paddingTop: 0 }} colSpan={9}>
                        <Collapse in={expandedRow === idx} timeout="auto" unmountOnExit>
                          <Box sx={{ display: 'flex', flexDirection: 'column', gap: 2, p: 2 }}>
                            {race.dogs.sort((a, b) => a.modelPrediction.rank - b.modelPrediction.rank).map(dog => (
                              <Paper key={dog.dogName} sx={{ p: 2 }}>
                                <Typography variant="subtitle1" gutterBottom>
                                  {dog.dogName}
                                </Typography>
                                <Box sx={{ display: 'flex', gap: 4 }}>
                                  <Box sx={{ flex: 1 }}>
                                    <Typography variant="subtitle2">Model Prediction</Typography>
                                      <Typography key={dog.dogName} variant="body2">
                                          {dog.modelPrediction.rank}. {dog.dogName}: {dog.modelPrediction.percentage}% (score: {dog.modelPrediction.rawScore})
                                      </Typography>
                                    <Typography variant="body2" fontStyle="italic">
                                      {dog.modelPrediction.comment}
                                    </Typography>
                                  </Box>
                                  <Box sx={{ flex: 1 }}>
                                    <Typography variant="subtitle2">Real Results</Typography>
                                    <Typography variant="body2">
                                      Rank: {dog.realResults.rank}, Odds: {dog.realResults.betfairOdds}
                                    </Typography>
                                  </Box>
                                </Box>
                              </Paper>
                            ))}
                          </Box>
                        </Collapse>
                      </TableCell>
                    </TableRow>
                  </>
                );
              })}
            </TableBody>
          </Table>
        </TableContainer>
      </Paper>
    ); 
  }

  const handleStageChange = (_: any, newStage: 1 | 2) => {
    if (newStage !== null) setStage(newStage);
  };

  return (
    <Box sx={{ display: 'flex', flexDirection: 'column', gap: 3, height: '100%' }}>
      {stage === 1 && renderStageOne()}
      {stage === 2 && renderStageTwo()}
      <Box sx={{ display: 'flex', justifyContent: 'center', mt: 'auto' }}>
        <ToggleButtonGroup value={stage} exclusive onChange={handleStageChange} aria-label="Stage">
          <ToggleButton value={1}>Этап 1</ToggleButton>
          <ToggleButton value={2}>Этап 2</ToggleButton>
        </ToggleButtonGroup>
      </Box>
    </Box>
  );
};

export default ResultsView;