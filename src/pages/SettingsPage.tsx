import { useEffect, useState } from 'react';
import {
  Box,
  Button,
  TextField,
  Select,
  MenuItem,
  FormControl,
  InputLabel,
  FormControlLabel,
  Checkbox,
  Dialog,
  DialogTitle,
  DialogContent,
  DialogActions,
  Snackbar,
  Alert,
} from '@mui/material';
import { invoke } from '@tauri-apps/api/core';

const modelOptions = ['o3-mini', 'o4-mini'];

const SettingsPage: React.FC = () => {
  const [model, setModel] = useState<string>('');
  const [frequencyPenalty, setFrequencyPenalty] = useState<number | null>(null);
  const [logprobs, setLogprobs] = useState<boolean | null>(null);
  const [maxCompletionTokens, setMaxCompletionTokens] = useState<number | null>(null);
  const [presencePenalty, setPresencePenalty] = useState<number | null>(null);
  const [reasoningEffort, setReasoningEffort] = useState<string | null>(null);
  const [seed, setSeed] = useState<number | null>(null);
  const [store, setStore] = useState<boolean | null>(null);
  const [temperature, setTemperature] = useState<number | null>(null);
  const [maxRaces, setMaxRaces] = useState<number>(0);
  const [racesPerRequest, setRacesPerRequest] = useState<number>(0);

  const [instruction, setInstruction] = useState<string>('');
  const [instructionOptions, setInstructionOptions] = useState<string[]>([]);

  const [showEditor, setShowEditor] = useState<boolean>(false);
  const [newInstructionName, setNewInstructionName] = useState<string>('');
  const [instructionText, setInstructionText] = useState<string>('');

  const [errors, setErrors] = useState<Record<string,string>>({});
  const [saveStatus, setSaveStatus] = useState<'success' | 'error' | null>(null);

  useEffect(() => {
    const loadSettings = async () => {
      try {
        const settings = await invoke<{
          model: string;
          max_completion_tokens: number | null;
          frequency_penalty: number | null;
          logprobs: boolean | null;
          presence_penalty: number | null;
          reasoning_effort: string | null;
          seed: number | null;
          store: boolean | null;
          temperature: number | null;
          max_races: number;
          races_per_request: number;
        }>('load_settings', {
          input: { model }
        });

        setModel(settings.model);
        setMaxCompletionTokens(settings.max_completion_tokens);
        setFrequencyPenalty(settings.frequency_penalty);
        setLogprobs(settings.logprobs);
        setPresencePenalty(settings.presence_penalty);
        setReasoningEffort(settings.reasoning_effort);
        setSeed(settings.seed);
        setStore(settings.store);
        setTemperature(settings.temperature);
        setMaxRaces(settings.max_races);
        setRacesPerRequest(settings.races_per_request);
      } catch (err) {
        console.error('load_settings error', err);
      }
    };

    if (model) loadSettings();
  }, [model]);

  useEffect(() => {
    const loadInstructionNames = async () => {
      try {
        const list: string[] = await invoke('read_instruction_names');
        setInstructionOptions(list);
      } catch {
        setInstructionOptions([]);
      }
    };
    loadInstructionNames();
  }, []);

  const validate = () => {
    const e: Record<string,string> = {};
    if (!model) e.model = 'Выберите модель';
    if (frequencyPenalty !== null && (frequencyPenalty < -2 || frequencyPenalty > 2))
      e.frequencyPenalty = 'От –2 до 2';
    if (presencePenalty !== null && (presencePenalty < -2 || presencePenalty > 2))
      e.presencePenalty = 'От –2 до 2';
    if (temperature !== null && (temperature < 0 || temperature > 2))
      e.temperature = 'От 0 до 2';
    if (maxRaces < 1)
      e.maxRaces = 'Должно быть ≥ 1';
    if (racesPerRequest < 1 || racesPerRequest > maxRaces)
      e.racesPerRequest = 'От 1 до Максимального числа гонок';
    if (!instruction) e.instruction = 'Выберите инструкцию';
    
    setErrors(e);

    return Object.keys(e).length === 0;
  };
  
  const handleSave = async () => {
    if (!validate()) return;

    try {
      await invoke('save_settings', {
        input: {
          model: model,
          max_completion_tokens: maxCompletionTokens,
          frequency_penalty: frequencyPenalty,
          logprobs: logprobs,
          presence_penalty: presencePenalty,
          reasoning_effort: reasoningEffort,
          seed: seed,
          store: store,
          temperature: temperature,
          max_races: maxRaces,
          races_per_request: racesPerRequest,
          instruction_name: instruction,
          selected: true
        }
      });

      setSaveStatus('success');
    } catch (err) {
      console.error('save_settings error', err);
      setSaveStatus('error');
    }
  };

  const handleCreateClick = () => {
    setShowEditor(true);
  };

  const handleAddInstruction = async () => {
    const name = newInstructionName.trim();
    if (name) {
      await invoke('add_instruction', {
        input: {
          name,
          content: instructionText
        }
      });
      setShowEditor(false);
      setNewInstructionName('');
      setInstructionText('');
    }
  };

  return (
    <Box sx={{ display: 'flex', flexDirection: 'column', height: '100vh', p: 4 }}>
      <Box sx={{ flexGrow: 1, overflow: 'auto', display: 'flex', flexDirection: 'column', gap: 2 }}>
        <FormControl fullWidth>
          <InputLabel>Модель</InputLabel>
          <Select 
            value={model} 
            label="Модель" 
            onChange={e => setModel(e.target.value)}
            error={Boolean(errors.model)}
          >
            {modelOptions.map(m => (
              <MenuItem key={m} value={m}>{m}</MenuItem>
            ))}
          </Select>
        </FormControl>
        <TextField
          label="frequency penalty"
          type="number"
          value={frequencyPenalty ?? ''}
          onChange={e =>
            setFrequencyPenalty(
              e.target.value === '' ? null : parseFloat(e.target.value)
            )
          }
          slotProps={{
            htmlInput: { min: -2, max: 2, step: 0.1 }
          }}
          fullWidth
          error={Boolean(errors.frequencyPenalty)}
          helperText={errors.frequencyPenalty}
        />
        <FormControlLabel
          control={
            <Checkbox
              checked={Boolean(logprobs)}
              onChange={e => setLogprobs(e.target.checked)}
            />
          }
          label="logprobs"
        />
        <TextField
          label="max_completion_tokens"
          type="number"
          value={maxCompletionTokens ?? ''}
          onChange={e =>
            setMaxCompletionTokens(
              e.target.value === '' ? null : parseInt(e.target.value, 10)
            )
          }
          fullWidth
        />
        <TextField
          label="presence penalty"
          type="number"
          value={presencePenalty ?? ''}
          onChange={e =>
            setPresencePenalty(
              e.target.value === '' ? null : parseFloat(e.target.value)
            )
          }
          slotProps={{
            htmlInput: { min: -2, max: 2, step: 0.1 }
          }}
          fullWidth
          error={Boolean(errors.presencePenalty)}
          helperText={errors.presencePenalty}
        />
        <FormControl fullWidth>
          <InputLabel>reasoning_effort</InputLabel>
          <Select
            value={reasoningEffort ?? ''}
            label="reasoning_effort"
            onChange={e =>
              setReasoningEffort(
                e.target.value === '' ? null : e.target.value
              )
            }
          >
            {['low', 'medium', 'high'].map(level => (
              <MenuItem key={level} value={level}>
                {level}
              </MenuItem>
            ))}
          </Select>
        </FormControl>
        <TextField
          label="seed"
          type="number"
          value={seed ?? ''}
          onChange={e =>
            setSeed(
              e.target.value === '' ? null : parseInt(e.target.value, 10)
            )
          }
          fullWidth
        />
        <FormControlLabel
          control={
            <Checkbox
              checked={Boolean(store)}
              onChange={e => setStore(e.target.checked)}
            />
          }
          label="store"
        />
        <TextField
          label="temperature"
          type="number"
          value={temperature ?? ''}
          onChange={e =>
            setTemperature(
              e.target.value === '' ? null : parseFloat(e.target.value)
            )
          }
          slotProps={{
            htmlInput: { min: 0, max: 2, step: 0.1 }
          }}
          fullWidth
          error={Boolean(errors.temperature)}
          helperText={errors.temperature}
        />
        <TextField
          label="Максимальное число гонок"
          type="number"
          value={maxRaces}
          slotProps={{
            inputLabel: {
              shrink: true
            }
          }}
          onChange={e => setMaxRaces(parseInt(e.target.value, 10))}
          fullWidth
          error={Boolean(errors.maxRaces)}
          helperText={errors.maxRaces}
        />
        <TextField
          label="Гонок в запросе"
          type="number"
          value={racesPerRequest}
          onChange={e => setRacesPerRequest(parseInt(e.target.value, 10))}
          fullWidth
          error={Boolean(errors.racesPerRequest)}
          helperText={errors.racesPerRequest}
          slotProps={{
            inputLabel: {
              shrink: true
            }
          }}
        />
        <FormControl fullWidth>
          <InputLabel>Инструкция</InputLabel>
          <Select 
            value={instruction} 
            label="Инструкция" 
            onChange={e => setInstruction(e.target.value)}
            error={Boolean(errors.instruction)}
          >
            {instructionOptions.map(inst => (
              <MenuItem 
                key={inst} 
                value={inst}
              >
                {inst}
              </MenuItem>
            ))}
          </Select>
        </FormControl>
        <Button variant="outlined" onClick={handleCreateClick}>Создать инструкцию</Button>
      </Box>
      <Button 
        variant="contained" 
        sx={{ alignSelf: 'center', mt: 2 }}
        onClick={handleSave}
      >
        Сохранить
      </Button>

      <Dialog
        open={showEditor}
        onClose={() => setShowEditor(false)}
        slotProps={{
          paper: {
            sx: {
              width: '800px',
              height: '600px',
            }
          }
        }}
      >
        <DialogTitle>Новая инструкция</DialogTitle>
        <DialogContent>
          <TextField
            autoFocus
            margin="dense"
            label="Название инструкции"
            fullWidth
            value={newInstructionName}
            onChange={e => setNewInstructionName(e.target.value)}
          />
          <TextField
            margin="dense"
            label="Контент инструкции"
            fullWidth
            multiline
            minRows={4}
            value={instructionText}
            onChange={e => setInstructionText(e.target.value)}
          />
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setShowEditor(false)}>Отмена</Button>
          <Button onClick={handleAddInstruction}>Сохранить</Button>
        </DialogActions>
      </Dialog>

      <Snackbar
        open={saveStatus === 'success'}
        autoHideDuration={4000}
        onClose={() => setSaveStatus(null)}
      >
        <Alert onClose={() => setSaveStatus(null)} severity="success">
          Настройки успешно сохранены
        </Alert>
      </Snackbar>
      <Snackbar
        open={saveStatus === 'error'}
        autoHideDuration={4000}
        onClose={() => setSaveStatus(null)}
      >
        <Alert onClose={() => setSaveStatus(null)} severity="error">
          Ошибка при сохранении настроек
         </Alert>
      </Snackbar>
    </Box>
  );
};

export default SettingsPage;
