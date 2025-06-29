export interface RaceCount {
  totalRaces: number;
  racesTracked: number;
}

export interface OddsRange {
  low: number;
  high: number;
}

export interface PositionInfo {
  badHit4Pos: number;
  badHit5Pos: number;
  badHit6Pos: number;
}

export interface TestErrors {
  totalEmptyContent: number;
  totalRaceParseError: number;
  totalMongoDbError: number;
}

export interface SkipInfo {
  skippedRacesLt5: number;
  skippedRacesGt6: number;
  skippedOddsRange: number;
  skippedFavorite: number;
}

export interface Balance {
  initialBalance: number;
  finalBalance: number;
}

export interface TestResultsMeta {
  raceCount: RaceCount;
  oddsRange: OddsRange;
  positionInfo: PositionInfo;
  skipInfo: SkipInfo;
  balance: Balance;
  errors: TestErrors;
  initialStake: number;
  percentage: number;
}

export interface TestResultsRaceMeta {
  date: string;
  distance: number;
  grade: string;
  time: string;
  track: string;
  currentBalance: number;
  profit: number;
}

export interface TestResultsRealResults {
  rank: number;
  betfairOdds: number;
}

export interface Prediction {
  meta: {
    time: string;
    distance: number;
    track: string;
    grade: string;
  };
  predictions: {
    name: string;
    rawScore: number;
    percentage: number;
    rank: number;
    comment: string;
  }[];
  summary: string;
}

export interface PredictionResults {
  name: string;
  rawScore: number;
  percentage: number;
  rank: number;
  comment: string;
}

export interface TestResultsDog {
  dogName: string;
  modelPrediction: PredictionResults;
  realResults: TestResultsRealResults;
}

export interface TestResultsRace {
  raceId: number;
  meta: TestResultsRaceMeta;
  dogs: TestResultsDog[];
  summary: string;
}

export interface TestResults {
  meta: TestResultsMeta;
  races: TestResultsRace[];
  requests: Record<string, any>[];
}

export interface TimeRange {
  startTime: string;
  endTime: string | null;
}