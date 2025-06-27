import { useState, useEffect } from 'react'
import {
  Box,
  CircularProgress,
  IconButton,
} from '@mui/material'
import dayjs, { Dayjs } from 'dayjs'
import ArrowBackIcon from '@mui/icons-material/ArrowBack'
import { InitialView } from './components/InitialView'
import { DISATNCES, DOGS_TIMEZONE, MAX_DISTANCE, MIN_DISTANCE } from '@/utils/constants'
import ResultsView from './components/ResultsView'
import { invoke } from '@tauri-apps/api/core'
import { TestResults } from '@/types'

const TestingPage = () => {
  const [step, setStep] = useState(0)
  const [isLoading, setIsLoading] = useState(false)

  const [timeMode, setTimeMode] = useState<'fixed' | 'range'>('fixed')
  const [fixedTime, setFixedTime] = useState<Dayjs | null>(dayjs().tz(DOGS_TIMEZONE))
  const [rangeTime, setRangeTime] = useState<[Dayjs | null, Dayjs | null]>([dayjs().tz(DOGS_TIMEZONE), dayjs().tz(DOGS_TIMEZONE).add(1, 'hour')])

  const [distanceMode, setDistanceMode] = useState<'all' | 'range' | 'select'>('all')
  const [minDistance, setMinDistance] = useState<number>(MIN_DISTANCE)
  const [maxDistance, setMaxDistance] = useState<number>(MAX_DISTANCE)
  const [distances, setDistances] = useState<number[]>(DISATNCES)

  const [isFavoriteProtected, setIsFavoriteProtected] = useState(false)
  const [initialStake, setInitialStake] = useState<number | ''>('')
  const [initialBalance, setInitialBalance] = useState<number | ''>('')
  const [oddsMin, setOddsMin] = useState<number | ''>('')
  const [oddsMax, setOddsMax] = useState<number | ''>('')

  const [errors, setErrors] = useState<Record<string,string>>({})
  const [runStatus, setRunStatus] = useState<'success'|'error'|null>(null)

  const [testResults, setTestResults] = useState<TestResults>()

  useEffect(() => {
    if (distanceMode === 'all') {
      setDistances(DISATNCES)
    } else if (distanceMode === 'range') {
      setDistances(DISATNCES.filter(d => d >= minDistance && d <= maxDistance))
    } else {
      setDistances([])
    }
  }, [distanceMode, minDistance, maxDistance])

  const handleSelectChange = (e: any) => {
    const vals = Array.isArray(e.target.value) ? e.target.value : [e.target.value]
    setDistances(vals.map((v: string | number) => Number(v)))
  }

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()

    if (!validate()) return
    
    setIsLoading(true)

    try {
      const dateTime = timeMode === "fixed" 
        ? { fixedDateTime: fixedTime!.format('YYYY-MM-DDTHH:mm:ss') }
        : { rangeDateTime: { startDateTime: rangeTime[0]!.format('YYYY-MM-DDTHH:mm:ss'), endDateTime: rangeTime[1]!.format('YYYY-MM-DDTHH:mm:ss') } };
      const oddsRange = { low: oddsMin, high: oddsMax };

      const payload = {
        dateTime,
        distances,
        initialStake,
        initialBalance,
        isFavoriteProtected,
        oddsRange
      };
      const results = await invoke<TestResults>('run_test', payload);
      // const results = testResultsMock;

      setTestResults(results);
      setRunStatus("success");
      setStep(1);
    } catch (error) {
      setRunStatus("error");
      console.error("run_test error", error);
    } finally {
      setIsLoading(false);
    }
  }

  const validate = () => {
    const e: Record<string,string> = {}

    if (!initialStake || initialStake <= 0) e.stake = 'Должно быть > 0'
    if (!initialBalance || initialBalance <= 0) e.balance = 'Должно быть > 0'
    
    if (!oddsMin || oddsMin <= 0) e.oddsMin = 'Должно быть > 0'
    else if (oddsMin < 1.1) e.oddsMin = 'Минимум 1.1'
    
    if (!oddsMax || oddsMax <= 0) e.oddsMax = 'Должно быть > 0'

    if (oddsMin && oddsMax && oddsMin > oddsMax) {
      e.oddsMin = 'Не больше max'
      e.oddsMax = 'Не меньше min'
    }

    setErrors(e)
    return Object.keys(e).length === 0
  }

  return (
    <Box sx={{ p: 4, height: '100vh', position: 'relative' }}>
      {step === 1 && (
        <IconButton onClick={() => setStep(0)} sx={{ mb: 2 }}>
          <ArrowBackIcon />
        </IconButton>
      )}

      {step === 0 && 
        <InitialView 
          timeMode={timeMode}
          fixedTime={fixedTime}
          rangeTime={rangeTime}
          distanceMode={distanceMode}
          minDistance={minDistance}
          maxDistance={maxDistance}
          distances={distances}
          initialStake={initialStake}
          errors={errors}
          initialBalance={initialBalance}
          favoriteProtection={isFavoriteProtected}
          oddsMin={oddsMin}
          oddsMax={oddsMax}
          runStatus={runStatus}
          handleTimeMode={setTimeMode}
          setFixedTime={setFixedTime}
          setRangeTime={setRangeTime}
          handleDistanceMode={setDistanceMode}
          setMinDistance={setMinDistance}
          setMaxDistance={setMaxDistance}
          handleSelectChange={handleSelectChange}
          handleInitialStake={setInitialStake}
          handleInitialBalance={setInitialBalance}
          handleFavoriteProtection={setIsFavoriteProtected}
          handleOddsMin={setOddsMin}
          handleOddsMax={setOddsMax}
          handleRunStatus={setRunStatus}
          onSubmit={handleSubmit}
        />
      }

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

      {step === 1 && <ResultsView data={testResults!}/>}
    </Box>
  )
}

export default TestingPage
