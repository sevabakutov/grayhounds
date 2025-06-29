import { TestResults } from "@/types";

export const testResultsMock: TestResults = {
  meta: {
    raceCount: {
      totalRaces: 2,
      racesTracked: 2,
    },
    oddsRange: {
      low: 3.5,
      high: 6.3,
    },
    positionInfo: {
      badHit4Pos: 0,
      badHit5Pos: 1,
      badHit6Pos: 0,
    },
    skipInfo: {
      skippedRacesLt5: 0,
      skippedRacesGt6: 0,
      skippedOddsRange: 0,
      skippedFavorite: 0,
    },
    balance: {
      initialBalance: 1000,
      finalBalance: 1100,
    },
    errors: {
      totalEmptyContent: 0,
      totalMongoDbError: 0,
      totalRaceParseError: 0,
    },
    initialStake: 20,
    percentage: 30,
  },
  races: [
    {
      raceId: 1,
      meta: {
        date: "2024-02-13",
        time: "14:00",
        distance: 500,
        grade: "A1",
        track: "Newcastle",
        currentBalance: 1010,
        profit: 1,
      },
      dogs: [
        {
          dogName: "Dog 1",
          modelPrediction: {
            name: "Dog 1",
            rawScore: 1.1,
            percentage: 25,
            rank: 1,
            comment: "Dog 1 comment",
          },
          realResults: {
            rank: 1,
            betfairOdds: 4.5,
          },
        },
        {
          dogName: "Dog 2",
          modelPrediction: {
            name: "Dog 2",
            rawScore: 1.0,
            percentage: 20,
            rank: 2,
            comment: "Dog 2 comment",
          },
          realResults: {
            rank: 2,
            betfairOdds: 3.8,
          },
        },
        {
          dogName: "Dog 3",
          modelPrediction: {
            name: "Dog 3",
            rawScore: 0.9,
            percentage: 18,
            rank: 3,
            comment: "Dog 3 comment",
          },
          realResults: {
            rank: 3,
            betfairOdds: 5.5,
          },
        },
        {
          dogName: "Dog 4",
          modelPrediction: {
            name: "Dog 4",
            rawScore: 0.8,
            percentage: 15,
            rank: 4,
            comment: "Dog 4 comment",
          },
          realResults: {
            rank: 4,
            betfairOdds: 6.1,
          },
        },
        {
          dogName: "Dog 5",
          modelPrediction: {
            name: "Dog 5",
            rawScore: 0.7,
            percentage: 12,
            rank: 5,
            comment: "Dog 5 comment",
          },
          realResults: {
            rank: 5,
            betfairOdds: 5.9,
          },
        },
        {
          dogName: "Dog 6",
          modelPrediction: {
            name: "Dog 6",
            rawScore: 0.5,
            percentage: 10,
            rank: 6,
            comment: "Dog 6 comment",
          },
          realResults: {
            rank: 6,
            betfairOdds: 7.2,
          },
        },
      ],
      summary: "Race 1 overall summary",
    },
    {
      raceId: 2,
      meta: {
        date: "2024-02-13",
        time: "14:12",
        distance: 500,
        grade: "A1",
        track: "Newcastle",
        currentBalance: 1015,
        profit: 1,
      },
      dogs: [
        {
          dogName: "Dog A",
          modelPrediction: {
            name: "Dog A",
            rawScore: 1.2,
            percentage: 27,
            rank: 1,
            comment: "Dog A comment",
          },
          realResults: {
            rank: 1,
            betfairOdds: 4.2,
          },
        },
        {
          dogName: "Dog B",
          modelPrediction: {
            name: "Dog B",
            rawScore: 1.0,
            percentage: 21,
            rank: 2,
            comment: "Dog B comment",
          },
          realResults: {
            rank: 2,
            betfairOdds: 3.9,
          },
        },
        {
          dogName: "Dog C",
          modelPrediction: {
            name: "Dog C",
            rawScore: 0.9,
            percentage: 19,
            rank: 3,
            comment: "Dog C comment",
          },
          realResults: {
            rank: 3,
            betfairOdds: 5.3,
          },
        },
        {
          dogName: "Dog D",
          modelPrediction: {
            name: "Dog D",
            rawScore: 0.8,
            percentage: 16,
            rank: 4,
            comment: "Dog D comment",
          },
          realResults: {
            rank: 4,
            betfairOdds: 6.5,
          },
        },
        {
          dogName: "Dog E",
          modelPrediction: {
            name: "Dog E",
            rawScore: 0.7,
            percentage: 12,
            rank: 5,
            comment: "Dog E comment",
          },
          realResults: {
            rank: 5,
            betfairOdds: 5.8,
          },
        },
        {
          dogName: "Dog F",
          modelPrediction: {
            name: "Dog F",
            rawScore: 0.5,
            percentage: 9,
            rank: 6,
            comment: "Dog F comment",
          },
          realResults: {
            rank: 6,
            betfairOdds: 7.0,
          },
        },
      ],
      summary: "Race 2 overall summary",
    },
  ],
  requests: [ 
    {
      "some data": "some data"
    } 
  ]
};
